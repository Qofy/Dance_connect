use actix_web::{get, put, web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;

use crate::authz::require_manage_system_settings;
use crate::db::Database;
use crate::handlers::errors::ApiError;
use crate::models::settings::UiSettings;
use crate::ui_languages::normalize_ui_language;
use std::collections::HashMap;

const SETTINGS_TREE: &str = "settings";
const UI_SETTINGS_KEY: &str = "ui";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateUiSettingsRequest {
    pub default_language: String,
}

fn load_ui_settings(db: &Database) -> UiSettings {
    db.get::<UiSettings>(SETTINGS_TREE, UI_SETTINGS_KEY)
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn save_ui_settings(db: &Database, settings: &UiSettings) -> Result<(), actix_web::Error> {
    db.insert(SETTINGS_TREE, UI_SETTINGS_KEY, settings)
        .map_err(actix_web::error::ErrorInternalServerError)
}

#[get("/ui-settings")]
pub async fn get_ui_settings(db: web::Data<Database>) -> Result<HttpResponse> {
    let settings = load_ui_settings(&db);
    Ok(HttpResponse::Ok().json(settings))
}

#[put("/ui-settings")]
pub async fn update_ui_settings(
    req: HttpRequest,
    db: web::Data<Database>,
    body: web::Json<UpdateUiSettingsRequest>,
) -> Result<HttpResponse> {
    let _claims = match require_manage_system_settings(&req) {
        Ok(claims) => claims,
        Err(response) => return Ok(response),
    };

    let normalized = match normalize_ui_language(&body.default_language) {
        Some(value) => value,
        None => {
            let mut errors = HashMap::new();
            errors.insert(
                "default_language".to_string(),
                "Invalid language code (supported UI language expected)".to_string(),
            );
            return Ok(HttpResponse::UnprocessableEntity()
                .json(ApiError::validation("Validation failed".into(), errors)));
        }
    };

    let mut settings = load_ui_settings(&db);
    settings.default_language = normalized;
    if let Err(err) = save_ui_settings(&db, &settings) {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(settings))
}
