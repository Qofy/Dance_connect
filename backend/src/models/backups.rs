use serde::{Deserialize, Serialize};

fn now_iso() -> String {
    crate::time::now()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupRecord {
    pub id: String,
    pub created_at: String,
    pub created_by: String,
    /// manual | scheduled | upload
    pub kind: String,
    pub filename: String,
    pub size_bytes: u64,
    pub include_uploads: bool,
    pub format_version: u32,
    pub last_updated: String,
}

impl BackupRecord {
    pub fn new(
        id: String,
        created_by: String,
        kind: String,
        filename: String,
        size_bytes: u64,
        include_uploads: bool,
        format_version: u32,
    ) -> Self {
        let now = now_iso();
        Self {
            id,
            created_at: now.clone(),
            created_by,
            kind,
            filename,
            size_bytes,
            include_uploads,
            format_version,
            last_updated: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_updated = now_iso();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_interval_minutes")]
    pub interval_minutes: u64,
    #[serde(default = "default_retention_count")]
    pub retention_count: usize,
    #[serde(default)]
    pub include_uploads: bool,
    /// sha256(password) sent by the frontend (never store plaintext)
    #[serde(default)]
    pub encryption_password_hash: Option<String>,
    #[serde(default)]
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub last_updated: String,
}

fn default_interval_minutes() -> u64 {
    60
}

fn default_retention_count() -> usize {
    10
}

impl Default for BackupSettings {
    fn default() -> Self {
        let now = now_iso();
        Self {
            enabled: false,
            interval_minutes: default_interval_minutes(),
            retention_count: default_retention_count(),
            include_uploads: true,
            encryption_password_hash: None,
            last_run_at: None,
            created_at: now.clone(),
            last_updated: now,
        }
    }
}

impl BackupSettings {
    pub fn touch(&mut self) {
        self.last_updated = now_iso();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CreateBackupRequest {
    pub password_hash: String,
    #[serde(default)]
    pub include_uploads: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RestoreBackupRequest {
    pub password_hash: String,
    /// safety flag to avoid accidental restores
    #[serde(default)]
    pub confirm: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct UpdateBackupSettingsRequest {
    pub enabled: Option<bool>,
    pub interval_minutes: Option<u64>,
    pub retention_count: Option<usize>,
    pub include_uploads: Option<bool>,
    /// sha256(password) sent by the frontend (never store plaintext)
    pub encryption_password_hash: Option<String>,
}
