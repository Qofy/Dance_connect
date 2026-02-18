use actix_web::{get, web, HttpResponse, Result};

use crate::handlers::errors::ApiError;
use crate::validation_rules::registry::get_action_rule;

fn verbose_feedback_enabled() -> bool {
    std::env::var("MODEL_VALIDATION_FEEDBACK")
        .ok()
        .map(|val| matches!(val.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

/// #feature[system]
#[get("/validation/{model}/{action}")]
pub async fn validation_rule(path: web::Path<(String, String)>) -> Result<HttpResponse> {
    let (model, action) = path.into_inner();
    let model_key = model.to_lowercase();
    let action_key = action.to_lowercase();

    match get_action_rule(&model_key, &action_key) {
        Some(rule) => Ok(HttpResponse::Ok().json(rule)),
        None => {
            let message = if verbose_feedback_enabled() {
                format!("Validation rule not found for model '{model}' action '{action}'")
            } else {
                "Validation rule not found".to_string()
            };
            Ok(HttpResponse::NotFound().json(ApiError {
                status: 404,
                code: "VALIDATION_RULE_NOT_FOUND",
                message,
                errors: None,
                timestamp: crate::time::now(),
                trace: None,
            }))
        }
    }
}
