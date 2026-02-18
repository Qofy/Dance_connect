use anyhow::{Context, Result};
use chrono::Utc;
use log::{error, info, trace, warn};
use sled::Db;
use std::{
    collections::HashMap,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

const EXPORT_MAGIC: &[u8; 8] = b"QFEXP001";
const EXPORT_VERSION: u32 = 1;
const EXPORT_FILE_NAME: &str = "export.bin";
const RESTORE_SIZE_RATIO_THRESHOLD: u64 = 10;
const RESTORE_RECORD_RATIO_THRESHOLD: usize = 10;
const MIN_BACKUP_SIZE_BYTES_FOR_RATIO: u64 = 1024 * 1024;
const MIN_BACKUP_RECORDS_FOR_RATIO: usize = 100;
const SMALL_DB_SIZE_BYTES: u64 = 512 * 1024;

pub struct BackupManager {
    db_path: PathBuf,
    backup_dir: PathBuf,
    name_template: String,
    lock: Arc<Mutex<()>>,
    db_handle: Arc<Mutex<Option<Arc<Db>>>>,
}

#[derive(Debug, Clone)]
pub enum BackupValidation {
    Export { record_count: usize },
    Legacy { db_size: u64 },
}

#[derive(Debug, Clone)]
struct DbStats {
    exists: bool,
    ready: bool,
    file_size: u64,
    modified: Option<chrono::DateTime<Utc>>,
    record_count: Option<usize>,
}

#[derive(Debug, Clone)]
struct BackupStats {
    export_path: Option<PathBuf>,
    size_bytes: u64,
    modified: Option<chrono::DateTime<Utc>>,
    record_count: Option<usize>,
}

impl BackupManager {
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        db_path: P,
        backup_dir: Q,
        name_template: &str,
    ) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            backup_dir: backup_dir.as_ref().to_path_buf(),
            name_template: name_template.to_string(),
            lock: Arc::new(Mutex::new(())),
            db_handle: Arc::new(Mutex::new(None)),
        }
    }

    #[allow(dead_code)]
    pub fn lock_handle(&self) -> Arc<Mutex<()>> {
        self.lock.clone()
    }

    pub async fn set_db_handle(&self, db: Arc<Db>) {
        let mut handle = self.db_handle.lock().await;
        *handle = Some(db);
    }

    async fn get_db_handle(&self) -> Result<Arc<Db>> {
        let handle = self.db_handle.lock().await;
        handle
            .as_ref()
            .cloned()
            .context("BackupManager DB handle not set")
    }

    pub async fn run(self: Arc<Self>, interval: Duration, retention: usize) {
        tokio::fs::create_dir_all(&self.backup_dir).await.ok();
        loop {
            tokio::time::sleep(interval).await;

            let lock = self.lock.clone();
            // 5 minutes timeout to acquire lock
            let acquired = tokio::time::timeout(Duration::from_secs(300), lock.lock()).await;
            if acquired.is_ok() {
                let mut attempts = 0;
                let mut last_err: Option<anyhow::Error> = None;
                while attempts < 3 {
                    attempts += 1;
                    match tokio::time::timeout(Duration::from_secs(180), self.do_backup(retention))
                        .await
                    {
                        Ok(Ok(())) => {
                            trace!("Sled backup completed (attempt {} of 3)", attempts);
                            last_err = None;
                            break;
                        }
                        Ok(Err(e)) => {
                            warn!("Sled backup error on attempt {}: {}", attempts, e);
                            last_err = Some(e);
                        }
                        Err(_) => {
                            warn!("Sled backup timed out on attempt {}", attempts);
                            last_err = Some(anyhow::anyhow!("timeout"));
                        }
                    }
                }
                if let Some(e) = last_err {
                    error!("Sled backup failed after 3 attempts: {}", e);
                }
            } else {
                warn!("Backup skipped: another backup in progress for >5 minutes");
            }
        }
    }

    pub fn build_backup_dir(&self) -> PathBuf {
        let ts = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let name = self.name_template.replace("{{timestamp}}", &ts);
        self.backup_dir.join(name)
    }

    pub async fn export_to_dir(&self, target_dir: &Path, force: bool) -> Result<()> {
        let db = self.get_db_handle().await?;
        let db_path = self.db_path.clone();
        let target_dir = target_dir.to_path_buf();
        tokio::task::spawn_blocking(move || {
            ensure_empty_dir(&target_dir, force)?;
            if !is_db_ready_for_backup(&db_path)? {
                anyhow::bail!("Database not ready for backup at {:?}", db_path);
            }
            let export_path = export_file_path(&target_dir);
            let export_result = export_db_to_file(db.as_ref(), &export_path);
            if let Err(err) = export_result {
                let _ = std::fs::remove_dir_all(&target_dir);
                return Err(err);
            }
            Ok(())
        })
        .await??;
        Ok(())
    }

    async fn do_backup(&self, retention: usize) -> Result<()> {
        let ts = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let db_path = self.db_path.clone();
        let name = self.name_template.replace("{{timestamp}}", &ts);
        let dst = self.backup_dir.join(name);
        let db = self.get_db_handle().await?;
        // perform blocking export in a blocking thread
        let backed_up = tokio::task::spawn_blocking(move || {
            if !is_db_ready_for_backup(&db_path)? {
                warn!(
                    "Skipping backup: database not ready (missing conf/db or db empty) at {:?}",
                    db_path
                );
                return Ok(false);
            }
            std::fs::create_dir_all(&dst)?;
            let export_path = export_file_path(&dst);
            let export_result = export_db_to_file(db.as_ref(), &export_path);
            if let Err(err) = export_result {
                let _ = std::fs::remove_dir_all(&dst);
                return Err(err);
            }
            Ok(true)
        })
        .await??;
        if backed_up {
            self.prune_old_backups(retention).await?;
        }
        Ok(())
    }

    fn name_prefix(&self) -> String {
        match self.name_template.split("{{timestamp}}").next() {
            Some(p) => p.to_string(),
            None => String::new(),
        }
    }

    /// List backup directories created by this service, newest first
    pub async fn list_backups_desc(&self) -> Result<Vec<PathBuf>> {
        let prefix = self.name_prefix();
        let mut entries = tokio::fs::read_dir(&self.backup_dir).await?;
        let mut items: Vec<(String, PathBuf)> = Vec::new();
        while let Some(e) = entries.next_entry().await? {
            let name = e.file_name().to_string_lossy().to_string();
            if !prefix.is_empty() && !name.starts_with(&prefix) {
                continue;
            }
            if let Ok(md) = e.metadata().await {
                if md.is_dir() {
                    items.push((name, e.path()));
                }
            }
        }
        // Newest first (timestamp is baked into name)
        items.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(items.into_iter().map(|(_, p)| p).collect())
    }

    async fn prune_old_backups(&self, keep: usize) -> Result<()> {
        let prefix = self.name_prefix();
        let mut entries = tokio::fs::read_dir(&self.backup_dir).await?;
        let mut items: Vec<(String, PathBuf)> = Vec::new();
        while let Some(e) = entries.next_entry().await? {
            let name = e.file_name().to_string_lossy().to_string();
            // only consider backup dirs we created
            if !prefix.is_empty() && !name.starts_with(&prefix) {
                continue;
            }
            if let Ok(md) = e.metadata().await {
                if md.is_dir() {
                    items.push((name, e.path()));
                }
            }
        }
        // Sort by name (timestamp included) ascending, so oldest first
        items.sort_by(|a, b| a.0.cmp(&b.0));
        let remove_count = items.len().saturating_sub(keep);
        for (_name, path) in items.iter().take(remove_count) {
            let _ = tokio::fs::remove_dir_all(path).await;
        }
        Ok(())
    }

    /// Get the latest backup directory (sorted by timestamp in filename)
    pub async fn get_latest_backup(&self) -> Result<Option<PathBuf>> {
        let prefix = self.name_prefix();

        let mut entries = match tokio::fs::read_dir(&self.backup_dir).await {
            Ok(e) => e,
            Err(err) => {
                warn!("Failed to read backup directory: {}", err);
                return Ok(None);
            }
        };

        let mut items: Vec<(String, PathBuf)> = Vec::new();

        loop {
            let entry = match entries.next_entry().await {
                Ok(Some(e)) => e,
                Ok(None) => break,
                Err(err) => {
                    warn!("Error reading backup directory entry: {}", err);
                    continue;
                }
            };

            let name = entry.file_name().to_string_lossy().to_string();

            // only consider backup dirs we created
            if !prefix.is_empty() && !name.starts_with(&prefix) {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(md) => md,
                Err(err) => {
                    warn!("Failed to read metadata for {}: {}", name, err);
                    continue;
                }
            };

            if metadata.is_dir() {
                items.push((name, entry.path()));
            }
        }

        if items.is_empty() {
            info!("No backup directories found");
            return Ok(None);
        }

        // Sort by name (timestamp included) descending, so latest first
        items.sort_by(|a, b| b.0.cmp(&a.0));
        info!("Found {} backups, latest: {}", items.len(), items[0].0);
        Ok(Some(items[0].1.clone()))
    }

    /// Restore database from the latest backup
    pub async fn restore_from_latest(&self) -> Result<bool> {
        let _guard = self.lock.lock().await;
        let backup_path = match self.get_latest_backup().await {
            Ok(Some(path)) => {
                info!("Latest backup found: {:?}", path);
                path
            }
            Ok(None) => {
                info!("No backups found to restore from");
                return Ok(false);
            }
            Err(err) => {
                error!("Failed to get latest backup: {}", err);
                return Ok(false);
            }
        };

        info!(
            "Attempting to restore database from backup: {:?}",
            backup_path
        );

        let backup_stats = match self.backup_stats(&backup_path).await {
            Ok(stats) => stats,
            Err(err) => {
                warn!("Failed to inspect latest backup: {}", err);
                return Ok(false);
            }
        };

        if backup_stats.size_bytes == 0 {
            warn!("Latest backup appears empty, skipping restore");
            return Ok(false);
        }

        let db_stats = match self.db_stats().await {
            Ok(stats) => stats,
            Err(err) => {
                warn!("Failed to inspect database status: {}", err);
                return Ok(false);
            }
        };

        let (should_restore, reason) = should_restore_from_backup(&db_stats, &backup_stats);
        info!("Restore decision: {} (reason: {})", should_restore, reason);

        if !should_restore {
            return Ok(false);
        }

        // Create database directory if it doesn't exist
        match tokio::fs::create_dir_all(&self.db_path).await {
            Ok(_) => {
                info!("Database directory created/verified");
            }
            Err(err) => {
                error!("Failed to create database directory: {}", err);
                return Err(err.into());
            }
        };

        if let Some(export_path) = &backup_stats.export_path {
            if let Ok(db) = self.get_db_handle().await {
                let export_path = export_path.clone();
                tokio::task::spawn_blocking(move || {
                    clear_sled_db(&db)?;
                    import_db_from_file(&db, &export_path)?;
                    db.flush()?;
                    Ok::<(), anyhow::Error>(())
                })
                .await??;
            } else {
                self.restore_from_path_inner(&backup_path, true).await?;
            }
        } else if self.get_db_handle().await.is_ok() {
            warn!("Latest backup is legacy; cannot restore into an open database");
            return Ok(false);
        } else {
            self.restore_from_path_inner(&backup_path, true).await?;
        }
        info!(
            "Database restored successfully from backup: {:?}",
            backup_path
        );
        Ok(true)
    }

    /// Restore database from a specific backup directory. If `force` is true,
    /// the current DB directory is moved aside (or removed) before restore.
    pub async fn restore_from_path(&self, backup_path: &Path, force: bool) -> Result<()> {
        let _guard = self.lock.lock().await;
        self.restore_from_path_inner(backup_path, force).await
    }

    async fn restore_from_path_inner(&self, backup_path: &Path, force: bool) -> Result<()> {
        if !backup_path.exists() {
            anyhow::bail!("Backup path does not exist: {:?}", backup_path);
        }

        if force && self.db_path.exists() {
            // Try to move corrupt DB aside, otherwise remove it
            let mut attempts = 0;
            loop {
                attempts += 1;
                let ts = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
                let mut backup_target = self.db_path.clone();
                backup_target.set_extension(format!("corrupt_{}", ts));

                match std::fs::rename(&self.db_path, &backup_target) {
                    Ok(_) => {
                        info!("Moved corrupt DB to {:?}", backup_target);
                        break;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to move existing DB to {:?}: {} (attempt {})",
                            backup_target, e, attempts
                        );
                        // After 1 attempt to move, just remove it
                        if attempts >= 1 {
                            match std::fs::remove_dir_all(&self.db_path) {
                                Ok(_) => {
                                    info!("Removed corrupt DB directory");
                                    break;
                                }
                                Err(remove_err) => {
                                    anyhow::bail!(
                                        "Failed to remove corrupt DB after failed rename: {}",
                                        remove_err
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        tokio::fs::create_dir_all(&self.db_path).await.ok();

        let export_path = export_file_path(backup_path);
        if export_path.exists() {
            let db_path = self.db_path.clone();
            tokio::task::spawn_blocking(move || {
                let db = sled::open(&db_path)?;
                import_db_from_file(&db, &export_path)?;
                db.flush()?;
                Ok::<(), anyhow::Error>(())
            })
            .await??;
        } else {
            let legacy_db = backup_path.join("db");
            if !legacy_db.exists() {
                anyhow::bail!(
                    "Backup missing export file and legacy db file: {:?}",
                    backup_path
                );
            }
            if std::fs::metadata(&legacy_db).map(|m| m.len()).unwrap_or(0) == 0 {
                anyhow::bail!("Legacy backup db file is empty: {:?}", backup_path);
            }
            let db_path = self.db_path.clone();
            let src = backup_path.to_path_buf();
            tokio::task::spawn_blocking(move || copy_dir_recursive_sync(&src, &db_path)).await??;
        }
        Ok(())
    }

    /// Try each backup (newest first) until one restores successfully.
    /// Returns true if a backup was copied into place.
    pub async fn restore_from_any_backup(&self) -> Result<bool> {
        let _guard = self.lock.lock().await;
        let backups = self.list_backups_desc().await?;
        if backups.is_empty() {
            info!("No backups available to attempt restore");
            return Ok(false);
        }
        info!(
            "Attempting restore from {} backups (newest first)",
            backups.len()
        );
        for path in backups {
            info!("Trying backup {:?}", path);
            match self.restore_from_path_inner(&path, true).await {
                Ok(_) => {
                    info!("Restore from {:?} completed", path);
                    return Ok(true);
                }
                Err(e) => {
                    warn!("Restore from {:?} failed: {}", path, e);
                    continue;
                }
            }
        }
        Ok(false)
    }

    async fn db_stats(&self) -> Result<DbStats> {
        let db_path = self.db_path.clone();
        let db_handle = self.db_handle.lock().await.clone();
        tokio::task::spawn_blocking(move || {
            let exists = db_path.exists();
            let db_file = db_path.join("db");
            let conf_file = db_path.join("conf");
            let mut file_size = 0;
            let mut modified = None;

            if let Ok(meta) = std::fs::metadata(&db_file) {
                file_size = meta.len();
                modified = meta
                    .modified()
                    .ok()
                    .map(|m| chrono::DateTime::<Utc>::from(m));
            }
            let ready = conf_file.exists() && file_size > 0;
            let record_count = match db_handle {
                Some(db) => Some(count_db_records(&db)?),
                None => None,
            };

            Ok::<DbStats, anyhow::Error>(DbStats {
                exists,
                ready,
                file_size,
                modified,
                record_count,
            })
        })
        .await?
    }

    async fn backup_stats(&self, backup_path: &Path) -> Result<BackupStats> {
        let backup_path = backup_path.to_path_buf();
        let name_template = self.name_template.clone();
        tokio::task::spawn_blocking(move || {
            let export_path = export_file_path(&backup_path);
            let (size_bytes, record_count) = if export_path.exists() {
                (
                    std::fs::metadata(&export_path)
                        .map(|m| m.len())
                        .unwrap_or(0),
                    Some(validate_export_file(&export_path)?),
                )
            } else {
                let legacy_db = backup_path.join("db");
                (
                    std::fs::metadata(&legacy_db).map(|m| m.len()).unwrap_or(0),
                    None,
                )
            };

            let modified = backup_timestamp_from_name(&backup_path, &name_template)
                .or_else(|| backup_dir_modified(&backup_path));

            Ok::<BackupStats, anyhow::Error>(BackupStats {
                export_path: if export_path.exists() {
                    Some(export_path)
                } else {
                    None
                },
                size_bytes,
                modified,
                record_count,
            })
        })
        .await?
    }
}

fn copy_dir_recursive_sync(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    for entry_res in std::fs::read_dir(src)? {
        let entry = entry_res?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_recursive_sync(&src_path, &dst_path)?;
        } else if ty.is_file() {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn ensure_empty_dir(dir: &Path, force: bool) -> Result<()> {
    if dir.exists() {
        if !dir.is_dir() {
            anyhow::bail!("Backup output is not a directory: {:?}", dir);
        }
        let mut entries = std::fs::read_dir(dir)?;
        if entries.next().is_some() {
            if force {
                std::fs::remove_dir_all(dir)?;
            } else {
                anyhow::bail!("Backup output directory is not empty: {:?}", dir);
            }
        }
    }
    std::fs::create_dir_all(dir)?;
    Ok(())
}

fn count_db_records(db: &Db) -> Result<usize> {
    let mut count = 0usize;
    for (_collection_type, _collection_name, iter) in db.export() {
        for _ in iter {
            count += 1;
        }
    }
    Ok(count)
}

fn clear_sled_db(db: &Db) -> Result<()> {
    for name in db.tree_names() {
        let tree = db.open_tree(name)?;
        tree.clear()?;
    }
    db.flush()?;
    Ok(())
}

fn backup_timestamp_from_name(
    backup_path: &Path,
    name_template: &str,
) -> Option<chrono::DateTime<Utc>> {
    let name = backup_path.file_name()?.to_string_lossy();
    let mut parts = name_template.split("{{timestamp}}");
    let prefix = parts.next().unwrap_or("");
    let suffix = parts.next().unwrap_or("");
    if !name.starts_with(prefix) || !name.ends_with(suffix) {
        return None;
    }
    let ts = &name[prefix.len()..name.len().saturating_sub(suffix.len())];
    chrono::DateTime::parse_from_str(ts, "%Y%m%dT%H%M%SZ")
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn backup_dir_modified(path: &Path) -> Option<chrono::DateTime<Utc>> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|m| chrono::DateTime::<Utc>::from(m))
}

fn should_restore_from_backup(db: &DbStats, backup: &BackupStats) -> (bool, String) {
    if !db.exists {
        return (true, "database directory missing".to_string());
    }
    if !db.ready {
        return (
            true,
            "database not ready (missing conf/db or zero size)".to_string(),
        );
    }
    if matches!(db.record_count, Some(0)) {
        return (true, "database has zero records".to_string());
    }
    if let (Some(db_records), Some(backup_records)) = (db.record_count, backup.record_count) {
        if db_records > 0
            && backup_records >= db_records.saturating_mul(RESTORE_RECORD_RATIO_THRESHOLD)
            && backup_records >= MIN_BACKUP_RECORDS_FOR_RATIO
        {
            return (
                true,
                format!(
                    "backup records ({}) are >= {}x database records ({})",
                    backup_records, RESTORE_RECORD_RATIO_THRESHOLD, db_records
                ),
            );
        }
    }
    if db.file_size > 0
        && backup.size_bytes >= db.file_size.saturating_mul(RESTORE_SIZE_RATIO_THRESHOLD)
        && backup.size_bytes >= MIN_BACKUP_SIZE_BYTES_FOR_RATIO
    {
        return (
            true,
            format!(
                "backup size ({}) is >= {}x database size ({})",
                backup.size_bytes, RESTORE_SIZE_RATIO_THRESHOLD, db.file_size
            ),
        );
    }
    if let (Some(backup_ts), Some(db_ts)) = (backup.modified, db.modified) {
        if backup_ts > db_ts && db.file_size < SMALL_DB_SIZE_BYTES {
            return (
                true,
                "backup is newer and database size is small".to_string(),
            );
        }
    }
    (
        false,
        "database appears healthy; skipping restore".to_string(),
    )
}

pub fn validate_backup_dir(path: &Path) -> Result<BackupValidation> {
    let export_path = export_file_path(path);
    if export_path.exists() {
        let record_count = validate_export_file(&export_path)?;
        return Ok(BackupValidation::Export { record_count });
    }

    let db_path = path.join("db");
    let conf_path = path.join("conf");
    if !conf_path.exists() {
        anyhow::bail!("Legacy backup missing conf file: {:?}", conf_path);
    }
    let db_meta = std::fs::metadata(&db_path)
        .with_context(|| format!("Legacy backup missing db file: {:?}", db_path))?;
    if db_meta.len() == 0 {
        anyhow::bail!("Legacy backup db file is empty: {:?}", db_path);
    }
    Ok(BackupValidation::Legacy {
        db_size: db_meta.len(),
    })
}

fn export_file_path(dir: &Path) -> PathBuf {
    dir.join(EXPORT_FILE_NAME)
}

fn is_db_ready_for_backup(db_path: &Path) -> Result<bool> {
    let db_file = db_path.join("db");
    let conf_file = db_path.join("conf");
    let db_meta = match std::fs::metadata(&db_file) {
        Ok(md) => md,
        Err(_) => return Ok(false),
    };
    if !conf_file.exists() {
        return Ok(false);
    }
    if db_meta.len() == 0 {
        return Ok(false);
    }
    Ok(true)
}

fn export_db_to_file(db: &Db, export_path: &Path) -> Result<()> {
    let mut writer = io::BufWriter::new(std::fs::File::create(export_path)?);
    writer.write_all(EXPORT_MAGIC)?;
    writer.write_all(&EXPORT_VERSION.to_le_bytes())?;

    for (collection_type, collection_name, iter) in db.export() {
        for kv in iter {
            let mut kv_iter = kv.into_iter();
            let key = kv_iter.next().context("export record missing key")?;
            let value = kv_iter.next().context("export record missing value")?;
            write_bytes(&mut writer, &collection_type)?;
            write_bytes(&mut writer, &collection_name)?;
            write_bytes(&mut writer, &key)?;
            write_bytes(&mut writer, &value)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn validate_export_file(export_path: &Path) -> Result<usize> {
    let mut reader = io::BufReader::new(std::fs::File::open(export_path)?);
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if magic != *EXPORT_MAGIC {
        anyhow::bail!("Invalid backup magic header");
    }
    let version = read_u32(&mut reader)?;
    if version != EXPORT_VERSION {
        anyhow::bail!("Unsupported backup version {}", version);
    }

    let mut record_count = 0;
    loop {
        let collection_type_len = match read_u32_opt(&mut reader)? {
            Some(v) => v,
            None => break,
        };
        let collection_type = read_bytes(&mut reader, collection_type_len)?;
        let collection_name_len = read_u32(&mut reader)?;
        let _collection_name = read_bytes(&mut reader, collection_name_len)?;
        let key_len = read_u32(&mut reader)?;
        let _key = read_bytes(&mut reader, key_len)?;
        let value_len = read_u32(&mut reader)?;
        let _value = read_bytes(&mut reader, value_len)?;

        if collection_type != b"tree" {
            anyhow::bail!("Unsupported collection type {:?}", collection_type);
        }

        record_count += 1;
    }

    Ok(record_count)
}

fn import_db_from_file(db: &Db, export_path: &Path) -> Result<()> {
    let mut reader = io::BufReader::new(std::fs::File::open(export_path)?);
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if magic != *EXPORT_MAGIC {
        anyhow::bail!("Invalid backup magic header");
    }
    let version = read_u32(&mut reader)?;
    if version != EXPORT_VERSION {
        anyhow::bail!("Unsupported backup version {}", version);
    }

    let mut trees: HashMap<Vec<u8>, sled::Tree> = HashMap::new();

    loop {
        let collection_type_len = match read_u32_opt(&mut reader)? {
            Some(v) => v,
            None => break,
        };
        let collection_type = read_bytes(&mut reader, collection_type_len)?;
        let collection_name_len = read_u32(&mut reader)?;
        let collection_name = read_bytes(&mut reader, collection_name_len)?;
        let key_len = read_u32(&mut reader)?;
        let key = read_bytes(&mut reader, key_len)?;
        let value_len = read_u32(&mut reader)?;
        let value = read_bytes(&mut reader, value_len)?;

        if collection_type != b"tree" {
            anyhow::bail!("Unsupported collection type {:?}", collection_type);
        }

        let tree = match trees.get(&collection_name) {
            Some(tree) => tree.clone(),
            None => {
                let tree = db.open_tree(&collection_name)?;
                trees.insert(collection_name.clone(), tree.clone());
                tree
            }
        };

        let old = tree.insert(key, value)?;
        if old.is_some() {
            anyhow::bail!("Import is overwriting existing data");
        }
    }

    Ok(())
}

fn write_bytes(writer: &mut impl Write, bytes: &[u8]) -> Result<()> {
    let len = u32::try_from(bytes.len()).context("Backup record exceeds u32 length limit")?;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

fn read_bytes(reader: &mut impl Read, len: u32) -> Result<Vec<u8>> {
    let len = usize::try_from(len).context("Invalid backup record length")?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_u32(reader: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u32_opt(reader: &mut impl Read) -> Result<Option<u32>> {
    let mut buf = [0u8; 4];
    let first = reader.read(&mut buf)?;
    if first == 0 {
        return Ok(None);
    }
    if first < buf.len() {
        reader.read_exact(&mut buf[first..])?;
    }
    Ok(Some(u32::from_le_bytes(buf)))
}
