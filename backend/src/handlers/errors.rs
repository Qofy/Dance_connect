use crate::time::now;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ApiError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<HashMap<String, String>>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
}

impl ApiError {
    pub fn validation(message: String, errors: HashMap<String, String>) -> Self {
        Self {
            status: 422,
            code: "VALIDATION_ERROR",
            message,
            errors: Some(errors),
            timestamp: now(),
            trace: None,
        }
    }

    #[allow(dead_code)]
    pub fn deserialization(details: String) -> Self {
        Self {
            status: 400,
            code: "DESERIALIZATION_ERROR",
            message: "Malformed JSON body".into(),
            errors: None,
            timestamp: now(),
            trace: Some(details),
        }
    }

    pub fn internal<M: Into<String>, T: std::fmt::Display>(
        code: M,
        err: T,
        include_trace: bool,
    ) -> Self {
        Self {
            status: 500,
            code: Box::leak(code.into().into_boxed_str()),
            message: "Internal server error".into(),
            errors: None,
            timestamp: now(),
            trace: if include_trace {
                Some(format!("{err}"))
            } else {
                None
            },
        }
    }

    pub fn forbidden() -> Self {
        Self {
            status: 403,
            code: "FORBIDDEN",
            message: "Access denied".into(),
            errors: None,
            timestamp: now(),
            trace: None,
        }
    }

    pub fn not_found(entity: &str) -> Self {
        Self {
            status: 404,
            code: "NOT_FOUND",
            message: format!("{entity} not found"),
            errors: None,
            timestamp: now(),
            trace: None,
        }
    }

    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: 401,
            code: "UNAUTHORIZED",
            message: message.to_string(),
            errors: None,
            timestamp: now(),
            trace: None,
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self {
            status: 400,
            code: "BAD_REQUEST",
            message: message.to_string(),
            errors: None,
            timestamp: now(),
            trace: None,
        }
    }

    pub fn feature_locked() -> Self {
        Self {
            status: 403,
            code: "FEATURE_LOCKED",
            message: "This feature requires a premium subscription tier".into(),
            errors: None,
            timestamp: now(),
            trace: None,
        }
    }

    pub fn from_serde_error(err: &serde_json::Error) -> Self {
        let mut errors: HashMap<String, String> = HashMap::new();
        let msg = err.to_string();
        let verbose = std::env::var("MODEL_VALIDATION_FEEDBACK")
            .ok()
            .map(|val| matches!(val.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);

        let missing_re = Regex::new(r#"missing field `([^`]+)`"#).unwrap();
        if let Some(cap) = missing_re.captures(&msg) {
            let field = cap.get(1).map(|m| m.as_str()).unwrap_or("body");
            errors.insert(field.to_string(), "required field is missing".into());
        }

        let unknown_re = Regex::new(r#"unknown field `([^`]+)`"#).unwrap();
        if let Some(cap) = unknown_re.captures(&msg) {
            let field = cap.get(1).map(|m| m.as_str()).unwrap_or("body");
            let mut detail = "field is not allowed".to_string();
            if verbose {
                let tick_re = Regex::new(r#"`([^`]+)`"#).unwrap();
                let mut allowed: Vec<&str> = tick_re
                    .captures_iter(&msg)
                    .filter_map(|c| c.get(1).map(|m| m.as_str()))
                    .filter(|val| *val != field)
                    .collect();
                allowed.sort_unstable();
                allowed.dedup();
                if !allowed.is_empty() {
                    detail = format!(
                        "field is not allowed. Allowed fields: {}",
                        allowed.join(", ")
                    );
                }
            }
            errors.insert(field.to_string(), detail);
        }

        let invalid_re =
            Regex::new(r#"invalid type: ([^,]+), expected ([^,]+) at line \d+ column \d+"#)
                .unwrap();
        if let Some(cap) = invalid_re.captures(&msg) {
            let got = cap.get(1).map(|m| m.as_str()).unwrap_or("value");
            let expected = cap.get(2).map(|m| m.as_str()).unwrap_or("valid type");
            errors.insert(
                "_json".into(),
                format!("invalid type {got}; expected {expected}"),
            );
        }

        if errors.is_empty() {
            errors.insert("_json".into(), msg.clone());
        }

        Self {
            status: 400,
            code: "VALIDATION_ERROR",
            message: "Invalid request body".into(),
            errors: Some(errors),
            timestamp: now(),
            trace: None,
        }
    }
}
