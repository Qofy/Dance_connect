// ============================================================================
// backend/src/models/error_log_config.rs - ERROR LOG CONFIGURATION MODELS
// ============================================================================
// User configuration for error logging behavior
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LoggingMode {
    Global,     // Log all application requests
    TesterOnly, // Log only Ad-hoc tester requests
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StorageBehavior {
    StoreAllExcept,    // Blacklist mode - store all except those in ignore_rules
    StoreOnlyFromList, // Whitelist mode - store only those matching ignore_rules
    Disabled,          // No logging
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRule {
    pub id: String,
    pub rule_type: String, // "url_pattern", "status_code", "method", "custom"
    pub pattern: String,   // Regex for URL, exact match for others
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLogConfig {
    pub user_id: String,
    pub logging_mode: LoggingMode,
    pub storage_behavior: StorageBehavior,
    pub ignore_rules: Vec<IgnoreRule>,
    pub max_logs_per_user: usize,            // Default: 1000
    pub auto_delete_after_days: Option<u32>, // Default: Some(30)
    pub created_at: String,
    pub updated_at: String,
}

impl Default for ErrorLogConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            user_id: String::new(),
            logging_mode: LoggingMode::TesterOnly,
            storage_behavior: StorageBehavior::Disabled,
            ignore_rules: Vec::new(),
            max_logs_per_user: 1000,
            auto_delete_after_days: Some(30),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl ErrorLogConfig {
    pub fn new(user_id: String) -> Self {
        let mut config = Self::default();
        config.user_id = user_id;
        config
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateErrorLogConfigRequest {
    pub logging_mode: Option<LoggingMode>,
    pub storage_behavior: Option<StorageBehavior>,
    pub ignore_rules: Option<Vec<IgnoreRule>>,
    pub max_logs_per_user: Option<usize>,
    pub auto_delete_after_days: Option<u32>,
}
