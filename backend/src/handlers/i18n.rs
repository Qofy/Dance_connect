use actix_web::{get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::authz::{require_claims, require_manage_system_settings};
use crate::db::Database;
use crate::handlers::errors::ApiError;
use crate::models::auth_types::Claims;
use crate::models::i18n::{MissingTranslationStat, TranslationOverrides};
use crate::models::{Invoice, InvoiceShare, Quote};
use crate::time::now;
use crate::ui_languages::normalize_ui_language;

const I18N_SYSTEM_TREE: &str = "i18n_system";
const I18N_USER_TREE: &str = "i18n_user";
const I18N_MISSING_TREE: &str = "i18n_missing";
const INVOICE_SHARES_TREE: &str = "invoice_shares";

const MAX_KEY_LEN: usize = 200;
const MAX_VALUE_LEN: usize = 2000;

#[derive(Debug, Deserialize)]
pub struct I18nQuery {
    pub token: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTranslationRequest {
    pub entries: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissingTranslationRequest {
    pub language: String,
    pub key: String,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MissingQuery {
    pub language: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolvedTranslations {
    pub language: String,
    pub entries: HashMap<String, String>,
    pub system_count: usize,
    pub user_count: usize,
}

fn normalize_lang_or_error(raw: &str) -> Result<String, HttpResponse> {
    match normalize_ui_language(raw) {
        Some(value) => Ok(value),
        None => {
            let mut errors = HashMap::new();
            errors.insert(
                "language".to_string(),
                "Invalid language code (supported UI language expected)".to_string(),
            );
            Err(HttpResponse::UnprocessableEntity()
                .json(ApiError::validation("Validation failed".into(), errors)))
        }
    }
}

fn validate_entries(entries: &HashMap<String, String>) -> Result<(), HttpResponse> {
    let mut errors: HashMap<String, String> = HashMap::new();

    for (key, value) in entries {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            errors.insert(
                "entries".to_string(),
                "translation key cannot be empty".to_string(),
            );
            continue;
        }
        if trimmed.len() > MAX_KEY_LEN {
            errors.insert(
                format!("entries.{trimmed}"),
                format!("translation key exceeds {MAX_KEY_LEN} characters"),
            );
        }
        if value.len() > MAX_VALUE_LEN {
            errors.insert(
                format!("entries.{trimmed}"),
                format!("translation value exceeds {MAX_VALUE_LEN} characters"),
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HttpResponse::UnprocessableEntity()
            .json(ApiError::validation("Validation failed".into(), errors)))
    }
}

fn build_user_key(user_id: &str, language: &str) -> String {
    format!("{user_id}:{language}")
}

fn load_overrides(
    db: &Database,
    tree: &str,
    key: &str,
) -> Result<Option<TranslationOverrides>, HttpResponse> {
    match db.get::<TranslationOverrides>(tree, key) {
        Ok(value) => Ok(value),
        Err(_) => Err(HttpResponse::InternalServerError().json(ApiError::internal(
            "DB_GET_FAILED",
            "Failed to load translations",
            false,
        ))),
    }
}

fn save_overrides(
    db: &Database,
    tree: &str,
    key: &str,
    overrides: &TranslationOverrides,
) -> Result<(), HttpResponse> {
    match db.insert(tree, key, overrides) {
        Ok(()) => Ok(()),
        Err(_) => Err(HttpResponse::InternalServerError().json(ApiError::internal(
            "DB_SAVE_FAILED",
            "Failed to save translations",
            false,
        ))),
    }
}

fn resolve_user_id_from_token(
    db: &Database,
    token: &str,
    kind: Option<&str>,
) -> Result<Option<String>, HttpResponse> {
    let token = token.trim();
    if token.is_empty() {
        return Ok(None);
    }

    let kind_value = kind.unwrap_or("").to_lowercase();
    let is_share_token = token.starts_with("share_");

    if kind_value == "invoice_share" || (kind_value.is_empty() && is_share_token) {
        let shares = match db.list::<InvoiceShare>(INVOICE_SHARES_TREE) {
            Ok(list) => list,
            Err(_) => {
                return Err(HttpResponse::InternalServerError().json(ApiError::internal(
                    "DB_LIST_FAILED",
                    "Failed to load invoice shares",
                    false,
                )))
            }
        };
        if let Some(share) = shares.into_iter().find(|s| s.share_token == token) {
            if share.is_valid() {
                return Ok(Some(share.user_id));
            }
        }
    }

    if kind_value == "invoice" || (kind_value.is_empty() && !is_share_token) {
        match db.get::<Invoice>("invoices", token) {
            Ok(Some(invoice)) => {
                if invoice.archived {
                    return Ok(None);
                }
                return Ok(Some(invoice.user_id));
            }
            Ok(None) => {}
            Err(_) => {
                return Err(HttpResponse::InternalServerError().json(ApiError::internal(
                    "DB_GET_FAILED",
                    "Failed to load invoice",
                    false,
                )))
            }
        }
    }

    if kind_value == "quote" || kind_value.is_empty() {
        let quotes = match db.list::<Quote>("quotes") {
            Ok(list) => list,
            Err(_) => {
                return Err(HttpResponse::InternalServerError().json(ApiError::internal(
                    "DB_LIST_FAILED",
                    "Failed to load quotes",
                    false,
                )))
            }
        };
        if let Some(quote) = quotes.into_iter().find(|q| {
            q.approval_token
                .as_ref()
                .map(|t| t == token)
                .unwrap_or(false)
                || q.id == token
        }) {
            if quote.status == "archived" || quote.converted_to_invoice {
                return Ok(None);
            }
            if !quote.public_view_enabled {
                return Ok(None);
            }
            return Ok(Some(quote.user_id));
        }
    }

    Ok(None)
}

async fn get_i18n_handler(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    query: web::Query<I18nQuery>,
) -> Result<HttpResponse> {
    let normalized = match normalize_lang_or_error(&path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let system_overrides = match load_overrides(&db, I18N_SYSTEM_TREE, &normalized) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let mut user_overrides: Option<TranslationOverrides> = None;
    if let Some(claims) = req.extensions().get::<Claims>().cloned() {
        let user_key = build_user_key(&claims.sub, &normalized);
        match load_overrides(&db, I18N_USER_TREE, &user_key) {
            Ok(value) => user_overrides = value,
            Err(response) => return Ok(response),
        }
    } else if let Some(token) = query.token.as_ref() {
        let resolved = match resolve_user_id_from_token(&db, token, query.kind.as_deref()) {
            Ok(value) => value,
            Err(response) => return Ok(response),
        };
        if let Some(user_id) = resolved {
            let user_key = build_user_key(&user_id, &normalized);
            match load_overrides(&db, I18N_USER_TREE, &user_key) {
                Ok(value) => user_overrides = value,
                Err(response) => return Ok(response),
            }
        }
    }

    let mut entries: HashMap<String, String> = HashMap::new();
    let mut system_count: usize = 0;
    let mut user_count: usize = 0;

    if let Some(system) = system_overrides {
        system_count = system.entries.len();
        for (key, value) in system.entries {
            entries.insert(key, value);
        }
    }

    if let Some(user) = user_overrides {
        user_count = user.entries.len();
        for (key, value) in user.entries {
            entries.insert(key, value);
        }
    }

    Ok(HttpResponse::Ok().json(ResolvedTranslations {
        language: normalized,
        entries,
        system_count,
        user_count,
    }))
}

#[get("/i18n/{lang}")]
pub async fn get_i18n(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    query: web::Query<I18nQuery>,
) -> Result<HttpResponse> {
    get_i18n_handler(req, db, path, query).await
}

#[get("/public/i18n/{lang}")]
pub async fn get_i18n_public(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    query: web::Query<I18nQuery>,
) -> Result<HttpResponse> {
    get_i18n_handler(req, db, path, query).await
}

#[get("/i18n/system/{lang}")]
pub async fn get_system_translations(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let _claims = match require_manage_system_settings(&req) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let normalized = match normalize_lang_or_error(&path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    match load_overrides(&db, I18N_SYSTEM_TREE, &normalized) {
        Ok(Some(value)) => Ok(HttpResponse::Ok().json(value)),
        Ok(None) => Ok(HttpResponse::Ok().json(TranslationOverrides::empty(&normalized, now()))),
        Err(response) => Ok(response),
    }
}

#[put("/i18n/system/{lang}")]
pub async fn update_system_translations(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    payload: web::Json<UpdateTranslationRequest>,
) -> Result<HttpResponse> {
    let _claims = match require_manage_system_settings(&req) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let normalized = match normalize_lang_or_error(&path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    if let Err(response) = validate_entries(&payload.entries) {
        return Ok(response);
    }

    let mut overrides = TranslationOverrides::empty(&normalized, now());
    overrides.entries = payload.entries.clone();
    overrides.updated_by = Some("system".to_string());

    if let Err(response) = save_overrides(&db, I18N_SYSTEM_TREE, &normalized, &overrides) {
        return Ok(response);
    }

    Ok(HttpResponse::Ok().json(overrides))
}

#[get("/i18n/user/{lang}")]
pub async fn get_user_translations(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let claims = match require_claims(&req) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let normalized = match normalize_lang_or_error(&path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let key = build_user_key(&claims.sub, &normalized);
    match load_overrides(&db, I18N_USER_TREE, &key) {
        Ok(Some(value)) => Ok(HttpResponse::Ok().json(value)),
        Ok(None) => Ok(HttpResponse::Ok().json(TranslationOverrides::empty(&normalized, now()))),
        Err(response) => Ok(response),
    }
}

#[put("/i18n/user/{lang}")]
pub async fn update_user_translations(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    payload: web::Json<UpdateTranslationRequest>,
) -> Result<HttpResponse> {
    let claims = match require_claims(&req) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let normalized = match normalize_lang_or_error(&path.into_inner()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    if let Err(response) = validate_entries(&payload.entries) {
        return Ok(response);
    }

    let mut overrides = TranslationOverrides::empty(&normalized, now());
    overrides.entries = payload.entries.clone();
    overrides.updated_by = Some(claims.sub.clone());

    let key = build_user_key(&claims.sub, &normalized);
    if let Err(response) = save_overrides(&db, I18N_USER_TREE, &key, &overrides) {
        return Ok(response);
    }

    Ok(HttpResponse::Ok().json(overrides))
}

#[post("/i18n/missing")]
pub async fn report_missing_translation(
    db: web::Data<Database>,
    payload: web::Json<MissingTranslationRequest>,
) -> Result<HttpResponse> {
    let normalized = match normalize_lang_or_error(&payload.language) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let key = payload.key.trim();
    if key.is_empty() || key.len() > MAX_KEY_LEN {
        let mut errors = HashMap::new();
        errors.insert(
            "key".to_string(),
            format!("translation key must be 1-{MAX_KEY_LEN} characters"),
        );
        return Ok(HttpResponse::UnprocessableEntity()
            .json(ApiError::validation("Validation failed".into(), errors)));
    }
    if let Some(context) = payload.context.as_ref() {
        if context.len() > MAX_VALUE_LEN {
            let mut errors = HashMap::new();
            errors.insert(
                "context".to_string(),
                format!("context exceeds {MAX_VALUE_LEN} characters"),
            );
            return Ok(HttpResponse::UnprocessableEntity()
                .json(ApiError::validation("Validation failed".into(), errors)));
        }
    }

    let record_key = format!("{normalized}:{key}");
    let mut was_existing = false;
    let mut stat = match db.get::<MissingTranslationStat>(I18N_MISSING_TREE, &record_key) {
        Ok(Some(existing)) => {
            was_existing = true;
            existing
        }
        Ok(None) => MissingTranslationStat::new(
            normalized.clone(),
            key.to_string(),
            payload.context.clone(),
            now(),
        ),
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "DB_GET_FAILED",
                "Failed to load missing translation stats",
                false,
            )))
        }
    };

    if was_existing {
        stat.bump(payload.context.clone(), now());
    }

    match db.insert(I18N_MISSING_TREE, &record_key, &stat) {
        Ok(()) => Ok(HttpResponse::Ok().json(stat)),
        Err(_) => Ok(HttpResponse::InternalServerError().json(ApiError::internal(
            "DB_SAVE_FAILED",
            "Failed to store missing translation stats",
            false,
        ))),
    }
}

#[get("/i18n/missing")]
pub async fn list_missing_translations(
    req: HttpRequest,
    db: web::Data<Database>,
    query: web::Query<MissingQuery>,
) -> Result<HttpResponse> {
    let _claims = match require_manage_system_settings(&req) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let language_filter = match query.language.as_ref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("all")
                || trimmed.eq_ignore_ascii_case("undefined")
                || trimmed.eq_ignore_ascii_case("null")
            {
                None
            } else {
                match normalize_ui_language(trimmed) {
                    Some(value) => Some(value),
                    None => {
                        let mut errors = HashMap::new();
                        errors.insert(
                            "language".to_string(),
                            "Invalid language code (supported UI language expected)".to_string(),
                        );
                        return Ok(HttpResponse::UnprocessableEntity()
                            .json(ApiError::validation("Validation failed".into(), errors)));
                    }
                }
            }
        }
        None => None,
    };

    let stats = match db.list::<MissingTranslationStat>(I18N_MISSING_TREE) {
        Ok(list) => list,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "DB_LIST_FAILED",
                "Failed to load missing translation stats",
                false,
            )))
        }
    };

    let filtered: Vec<MissingTranslationStat> = match language_filter {
        Some(language) => stats
            .into_iter()
            .filter(|stat| stat.language == language)
            .collect(),
        None => stats,
    };

    Ok(HttpResponse::Ok().json(filtered))
}
