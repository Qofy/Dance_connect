// ============================================================================
// backend/src/models/error_log.rs - ERROR LOG DATA MODELS
// ============================================================================
// Track API errors (40x, 50x) for debugging and analysis
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    pub id: String,
    pub user_id: String,
    pub company_id: Option<String>,
    pub method: String,   // GET, POST, PUT, DELETE, PATCH
    pub path: String,     // /api/companies
    pub full_url: String, // http://localhost:8061/api/companies?id=123
    #[serde(default)]
    pub query_params: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub request_body: Option<String>, // JSON payload
    pub response_status: u16,
    pub response_body: String, // Truncated to 10KB
    pub execution_time_ms: Option<i64>,
    pub logged_at: DateTime<Utc>,
    pub error_type: String, // log_400, log_401, log_403, log_409, log_500, log_5xx, etc.
    pub flags: Vec<String>, // ["Bug", "Attack", "Performance Issue", ...]
    pub custom_flag: Option<String>,
    pub notes: Option<String>,
    pub from_tester: bool, // Whether error occurred from Ad-hoc tester
    #[serde(default)]
    pub custom_headers: HashMap<String, String>, // User-added headers
}

impl ErrorLog {
    pub fn new(user_id: String, method: String, path: String, status: u16) -> Self {
        let error_type = match status {
            400 => "log_400".to_string(),
            401 => "log_401".to_string(),
            403 => "log_403".to_string(),
            409 => "log_409".to_string(),
            500 => "log_500".to_string(),
            501 => "log_501".to_string(),
            502 => "log_502".to_string(),
            503 => "log_503".to_string(),
            _ => {
                if status >= 500 {
                    "log_5xx".to_string()
                } else if status >= 400 {
                    "log_4xx".to_string()
                } else {
                    format!("log_{}", status)
                }
            }
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            company_id: None,
            method,
            path,
            full_url: String::new(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            request_body: None,
            response_status: status,
            response_body: String::new(),
            execution_time_ms: None,
            logged_at: Utc::now(),
            error_type,
            flags: Vec::new(),
            custom_flag: None,
            notes: None,
            from_tester: false,
            custom_headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateErrorLogRequest {
    pub method: String,
    pub path: String,
    pub full_url: String,
    #[serde(default)]
    pub query_params: Option<HashMap<String, String>>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_body: String,
    pub execution_time_ms: Option<i64>,
    #[serde(default)]
    pub company_id: Option<String>,
    #[serde(default)]
    pub custom_headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub from_tester: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorLogSummary {
    pub id: String,
    pub method: String,
    pub path: String,
    pub response_status: u16,
    pub error_type: String,
    pub logged_at: String,
    pub flags: Vec<String>,
    pub custom_flag: Option<String>,
    pub from_tester: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorLogListResponse {
    pub items: Vec<ErrorLogSummary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorLogDetailResponse {
    pub id: String,
    pub method: String,
    pub path: String,
    pub full_url: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_body: String,
    pub execution_time_ms: Option<i64>,
    pub logged_at: String,
    pub error_type: String,
    pub flags: Vec<String>,
    pub custom_flag: Option<String>,
    pub notes: Option<String>,
    pub from_tester: bool,
    pub company_id: Option<String>,
    pub custom_headers: HashMap<String, String>,
}

impl From<ErrorLog> for ErrorLogDetailResponse {
    fn from(log: ErrorLog) -> Self {
        Self {
            id: log.id,
            method: log.method,
            path: log.path,
            full_url: log.full_url,
            query_params: log.query_params,
            headers: log.headers,
            request_body: log.request_body,
            response_status: log.response_status,
            response_body: log.response_body,
            execution_time_ms: log.execution_time_ms,
            logged_at: log.logged_at.to_rfc3339(),
            error_type: log.error_type,
            flags: log.flags,
            custom_flag: log.custom_flag,
            notes: log.notes,
            from_tester: log.from_tester,
            company_id: log.company_id,
            custom_headers: log.custom_headers,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateFlagsRequest {
    pub flags: Vec<String>,
    pub custom_flag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkDeleteRequest {
    pub ids: Vec<String>,
}
