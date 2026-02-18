use crate::authz::{claims_has_permission, is_admin, is_super_admin, require_claims};
use crate::config::AppConfig;
use crate::db::Database;
use crate::handlers::errors::ApiError;
use crate::models::{
    BackupRecord, BackupSettings, Claims, CreateBackupRequest, RestoreBackupRequest,
    UpdateBackupSettingsRequest, UserRecord,
};
use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Result};
use aes_gcm::aead::{AeadInPlace, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Tag};
use base64::Engine;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::Builder as TarBuilder;
use uuid::Uuid;

static BACKUP_LOCK: Lazy<tokio::sync::Mutex<()>> = Lazy::new(|| tokio::sync::Mutex::new(()));

const RECORDS_TREE: &str = "backup_archives";
const SETTINGS_TREE: &str = "backup_settings";
const SETTINGS_KEY: &str = "global";

const BACKUP_MAGIC: &[u8; 4] = b"QFBK";
const BACKUP_FORMAT_VERSION: u32 = 1;
const BACKUP_CHUNK_SIZE: usize = 1024 * 1024; // 1 MiB

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupFileHeaderV1 {
    format: String,
    version: u32,
    created_at: String,
    include_uploads: bool,
    payload_size_bytes: u64,
    kdf: BackupKdfHeader,
    cipher: BackupCipherHeader,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupKdfHeader {
    algorithm: String,
    salt_b64: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupCipherHeader {
    algorithm: String,
    nonce_prefix_b64: String,
    chunk_size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupManifest {
    manifest_version: u32,
    created_at: String,
    created_by: String,
    include_uploads: bool,
    trees: Vec<BackupTreeSummary>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupTreeSummary {
    tree: String,
    record_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupTreeDump {
    tree: String,
    records: Vec<BackupKv>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BackupKv {
    key: String,
    #[serde(default)]
    key_base64: Option<String>,
    #[serde(default)]
    value_json: Option<serde_json::Value>,
    #[serde(default)]
    value_base64: Option<String>,
}

fn now_iso() -> String {
    crate::time::now()
}

fn require_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(response) => return Err(response),
    };
    if is_admin(&claims) || claims_has_permission(&claims, "manage_system_settings") {
        Ok(claims)
    } else {
        Err(HttpResponse::Forbidden().json(ApiError::forbidden()))
    }
}

fn get_subscription_tier(req: &HttpRequest, db: &Database) -> Option<String> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if is_super_admin(claims) {
            return Some("enterprise".to_string());
        }
        if let Some(tier) = claims.subscription_tier.clone() {
            return Some(tier);
        }
        if let Ok(Some(user)) = db.get::<UserRecord>("users", &claims.sub) {
            return user.subscription_tier.clone();
        }
    }
    None
}

fn check_backup_feature_gate(req: &HttpRequest, db: &Database) -> Result<(), HttpResponse> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if is_admin(claims) || claims_has_permission(claims, "manage_system_settings") {
            return Ok(());
        }
    }
    let tier = get_subscription_tier(req, db);
    if !crate::models::plans::check_feature_access(db, tier.as_deref(), "backups") {
        return Err(HttpResponse::Forbidden().json(ApiError::feature_locked()));
    }
    Ok(())
}

fn archives_dir(cfg: &AppConfig) -> PathBuf {
    Path::new(&cfg.backup_dir).join("archives")
}

fn backup_path(cfg: &AppConfig, backup_id: &str) -> PathBuf {
    archives_dir(cfg).join(format!("{}.qfbk", backup_id))
}

fn temp_tar_gz_path(cfg: &AppConfig, backup_id: &str) -> PathBuf {
    archives_dir(cfg).join(format!("{}.tar.gz.tmp", backup_id))
}

fn sanitize_tree_filename(tree: &str) -> String {
    let mut out = String::with_capacity(tree.len());
    for ch in tree.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "tree".into()
    } else {
        out
    }
}

fn derive_key(password_hash: &str, salt: &[u8]) -> std::result::Result<[u8; 32], String> {
    use argon2::Argon2;
    let mut out = [0u8; 32];
    if let Err(err) =
        Argon2::default().hash_password_into(password_hash.trim().as_bytes(), salt, &mut out)
    {
        return Err(format!("key derivation failed: {err}"));
    }
    Ok(out)
}

fn build_nonce(prefix_8: &[u8; 8], chunk_index: u32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(prefix_8);
    nonce[8..].copy_from_slice(&chunk_index.to_be_bytes());
    nonce
}

fn write_header(w: &mut dyn Write, header: &BackupFileHeaderV1) -> std::io::Result<()> {
    let header_json = match serde_json::to_vec(header) {
        Ok(value) => value,
        Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
    };
    if let Err(err) = w.write_all(BACKUP_MAGIC) {
        return Err(err);
    }
    if let Err(err) = w.write_all(&BACKUP_FORMAT_VERSION.to_be_bytes()) {
        return Err(err);
    }
    let len_u32: u32 = match header_json.len().try_into() {
        Ok(value) => value,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "header too large",
            ))
        }
    };
    if let Err(err) = w.write_all(&len_u32.to_be_bytes()) {
        return Err(err);
    }
    if let Err(err) = w.write_all(&header_json) {
        return Err(err);
    }
    Ok(())
}

fn read_header(r: &mut dyn Read) -> std::result::Result<BackupFileHeaderV1, String> {
    let mut magic = [0u8; 4];
    if let Err(err) = r.read_exact(&mut magic) {
        return Err(format!("backup header read failed: {err}"));
    }
    if &magic != BACKUP_MAGIC {
        return Err("Invalid backup file (bad magic)".into());
    }

    let mut ver = [0u8; 4];
    if let Err(err) = r.read_exact(&mut ver) {
        return Err(format!("backup header read failed: {err}"));
    }
    let version = u32::from_be_bytes(ver);
    if version != BACKUP_FORMAT_VERSION {
        return Err(format!("Unsupported backup version: {}", version));
    }

    let mut len_buf = [0u8; 4];
    if let Err(err) = r.read_exact(&mut len_buf) {
        return Err(format!("backup header read failed: {err}"));
    }
    let header_len = u32::from_be_bytes(len_buf) as usize;
    if header_len == 0 || header_len > 8 * 1024 * 1024 {
        return Err("Invalid backup header length".into());
    }

    let mut header_json = vec![0u8; header_len];
    if let Err(err) = r.read_exact(&mut header_json) {
        return Err(format!("backup header read failed: {err}"));
    }

    match serde_json::from_slice::<BackupFileHeaderV1>(&header_json) {
        Ok(value) => Ok(value),
        Err(err) => Err(format!("Invalid backup header JSON: {err}")),
    }
}

fn encrypt_file_to_backup(
    password_hash: &str,
    include_uploads: bool,
    tar_gz_path: &Path,
    out_path: &Path,
) -> std::result::Result<(BackupFileHeaderV1, u64), String> {
    let payload_size_bytes = match fs::metadata(tar_gz_path) {
        Ok(metadata) => metadata.len(),
        Err(err) => return Err(format!("Failed to stat payload: {err}")),
    };

    let mut salt = [0u8; 16];
    let mut nonce_prefix = [0u8; 8];
    {
        use rand::RngCore;
        let mut rng = rand::rngs::OsRng;
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce_prefix);
    }

    let key_bytes = match derive_key(password_hash, &salt) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));

    let header = BackupFileHeaderV1 {
        format: "mitote_v026_bike_connect_backend-backup".into(),
        version: BACKUP_FORMAT_VERSION,
        created_at: now_iso(),
        include_uploads,
        payload_size_bytes,
        kdf: BackupKdfHeader {
            algorithm: "argon2id".into(),
            salt_b64: base64::engine::general_purpose::STANDARD.encode(salt),
        },
        cipher: BackupCipherHeader {
            algorithm: "aes-256-gcm".into(),
            nonce_prefix_b64: base64::engine::general_purpose::STANDARD.encode(nonce_prefix),
            chunk_size_bytes: BACKUP_CHUNK_SIZE,
        },
    };

    let input = match File::open(tar_gz_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to open payload: {err}")),
    };
    let mut reader = BufReader::new(input);
    let output = match File::create(out_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to create backup: {err}")),
    };
    let mut writer = BufWriter::new(output);

    if let Err(err) = write_header(&mut writer, &header) {
        return Err(format!("Failed to write backup header: {err}"));
    }

    let mut chunk_index: u32 = 0;
    let mut total_written_plain: u64 = 0;

    loop {
        let mut buf = vec![0u8; BACKUP_CHUNK_SIZE];
        let n = match reader.read(&mut buf) {
            Ok(value) => value,
            Err(err) => return Err(format!("Failed to read payload: {err}")),
        };
        if n == 0 {
            break;
        }
        buf.truncate(n);
        total_written_plain += n as u64;

        let nonce_bytes = build_nonce(&nonce_prefix, chunk_index);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
        let tag = match cipher.encrypt_in_place_detached(nonce, b"", &mut buf) {
            Ok(value) => value,
            Err(_) => return Err("Encryption failed".to_string()),
        };

        let ct_len: u32 = match buf.len().try_into() {
            Ok(value) => value,
            Err(_) => return Err("Payload chunk too large".to_string()),
        };
        if let Err(err) = writer.write_all(&ct_len.to_be_bytes()) {
            return Err(format!("Failed to write backup: {err}"));
        }
        if let Err(err) = writer.write_all(&buf) {
            return Err(format!("Failed to write backup: {err}"));
        }
        if let Err(err) = writer.write_all(tag.as_slice()) {
            return Err(format!("Failed to write backup: {err}"));
        }

        chunk_index = match chunk_index.checked_add(1) {
            Some(value) => value,
            None => return Err("Backup too large".to_string()),
        };
    }

    if let Err(err) = writer.flush() {
        return Err(format!("Failed to finalize backup: {err}"));
    }

    if total_written_plain != payload_size_bytes {
        return Err("Backup encryption failed (size mismatch)".into());
    }

    let encrypted_size_bytes = match fs::metadata(out_path) {
        Ok(metadata) => metadata.len(),
        Err(err) => return Err(format!("Failed to stat backup: {err}")),
    };

    Ok((header, encrypted_size_bytes))
}

fn decrypt_backup_to_tar_gz(
    password_hash: &str,
    backup_path: &Path,
    out_tar_gz_path: &Path,
) -> std::result::Result<BackupFileHeaderV1, String> {
    let input = match File::open(backup_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to open backup: {err}")),
    };
    let mut reader = BufReader::new(input);
    let header = match read_header(&mut reader) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };

    let salt =
        match base64::engine::general_purpose::STANDARD.decode(header.kdf.salt_b64.as_bytes()) {
            Ok(value) => value,
            Err(err) => return Err(format!("Invalid salt in header: {err}")),
        };
    let nonce_prefix_vec = match base64::engine::general_purpose::STANDARD
        .decode(header.cipher.nonce_prefix_b64.as_bytes())
    {
        Ok(value) => value,
        Err(err) => return Err(format!("Invalid nonce prefix in header: {err}")),
    };
    if nonce_prefix_vec.len() != 8 {
        return Err("Invalid nonce prefix length".into());
    }
    let mut nonce_prefix = [0u8; 8];
    nonce_prefix.copy_from_slice(&nonce_prefix_vec);

    let key_bytes = match derive_key(password_hash, &salt) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));

    let output = match File::create(out_tar_gz_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to create temp payload: {err}")),
    };
    let mut writer = BufWriter::new(output);

    let mut chunk_index: u32 = 0;
    let mut total_plain: u64 = 0;

    loop {
        let mut len_buf = [0u8; 4];
        match reader.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(format!("Failed to read backup: {e}")),
        }
        let ct_len = u32::from_be_bytes(len_buf) as usize;
        if ct_len == 0 || ct_len > 64 * 1024 * 1024 {
            return Err("Invalid encrypted chunk size".into());
        }

        let mut ciphertext = vec![0u8; ct_len];
        if let Err(err) = reader.read_exact(&mut ciphertext) {
            return Err(format!("Failed to read backup chunk: {err}"));
        }
        let mut tag_buf = [0u8; 16];
        if let Err(err) = reader.read_exact(&mut tag_buf) {
            return Err(format!("Failed to read backup tag: {err}"));
        }

        let nonce_bytes = build_nonce(&nonce_prefix, chunk_index);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
        if cipher
            .decrypt_in_place_detached(nonce, b"", &mut ciphertext, Tag::from_slice(&tag_buf))
            .is_err()
        {
            return Err("Decryption failed (wrong password or corrupted backup)".to_string());
        }

        if let Err(err) = writer.write_all(&ciphertext) {
            return Err(format!("Failed to write temp payload: {err}"));
        }
        total_plain += ciphertext.len() as u64;

        chunk_index = match chunk_index.checked_add(1) {
            Some(value) => value,
            None => return Err("Backup too large".to_string()),
        };
    }

    if let Err(err) = writer.flush() {
        return Err(format!("Failed to finalize temp payload: {err}"));
    }

    if total_plain != header.payload_size_bytes {
        return Err("Decryption failed (size mismatch)".into());
    }

    Ok(header)
}

fn build_backup_tar_gz(
    db: &Database,
    cfg: &AppConfig,
    created_by: &str,
    include_uploads: bool,
    out_tar_gz_path: &Path,
) -> std::result::Result<BackupManifest, String> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Cursor;

    let out_file = match File::create(out_tar_gz_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to create backup payload: {err}")),
    };
    let encoder = GzEncoder::new(out_file, Compression::default());
    let mut tar = TarBuilder::new(encoder);

    let mut tree_summaries: Vec<BackupTreeSummary> = Vec::new();

    let mut names: Vec<String> = db
        .db
        .tree_names()
        .into_iter()
        .filter_map(|ivec| String::from_utf8(ivec.to_vec()).ok())
        .collect();
    names.sort();

    for tree_name in names {
        let tree = match db
            .db
            .open_tree(&tree_name)
            .map_err(|e| format!("Failed to open tree {tree_name}: {e}"))
        {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        let mut records: Vec<BackupKv> = Vec::new();
        for item in tree.iter() {
            let (k, v) = match item.map_err(|e| format!("Failed to read tree {tree_name}: {e}")) {
                Ok(value) => value,
                Err(err) => return Err(err),
            };
            let (key, key_base64) = match String::from_utf8(k.to_vec()) {
                Ok(s) => (s, None),
                Err(_) => {
                    let b64 = base64::engine::general_purpose::STANDARD.encode(k.as_ref());
                    (b64.clone(), Some(b64))
                }
            };

            let value_json = serde_json::from_slice::<serde_json::Value>(v.as_ref()).ok();
            let value_base64 = if value_json.is_none() {
                Some(base64::engine::general_purpose::STANDARD.encode(v.as_ref()))
            } else {
                None
            };
            records.push(BackupKv {
                key,
                key_base64,
                value_json,
                value_base64,
            });
        }

        tree_summaries.push(BackupTreeSummary {
            tree: tree_name.clone(),
            record_count: records.len(),
        });

        let dump = BackupTreeDump {
            tree: tree_name.clone(),
            records,
        };
        let bytes = match serde_json::to_vec(&dump) {
            Ok(value) => value,
            Err(err) => return Err(format!("Failed to serialize tree {tree_name}: {err}")),
        };

        let filename = format!("db/trees/{}.json", sanitize_tree_filename(&tree_name));
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        if let Err(err) = tar.append_data(&mut header, filename, &mut Cursor::new(bytes)) {
            return Err(format!("Failed to write backup payload: {err}"));
        }
    }

    let manifest = BackupManifest {
        manifest_version: 1,
        created_at: now_iso(),
        created_by: created_by.to_string(),
        include_uploads,
        trees: tree_summaries,
    };

    let manifest_bytes = match serde_json::to_vec(&manifest) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to serialize manifest: {err}")),
    };
    let mut manifest_header = tar::Header::new_gnu();
    manifest_header.set_size(manifest_bytes.len() as u64);
    manifest_header.set_mode(0o644);
    manifest_header.set_cksum();
    if let Err(err) = tar.append_data(
        &mut manifest_header,
        "manifest.json",
        &mut Cursor::new(manifest_bytes),
    ) {
        return Err(format!("Failed to write backup payload: {err}"));
    }

    if include_uploads {
        let uploads_path = Path::new(&cfg.uploads_path);
        if uploads_path.exists() {
            if let Err(err) = tar.append_dir_all("uploads", uploads_path) {
                return Err(format!("Failed to add uploads to backup: {err}"));
            }
        }
    }

    if let Err(err) = tar.finish() {
        return Err(format!("Failed to finalize backup payload: {err}"));
    }

    Ok(manifest)
}

fn strip_leading_dot(path: &Path) -> &Path {
    if let Ok(stripped) = path.strip_prefix("./") {
        stripped
    } else {
        path
    }
}

fn is_safe_rel_path(path: &Path) -> bool {
    use std::path::Component;
    !path.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

fn restore_from_tar_gz(
    db: &Database,
    cfg: &AppConfig,
    tar_gz_path: &Path,
) -> std::result::Result<(), String> {
    use flate2::read::GzDecoder;

    // Pass 1: read manifest (do not wipe DB until we can read it)
    let f1 = match File::open(tar_gz_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to open payload: {err}")),
    };
    let gz1 = GzDecoder::new(f1);
    let mut ar1 = tar::Archive::new(gz1);
    let mut manifest: Option<BackupManifest> = None;
    let entries = match ar1
        .entries()
        .map_err(|e| format!("Failed to read archive: {e}"))
    {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    for entry in entries {
        let mut entry = match entry.map_err(|e| format!("Failed to read archive: {e}")) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let p = match entry
            .path()
            .map_err(|e| format!("Invalid archive path: {e}"))
        {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let p = strip_leading_dot(&p);
        if p == Path::new("manifest.json") {
            let mut buf: Vec<u8> = Vec::new();
            if let Err(err) = entry.read_to_end(&mut buf) {
                return Err(format!("Failed to read manifest: {err}"));
            }
            let parsed_manifest = match serde_json::from_slice::<BackupManifest>(&buf) {
                Ok(value) => value,
                Err(err) => return Err(format!("Invalid manifest JSON: {err}")),
            };
            manifest = Some(parsed_manifest);
            break;
        }
    }
    let manifest = match manifest {
        Some(m) => m,
        None => return Err("Invalid backup: manifest.json missing".into()),
    };

    // Wipe all trees before restoring
    let tree_names: Vec<String> = db
        .db
        .tree_names()
        .into_iter()
        .filter_map(|ivec| String::from_utf8(ivec.to_vec()).ok())
        .collect();
    for name in tree_names {
        let tree = match db
            .db
            .open_tree(&name)
            .map_err(|e| format!("Failed to open tree {name}: {e}"))
        {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        if let Err(err) = tree.clear() {
            return Err(format!("Failed to clear tree {name}: {err}"));
        }
    }
    if let Err(err) = db.db.flush() {
        return Err(format!("Failed to flush DB: {err}"));
    }

    // Prepare uploads directory for overwrite (only if backup includes it)
    let uploads_dir = Path::new(&cfg.uploads_path);
    if manifest.include_uploads {
        if uploads_dir.exists() {
            let _ = fs::remove_dir_all(uploads_dir);
        }
        let _ = fs::create_dir_all(uploads_dir);
    }

    // Pass 2: restore trees + uploads
    let f2 = match File::open(tar_gz_path) {
        Ok(value) => value,
        Err(err) => return Err(format!("Failed to open payload: {err}")),
    };
    let gz2 = GzDecoder::new(f2);
    let mut ar2 = tar::Archive::new(gz2);

    let entries = match ar2
        .entries()
        .map_err(|e| format!("Failed to read archive: {e}"))
    {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    for entry in entries {
        let mut entry = match entry.map_err(|e| format!("Failed to read archive entry: {e}")) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let p = match entry
            .path()
            .map_err(|e| format!("Invalid archive path: {e}"))
        {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let p = strip_leading_dot(&p).to_path_buf();
        if !is_safe_rel_path(&p) {
            continue;
        }

        if p.starts_with("db/trees/") && p.extension().and_then(|s| s.to_str()) == Some("json") {
            let mut buf: Vec<u8> = Vec::new();
            if let Err(err) = entry.read_to_end(&mut buf) {
                return Err(format!("Failed to read tree dump: {err}"));
            }
            let dump: BackupTreeDump = match serde_json::from_slice(&buf) {
                Ok(value) => value,
                Err(err) => return Err(format!("Invalid tree dump JSON: {err}")),
            };

            let tree = match db
                .db
                .open_tree(&dump.tree)
                .map_err(|e| format!("Failed to open tree {}: {e}", dump.tree))
            {
                Ok(value) => value,
                Err(err) => return Err(err),
            };
            for rec in dump.records {
                let key_bytes = if let Some(b64) = rec.key_base64 {
                    match base64::engine::general_purpose::STANDARD.decode(b64.as_bytes()) {
                        Ok(value) => value,
                        Err(err) => return Err(format!("Failed to decode base64 key: {err}")),
                    }
                } else {
                    rec.key.as_bytes().to_vec()
                };
                if let Some(v) = rec.value_json {
                    let val_bytes = match serde_json::to_vec(&v) {
                        Ok(value) => value,
                        Err(err) => return Err(format!("Failed to encode JSON value: {err}")),
                    };
                    if let Err(err) = tree.insert(key_bytes, val_bytes) {
                        return Err(format!("Failed to insert record: {err}"));
                    }
                } else if let Some(b64) = rec.value_base64 {
                    let val_bytes = match base64::engine::general_purpose::STANDARD
                        .decode(b64.as_bytes())
                    {
                        Ok(value) => value,
                        Err(err) => return Err(format!("Failed to decode base64 value: {err}")),
                    };
                    if let Err(err) = tree.insert(key_bytes, val_bytes) {
                        return Err(format!("Failed to insert record: {err}"));
                    }
                }
            }
            continue;
        }

        if manifest.include_uploads && p.starts_with("uploads/") {
            let rel = p.strip_prefix("uploads").unwrap_or_else(|_| Path::new(""));
            if !is_safe_rel_path(rel) {
                continue;
            }
            let out_path = uploads_dir.join(rel);
            if entry.header().entry_type().is_dir() {
                let _ = fs::create_dir_all(&out_path);
                continue;
            }
            if let Some(parent) = out_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut out = match File::create(&out_path) {
                Ok(value) => value,
                Err(err) => return Err(format!("Failed to write upload file: {err}")),
            };
            if let Err(err) = std::io::copy(&mut entry, &mut out) {
                return Err(format!("Failed to write upload file: {err}"));
            }
            continue;
        }
    }

    if let Err(err) = db.db.flush() {
        return Err(format!("Failed to flush DB: {err}"));
    }
    Ok(())
}

async fn load_settings(db: &Database) -> BackupSettings {
    db.get::<BackupSettings>(SETTINGS_TREE, SETTINGS_KEY)
        .ok()
        .flatten()
        .unwrap_or_default()
}

async fn save_settings(
    db: &Database,
    mut settings: BackupSettings,
) -> std::result::Result<BackupSettings, String> {
    settings.touch();
    if let Err(err) = db.insert(SETTINGS_TREE, SETTINGS_KEY, &settings) {
        return Err(format!("Failed to save settings: {err}"));
    }
    Ok(settings)
}

async fn prune_scheduled_backups(
    db: &Database,
    cfg: &AppConfig,
    retention: usize,
) -> std::result::Result<(), String> {
    let mut backups: Vec<BackupRecord> = db.list(RECORDS_TREE).unwrap_or_default();
    backups.retain(|b| b.kind == "scheduled");
    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // newest first
    if backups.len() <= retention {
        return Ok(());
    }
    for rec in backups.into_iter().skip(retention) {
        let path = backup_path(cfg, &rec.id);
        let _ = fs::remove_file(path);
        let _ = db.delete(RECORDS_TREE, &rec.id);
    }
    Ok(())
}

pub fn start_backup_scheduler(db: Database, cfg: AppConfig) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let settings = load_settings(&db).await;
            if !settings.enabled {
                continue;
            }
            let password_hash = match settings.encryption_password_hash.clone() {
                Some(p) if !p.trim().is_empty() => p,
                _ => continue,
            };

            let interval = Duration::from_secs(settings.interval_minutes.saturating_mul(60));
            let last_run = settings
                .last_run_at
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));
            let due = match last_run {
                None => true,
                Some(ts) => {
                    Utc::now()
                        .signed_duration_since(ts)
                        .to_std()
                        .unwrap_or_default()
                        >= interval
                }
            };
            if !due {
                continue;
            }

            let _guard = BACKUP_LOCK.lock().await;
            let backup_id = Uuid::new_v4().to_string();
            let tar_gz_path = temp_tar_gz_path(&cfg, &backup_id);
            let out_path = backup_path(&cfg, &backup_id);
            let filename = format!("{}.qfbk", backup_id);
            let include_uploads = settings.include_uploads;

            let run_result = tokio::task::spawn_blocking({
                let db = db.clone();
                let cfg = cfg.clone();
                let password_hash = password_hash.clone();
                move || {
                    if let Err(err) = fs::create_dir_all(archives_dir(&cfg)) {
                        return Err(format!("Failed to create archives dir: {err}"));
                    }
                    let _manifest = match build_backup_tar_gz(
                        &db,
                        &cfg,
                        "system",
                        include_uploads,
                        &tar_gz_path,
                    ) {
                        Ok(value) => value,
                        Err(err) => return Err(err),
                    };
                    let (_header, enc_size) = match encrypt_file_to_backup(
                        &password_hash,
                        include_uploads,
                        &tar_gz_path,
                        &out_path,
                    ) {
                        Ok(value) => value,
                        Err(err) => return Err(err),
                    };
                    let _ = fs::remove_file(&tar_gz_path);
                    Ok::<u64, String>(enc_size)
                }
            })
            .await;

            match run_result {
                Ok(Ok(enc_size)) => {
                    let record = BackupRecord::new(
                        backup_id.clone(),
                        "system".into(),
                        "scheduled".into(),
                        filename,
                        enc_size,
                        include_uploads,
                        BACKUP_FORMAT_VERSION,
                    );
                    let _ = db.insert(RECORDS_TREE, &record.id, &record);

                    let mut updated = settings.clone();
                    updated.last_run_at = Some(Utc::now().to_rfc3339());
                    let _ = save_settings(&db, updated).await;
                    let _ = prune_scheduled_backups(&db, &cfg, settings.retention_count).await;
                }
                _ => {
                    // ignore; scheduler is best-effort
                }
            }
        }
    });
}

/// #feature[backups]
#[get("/backups")]
pub async fn list_backups(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let mut items: Vec<BackupRecord> = db.list(RECORDS_TREE).unwrap_or_default();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(HttpResponse::Ok().json(items))
}

/// #feature[backups]
#[get("/backups/settings")]
pub async fn get_backup_settings(
    req: HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let settings = load_settings(&db).await;
    Ok(HttpResponse::Ok().json(settings))
}

/// #feature[backups]
#[put("/backups/settings")]
pub async fn update_backup_settings(
    req: HttpRequest,
    db: web::Data<Database>,
    body: web::Json<UpdateBackupSettingsRequest>,
) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let patch = body.into_inner();
    let mut settings = load_settings(&db).await;
    if let Some(v) = patch.enabled {
        settings.enabled = v;
    }
    if let Some(v) = patch.interval_minutes {
        settings.interval_minutes = v.max(1);
    }
    if let Some(v) = patch.retention_count {
        settings.retention_count = v.max(1).min(200);
    }
    if let Some(v) = patch.include_uploads {
        settings.include_uploads = v;
    }
    if let Some(v) = patch.encryption_password_hash {
        if v.trim().is_empty() {
            settings.encryption_password_hash = None;
        } else {
            settings.encryption_password_hash = Some(v);
        }
    }

    match save_settings(&db, settings).await {
        Ok(saved) => Ok(HttpResponse::Ok().json(saved)),
        Err(msg) => Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "BACKUP_SETTINGS_SAVE_FAILED",
            msg,
            false,
        ))),
    }
}

/// #feature[backups]
#[post("/backups")]
pub async fn create_backup(
    req: HttpRequest,
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    body: web::Json<CreateBackupRequest>,
) -> Result<HttpResponse> {
    let claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let payload = body.into_inner();
    if payload.password_hash.trim().is_empty() {
        let mut errors = HashMap::new();
        errors.insert("password_hash".into(), "required".into());
        return Ok(
            HttpResponse::UnprocessableEntity().json(ApiError::validation(
                "Missing required fields".into(),
                errors,
            )),
        );
    }

    let settings = load_settings(&db).await;
    let include_uploads = payload.include_uploads.unwrap_or(settings.include_uploads);

    let _guard = BACKUP_LOCK.lock().await;

    let backup_id = Uuid::new_v4().to_string();
    let tar_gz_path = temp_tar_gz_path(&cfg, &backup_id);
    let out_path = backup_path(&cfg, &backup_id);
    let created_by = claims.sub.clone();
    let created_by_for_job = created_by.clone();
    let filename = format!("{}.qfbk", backup_id);
    let password_hash = payload.password_hash.clone();
    let cfg_clone = cfg.get_ref().clone();
    let db_clone = db.get_ref().clone();

    let create_result = tokio::task::spawn_blocking(move || {
        if let Err(err) = fs::create_dir_all(archives_dir(&cfg_clone)) {
            return Err(format!("Failed to create archives dir: {err}"));
        }
        let _manifest = match build_backup_tar_gz(
            &db_clone,
            &cfg_clone,
            &created_by_for_job,
            include_uploads,
            &tar_gz_path,
        ) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let (_header, enc_size) = match encrypt_file_to_backup(
            &password_hash,
            include_uploads,
            &tar_gz_path,
            &out_path,
        ) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let _ = fs::remove_file(&tar_gz_path);
        Ok::<u64, String>(enc_size)
    })
    .await;

    let enc_size = match create_result {
        Ok(Ok(size)) => size,
        Ok(Err(msg)) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "BACKUP_CREATE_FAILED",
                msg,
                false,
            )))
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "BACKUP_CREATE_FAILED",
                e,
                false,
            )))
        }
    };

    let record = BackupRecord::new(
        backup_id.clone(),
        created_by,
        "manual".into(),
        filename,
        enc_size,
        include_uploads,
        BACKUP_FORMAT_VERSION,
    );
    if let Err(e) = db.insert(RECORDS_TREE, &record.id, &record) {
        return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "BACKUP_RECORD_SAVE_FAILED",
            e,
            false,
        )));
    }

    Ok(HttpResponse::Created().json(record))
}

/// #feature[backups]
#[get("/backups/{backup_id}/download")]
pub async fn download_backup(
    req: HttpRequest,
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let backup_id = path.into_inner();
    let record: BackupRecord = match db.get(RECORDS_TREE, &backup_id) {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(HttpResponse::NotFound().json(ApiError::not_found("Backup"))),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "BACKUP_GET_FAILED",
                e,
                false,
            )))
        }
    };

    let file_path = backup_path(&cfg, &record.id);
    let data = match tokio::fs::read(&file_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "BACKUP_READ_FAILED",
                e,
                false,
            )))
        }
    };

    let filename = format!(
        "mitote_v026_bike_connect_backend-backup-{}-{}.qfbk",
        record.kind,
        record.created_at.replace(':', "-")
    );
    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(data))
}

/// #feature[backups]
#[post("/backups/upload")]
pub async fn upload_backup(
    req: HttpRequest,
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    body: web::Bytes,
) -> Result<HttpResponse> {
    let claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    if body.len() < 16 {
        return Ok(HttpResponse::BadRequest().json(ApiError::bad_request("Invalid backup upload")));
    }
    if &body[..4] != BACKUP_MAGIC {
        return Ok(HttpResponse::BadRequest()
            .json(ApiError::bad_request("Invalid backup upload (bad magic)")));
    }

    let _guard = BACKUP_LOCK.lock().await;

    let backup_id = Uuid::new_v4().to_string();
    let out_path = backup_path(&cfg, &backup_id);
    if let Err(e) = tokio::fs::create_dir_all(archives_dir(&cfg)).await {
        return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "BACKUP_DIR_CREATE_FAILED",
            e,
            false,
        )));
    }
    if let Err(e) = tokio::fs::write(&out_path, body).await {
        return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "BACKUP_UPLOAD_FAILED",
            e,
            false,
        )));
    }

    let size_bytes = match tokio::fs::metadata(&out_path).await {
        Ok(md) => md.len(),
        Err(_) => 0,
    };

    let record = BackupRecord::new(
        backup_id.clone(),
        claims.sub,
        "upload".into(),
        out_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("backup.qfbk")
            .to_string(),
        size_bytes,
        true,
        BACKUP_FORMAT_VERSION,
    );
    if let Err(e) = db.insert(RECORDS_TREE, &record.id, &record) {
        return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "BACKUP_RECORD_SAVE_FAILED",
            e,
            false,
        )));
    }

    Ok(HttpResponse::Created().json(record))
}

/// #feature[backups]
#[post("/backups/{backup_id}/restore")]
pub async fn restore_backup(
    req: HttpRequest,
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    path: web::Path<String>,
    body: web::Json<RestoreBackupRequest>,
) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let backup_id = path.into_inner();
    let request = body.into_inner();
    if !request.confirm {
        let mut errors = HashMap::new();
        errors.insert("confirm".into(), "must be true".into());
        return Ok(
            HttpResponse::UnprocessableEntity().json(ApiError::validation(
                "Restore requires confirmation".into(),
                errors,
            )),
        );
    }
    if request.password_hash.trim().is_empty() {
        let mut errors = HashMap::new();
        errors.insert("password_hash".into(), "required".into());
        return Ok(
            HttpResponse::UnprocessableEntity().json(ApiError::validation(
                "Missing required fields".into(),
                errors,
            )),
        );
    }

    let record_exists: Option<BackupRecord> = db.get(RECORDS_TREE, &backup_id).ok().flatten();
    if record_exists.is_none() {
        return Ok(HttpResponse::NotFound().json(ApiError::not_found("Backup")));
    }

    let _guard = BACKUP_LOCK.lock().await;

    let backup_file = backup_path(&cfg, &backup_id);
    if !backup_file.exists() {
        return Ok(HttpResponse::NotFound().json(ApiError::not_found("Backup file")));
    }

    let temp_payload = temp_tar_gz_path(&cfg, &format!("restore_{}", backup_id));
    let db_clone = db.get_ref().clone();
    let cfg_clone = cfg.get_ref().clone();
    let password_hash = request.password_hash.clone();

    let restore_result = tokio::task::spawn_blocking(move || {
        if let Err(err) = fs::create_dir_all(archives_dir(&cfg_clone)) {
            return Err(format!("Failed to create archives dir: {err}"));
        }
        let _header = match decrypt_backup_to_tar_gz(&password_hash, &backup_file, &temp_payload) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let res = restore_from_tar_gz(&db_clone, &cfg_clone, &temp_payload);
        let _ = fs::remove_file(&temp_payload);
        res
    })
    .await;

    match restore_result {
        Ok(Ok(())) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "message": "Backup restored"
        }))),
        Ok(Err(msg)) => Ok(HttpResponse::BadRequest().json(ApiError::bad_request(&msg))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "BACKUP_RESTORE_FAILED",
            e,
            false,
        ))),
    }
}

/// #feature[backups]
#[delete("/backups/{backup_id}")]
pub async fn delete_backup(
    req: HttpRequest,
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(c) => c,
        Err(e) => return Ok(e),
    };
    if let Err(e) = check_backup_feature_gate(&req, &db) {
        return Ok(e);
    }

    let backup_id = path.into_inner();
    let record: BackupRecord = match db.get(RECORDS_TREE, &backup_id) {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(HttpResponse::NotFound().json(ApiError::not_found("Backup"))),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "BACKUP_GET_FAILED",
                e,
                false,
            )))
        }
    };

    let _guard = BACKUP_LOCK.lock().await;

    let file_path = backup_path(&cfg, &record.id);
    let _ = tokio::fs::remove_file(&file_path).await;
    let _ = db.delete(RECORDS_TREE, &record.id);

    Ok(HttpResponse::Ok().finish())
}
