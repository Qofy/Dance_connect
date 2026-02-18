use once_cell::sync::Lazy;
use std::env;

/// Format for structured logs.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum LogFormat {
    Json,
    Yaml,
}

impl LogFormat {
    fn from_env_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_ref() {
            "json" => LogFormat::Json,
            _ => LogFormat::Yaml,
        }
    }
}

static LOG_FORMAT: Lazy<LogFormat> = Lazy::new(|| {
    // Load .env file first
    let _ = dotenvy::dotenv();

    if let Ok(val) = env::var("LOG_FORMAT") {
        LogFormat::from_env_value(&val)
    } else {
        LogFormat::Yaml
    }
});

/// Returns the configured log format, defaulting to YAML.
pub fn current_log_format() -> LogFormat {
    *LOG_FORMAT
}

static LOG_COLORS: Lazy<bool> = Lazy::new(|| {
    // Load .env file first
    let _ = dotenvy::dotenv();

    env::var("LOG_COLORS")
        .map(|val| {
            let normalized = val.trim().to_ascii_lowercase();
            normalized == "yes" || normalized == "true" || normalized == "1"
        })
        .unwrap_or(false) // Default to "no" (false)
});

/// Returns whether colored logs are enabled (default: false).
pub fn colors_enabled() -> bool {
    *LOG_COLORS
}
