// ============================================================================
// backend/src/handlers/error_log.rs - ERROR LOG HANDLERS
// ============================================================================
// API endpoints for saving, retrieving, and managing error logs
// ============================================================================

use crate::authz::require_user_id;
use crate::db::Database;
use crate::models::{
    BulkDeleteRequest, CreateErrorLogRequest, ErrorLog, ErrorLogConfig, ErrorLogDetailResponse,
    ErrorLogListResponse, ErrorLogSummary, UpdateErrorLogConfigRequest, UpdateFlagsRequest,
};
use actix_web::{delete, get, patch, post, put, web, HttpRequest, HttpResponse, Result};
use serde_json::json;

// ============================================================================
// ERROR LOG CRUD ENDPOINTS
// ============================================================================

/// POST /error-logs
/// #feature[system]
#[post("/error-logs")]
pub async fn create_error_log(
    req: HttpRequest,
    db: web::Data<Database>,
    payload: web::Json<CreateErrorLogRequest>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };
    let mut log = ErrorLog::new(
        user_id.clone(),
        payload.method.clone(),
        payload.path.clone(),
        payload.response_status,
    );

    log.full_url = payload.full_url.clone();
    log.query_params = payload.query_params.clone().unwrap_or_default();
    log.headers = payload.headers.clone().unwrap_or_default();
    log.request_body = payload.request_body.clone();
    log.response_body = payload.response_body.clone();
    log.execution_time_ms = payload.execution_time_ms;
    log.company_id = payload.company_id.clone();
    log.custom_headers = payload.custom_headers.clone().unwrap_or_default();
    log.from_tester = payload.from_tester.unwrap_or(false);

    // Store in database
    let id = log.id.clone();
    if let Err(err) = db
        .insert("error_logs", &id, &log)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to save error log"))
    {
        return Err(err);
    }

    // Add to user's error log index (most recent first, keep last 1000)
    let index_key = format!("user:{}", user_id);
    let mut user_logs: Vec<String> = db
        .get("error_log_index", &index_key)
        .ok()
        .flatten()
        .unwrap_or_default();

    user_logs.insert(0, id.clone());
    user_logs.truncate(1000); // Keep only last 1000 entries

    if let Err(err) = db
        .insert("error_log_index", &index_key, &user_logs)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to save index"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Created().json(json!({
        "id": id,
        "status": "logged",
        "message": "Error log saved successfully"
    })))
}

/// GET /error-logs
/// #feature[system]
#[get("/error-logs")]
pub async fn list_error_logs(
    req: HttpRequest,
    db: web::Data<Database>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };

    let limit = query
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50)
        .min(200);

    let offset = query
        .get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .unwrap_or(0);

    // Optional filters
    let status_filter = query.get("status").and_then(|s| s.parse::<u16>().ok());
    let from_tester_filter = query
        .get("from_tester")
        .and_then(|f| f.parse::<bool>().ok());

    // Get user's error log index
    let index_key = format!("user:{}", user_id);
    let user_logs: Vec<String> = db
        .get("error_log_index", &index_key)
        .ok()
        .flatten()
        .unwrap_or_default();

    let total = user_logs.len() as i64;

    // Fetch paginated items
    let items: Vec<ErrorLogSummary> = user_logs
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
        .filter_map(|id| {
            db.get::<ErrorLog>("error_logs", id)
                .ok()
                .flatten()
                .map(|log| ErrorLogSummary {
                    id: log.id,
                    method: log.method,
                    path: log.path,
                    response_status: log.response_status,
                    error_type: log.error_type,
                    logged_at: log.logged_at.to_rfc3339(),
                    flags: log.flags,
                    custom_flag: log.custom_flag,
                    from_tester: log.from_tester,
                })
        })
        .filter(|log| {
            if let Some(status) = status_filter {
                if log.response_status != status {
                    return false;
                }
            }
            if let Some(from_tester) = from_tester_filter {
                if log.from_tester != from_tester {
                    return false;
                }
            }
            true
        })
        .collect();

    Ok(HttpResponse::Ok().json(ErrorLogListResponse {
        items,
        total,
        limit,
        offset,
    }))
}

/// GET /error-logs/{id}
/// #feature[system]
#[get("/error-logs/{id}")]
pub async fn get_error_log(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };
    let id = path.into_inner();

    let log: ErrorLog = match db.get("error_logs", &id).ok().flatten() {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorNotFound("Error log not found")),
    };

    // Verify ownership
    if log.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    Ok(HttpResponse::Ok().json(ErrorLogDetailResponse::from(log)))
}

/// DELETE /error-logs/{id}
/// #feature[system]
#[delete("/error-logs/{id}")]
pub async fn delete_error_log(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };
    let id = path.into_inner();

    // Verify ownership before deleting
    let log: ErrorLog = match db.get("error_logs", &id).ok().flatten() {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorNotFound("Error log not found")),
    };

    if log.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    // Remove from storage
    let _ = db.delete("error_logs", &id);

    // Remove from index
    let index_key = format!("user:{}", user_id);
    let mut user_logs: Vec<String> = db
        .get("error_log_index", &index_key)
        .ok()
        .flatten()
        .unwrap_or_default();
    user_logs.retain(|lid| lid != &id);
    if let Err(err) = db
        .insert("error_log_index", &index_key, &user_logs)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to update index"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "deleted",
        "message": "Error log deleted successfully"
    })))
}

/// POST /error-logs/bulk-delete
/// #feature[system]
#[post("/error-logs/bulk-delete")]
pub async fn bulk_delete_error_logs(
    req: HttpRequest,
    db: web::Data<Database>,
    payload: web::Json<BulkDeleteRequest>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };
    let mut deleted_count = 0;

    for id in &payload.ids {
        // Verify ownership
        if let Ok(Some(log)) = db.get::<ErrorLog>("error_logs", id) {
            if log.user_id == user_id {
                let _ = db.delete("error_logs", id);
                deleted_count += 1;
            }
        }
    }

    // Remove from index
    let index_key = format!("user:{}", user_id);
    let mut user_logs: Vec<String> = db
        .get("error_log_index", &index_key)
        .ok()
        .flatten()
        .unwrap_or_default();
    user_logs.retain(|id| !payload.ids.contains(id));
    if let Err(err) = db
        .insert("error_log_index", &index_key, &user_logs)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to update index"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(json!({
        "deleted_count": deleted_count,
        "status": "completed"
    })))
}

/// DELETE /error-logs/clear-all
/// #feature[system]
#[delete("/error-logs/clear-all")]
pub async fn clear_all_error_logs(
    req: HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };

    // Get all log IDs
    let index_key = format!("user:{}", user_id);
    let user_logs: Vec<String> = db
        .get("error_log_index", &index_key)
        .ok()
        .flatten()
        .unwrap_or_default();

    let deleted_count = user_logs.len();

    // Delete all logs
    for id in &user_logs {
        let _ = db.delete("error_logs", id);
    }

    // Clear the index
    let _ = db.delete("error_log_index", &index_key);

    Ok(HttpResponse::Ok().json(json!({
        "deleted_count": deleted_count,
        "status": "cleared",
        "message": "All error logs cleared successfully"
    })))
}

/// PATCH /error-logs/{id}/flags
/// #feature[system]
#[patch("/error-logs/{id}/flags")]
pub async fn update_error_log_flags(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    payload: web::Json<UpdateFlagsRequest>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };
    let id = path.into_inner();

    let mut log: ErrorLog = match db.get("error_logs", &id).ok().flatten() {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorNotFound("Error log not found")),
    };

    // Verify ownership
    if log.user_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }

    // Update the flags
    log.flags = payload.flags.clone();
    log.custom_flag = payload.custom_flag.clone();

    // Save back to database
    if let Err(err) = db
        .insert("error_logs", &id, &log)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to update flags"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "updated",
        "id": id,
        "flags": log.flags,
        "custom_flag": log.custom_flag,
        "message": "Flags updated successfully"
    })))
}

// ============================================================================
// ERROR LOG CONFIGURATION ENDPOINTS
// ============================================================================

/// GET /error-logging/config
/// #feature[system]
#[get("/error-logging/config")]
pub async fn get_error_log_config(
    req: HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };

    let config_key = format!("user:{}", user_id);
    let config: ErrorLogConfig = db
        .get("error_log_config", &config_key)
        .ok()
        .flatten()
        .unwrap_or_else(|| ErrorLogConfig::new(user_id.clone()));

    Ok(HttpResponse::Ok().json(config))
}

/// PUT /error-logging/config
/// #feature[system]
#[put("/error-logging/config")]
pub async fn update_error_log_config(
    req: HttpRequest,
    db: web::Data<Database>,
    payload: web::Json<UpdateErrorLogConfigRequest>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };

    let config_key = format!("user:{}", user_id);
    let mut config: ErrorLogConfig = db
        .get("error_log_config", &config_key)
        .ok()
        .flatten()
        .unwrap_or_else(|| ErrorLogConfig::new(user_id.clone()));

    // Update fields if provided
    if let Some(mode) = &payload.logging_mode {
        config.logging_mode = mode.clone();
    }
    if let Some(behavior) = &payload.storage_behavior {
        config.storage_behavior = behavior.clone();
    }
    if let Some(rules) = &payload.ignore_rules {
        config.ignore_rules = rules.clone();
    }
    if let Some(max) = payload.max_logs_per_user {
        config.max_logs_per_user = max;
    }
    if let Some(days) = payload.auto_delete_after_days {
        config.auto_delete_after_days = Some(days);
    }

    config.updated_at = chrono::Utc::now().to_rfc3339();

    if let Err(err) = db
        .insert("error_log_config", &config_key, &config)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to save config"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(config))
}

/// POST /error-logging/config/reset
/// #feature[system]
#[post("/error-logging/config/reset")]
pub async fn reset_error_log_config(
    req: HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let user_id = match require_user_id(&req) {
        Ok(id) => id,
        Err(response) => return Ok(response),
    };

    let config_key = format!("user:{}", user_id);
    let config = ErrorLogConfig::new(user_id.clone());

    if let Err(err) = db
        .insert("error_log_config", &config_key, &config)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to reset config"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(config))
}
