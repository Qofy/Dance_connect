// src/handlers/dsr.rs
// Data Subject Rights endpoints: export and delete/anonymize with retention guard and audit logging.
use crate::authz::require_claims;
use crate::handlers::errors::ApiError;
use crate::models::UserRecord;
use crate::time::now;
use crate::{db::Database, models::Customer};
use actix_web::{get, post, web, HttpResponse, Result};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

const RETENTION_YEARS: i64 = 10;

#[derive(Debug, Serialize)]
struct DsrExport {
    user: Option<UserRecord>,
    companies: serde_json::Value,
    customers: serde_json::Value,
    quotes: serde_json::Value,
    invoices: serde_json::Value,
    generated_at: String,
}

#[derive(Debug, Serialize)]
struct AuditLogEntry {
    id: String,
    action: String,
    user_id: String,
    company_id: Option<String>,
    details: serde_json::Value,
    created_at: String,
}

fn company_from_header(req: &actix_web::HttpRequest) -> Option<String> {
    req.headers()
        .get("X-Company-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn write_audit(db: &Database, entry: AuditLogEntry) {
    let _ = db.insert("audit_logs", &entry.id, &entry);
}

fn parse_iso(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

fn retention_blocked(last_updated: &str) -> bool {
    if let Some(dt) = parse_iso(last_updated) {
        dt > Utc::now() - Duration::days(RETENTION_YEARS * 365)
    } else {
        false
    }
}

fn guard_retention_for_company(db: &Database, company_id: &str) -> Result<(), HttpResponse> {
    if let Ok(quotes) = db.list::<serde_json::Value>("quotes") {
        for q in quotes {
            let cid = q
                .get("company_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let lu = q
                .get("last_updated")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if cid == company_id && retention_blocked(lu) {
                return Err(HttpResponse::Forbidden().json(ApiError {
                    status: 403,
                    code: "RETENTION_LOCK",
                    message: format!(
                        "Cannot delete/anonymize data: quote {} is within retention window",
                        q.get("id").and_then(|v| v.as_str()).unwrap_or("")
                    ),
                    errors: None,
                    timestamp: now(),
                    trace: None,
                }));
            }
        }
    }
    if let Ok(inv) = db.list::<serde_json::Value>("invoices") {
        for i in inv {
            let cid = i
                .get("company_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let lu = i
                .get("last_updated")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if cid == company_id && retention_blocked(lu) {
                return Err(HttpResponse::Forbidden().json(ApiError {
                    status: 403,
                    code: "RETENTION_LOCK",
                    message: format!(
                        "Cannot delete/anonymize data: invoice {} is within retention window",
                        i.get("id").and_then(|v| v.as_str()).unwrap_or("")
                    ),
                    errors: None,
                    timestamp: now(),
                    trace: None,
                }));
            }
        }
    }
    Ok(())
}

/// #feature[compliance]
#[get("/dsr/export")]
pub async fn export(req: actix_web::HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let claims = match require_claims(&req) {
        Ok(c) => c,
        Err(response) => return Ok(response),
    };
    let company_id = company_from_header(&req);

    let user = db.get::<UserRecord>("users", &claims.sub).ok().flatten();
    let companies: serde_json::Value = db
        .list::<serde_json::Value>("companies")
        .unwrap_or_default()
        .into_iter()
        .filter(|c| {
            c.get("user_id")
                .and_then(|v| v.as_str())
                .map(|id| id == claims.sub)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>()
        .into();
    let customers: serde_json::Value = db
        .list::<serde_json::Value>("customers")
        .unwrap_or_default()
        .into_iter()
        .filter(|c| {
            if let Some(cid) = &company_id {
                c.get("company_id")
                    .and_then(|v| v.as_str())
                    .map(|id| id == cid)
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .into();
    let quotes: serde_json::Value = db
        .list::<serde_json::Value>("quotes")
        .unwrap_or_default()
        .into_iter()
        .filter(|q| {
            if let Some(cid) = &company_id {
                q.get("company_id")
                    .and_then(|v| v.as_str())
                    .map(|id| id == cid)
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .into();
    let invoices: serde_json::Value = db
        .list::<serde_json::Value>("invoices")
        .unwrap_or_default()
        .into_iter()
        .filter(|i| {
            if let Some(cid) = &company_id {
                i.get("company_id")
                    .and_then(|v| v.as_str())
                    .map(|id| id == cid)
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .into();

    let snapshot = DsrExport {
        user,
        companies,
        customers,
        quotes,
        invoices,
        generated_at: now(),
    };

    write_audit(
        &db,
        AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            action: "DSR_EXPORT".into(),
            user_id: claims.sub.clone(),
            company_id: company_id.clone(),
            details: json!({ "company_id": company_id }),
            created_at: now(),
        },
    );

    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .json(snapshot))
}

/// #feature[compliance]
#[post("/dsr/delete")]
pub async fn delete(req: actix_web::HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let claims = match require_claims(&req) {
        Ok(c) => c,
        Err(response) => return Ok(response),
    };
    let company_id = match company_from_header(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::BadRequest().json(ApiError {
                status: 400,
                code: "MISSING_COMPANY",
                message: "X-Company-Id header is required for DSR delete".into(),
                errors: None,
                timestamp: now(),
                trace: None,
            }))
        }
    };

    // Retention guard
    if let Err(err) = guard_retention_for_company(&db, &company_id) {
        return Ok(err);
    }

    // Anonymize customers for this company
    if let Ok(customers) = db.list::<Customer>("customers") {
        for mut c in customers.into_iter().filter(|c| c.company_id == company_id) {
            c.name = "anonymized".into();
            c.email = "anonymized@example.com".into();
            c.phone = "anonymized".into();
            c.contact_person = Some("anonymized".into());
            c.notes = None;
            c.last_updated = now();
            let _ = db.insert("customers", &c.id, &c);
        }
    }

    write_audit(
        &db,
        AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            action: "DSR_DELETE".into(),
            user_id: claims.sub.clone(),
            company_id: Some(company_id.clone()),
            details: json!({ "company_id": company_id }),
            created_at: now(),
        },
    );

    Ok(HttpResponse::Ok().json(json!({"status": "ok"})))
}
