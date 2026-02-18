use chrono::{Local, Utc};
use colored::*;
use env_logger::{Builder, Env};
use log::info;
use serde_json::json;
use serde_yaml;
use std::io::Write;

use crate::log_format::{colors_enabled, current_log_format, LogFormat};

pub fn setup_logger() {
    // Load .env file before checking environment variables
    let _ = dotenvy::dotenv();

    let log_format = current_log_format();
    let use_colors = colors_enabled();

    Builder::from_env(Env::default().default_filter_or("info"))
        .format(move |buf, record| {
            let file = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);
            let timestamp = Local::now().format("%Y%m%d %H:%M:%S").to_string();

            // Format level with colors if enabled
            let level_str = if use_colors {
                match record.level() {
                    log::Level::Error => format!("{}", record.level()).red().to_string(),
                    log::Level::Warn => format!(" {}", record.level()).yellow().to_string(),
                    log::Level::Info => format!(" {}", record.level()).green().to_string(),
                    log::Level::Debug => format!("{}", record.level()).blue().to_string(),
                    log::Level::Trace => format!("{}", record.level()).purple().to_string(),
                }
            } else {
                format!("{}", record.level())
            };

            let entry = json!({
                "timestamp": Utc::now().to_rfc3339(),
                "level": record.level().to_string(),
                "target": record.target(),
                "file": file,
                "line": line,
                "message": format!("{}", record.args()),
            });

            let _formatted = match log_format {
                LogFormat::Json => serde_json::to_string(&entry)
                    .unwrap_or_else(|_| "{\"error\":\"formatting failure\"}".into()),
                LogFormat::Yaml => serde_yaml::to_string(&entry)
                    .map(|s| s.trim_end().to_string())
                    .unwrap_or_else(|_| "---\nerror: formatting failure".into()),
            };

            // Output format with colors if enabled
            if use_colors {
                writeln!(
                    buf,
                    "{}:{} [{}]{}: {}",
                    file,
                    line,
                    timestamp.purple(),
                    level_str,
                    record.args()
                )
            } else {
                writeln!(
                    buf,
                    "{}:{} [{}]{}: {}",
                    file,
                    line,
                    timestamp,
                    level_str,
                    record.args()
                )
            }
        })
        .init();

    info!("Logger initialized");
}

/// Create a custom Logger middleware that respects LOG_IGNORE_LIST
/// Filters out requests matching the ignore rules before logging
pub fn create_logger_middleware() -> actix_web::middleware::Logger {
    use actix_web::middleware::Logger;

    let _ = dotenvy::dotenv();

    const IGNORE_LIST_SPLIT_CHARS: &[char] = &[',', '\n', '\r'];
    const HTTP_METHODS: &[&str] = &[
        "GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD", "TRACE", "CONNECT",
    ];

    let raw = std::env::var("LOG_IGNORE_LIST").unwrap_or_default();
    let mut logger = Logger::default();

    for entry in raw.split(|c| IGNORE_LIST_SPLIT_CHARS.contains(&c)) {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Supported formats:
        // - "/api/health"
        // - "GET /api/health"
        let mut parts = trimmed.split_whitespace();
        let first = parts.next().unwrap_or_default();
        let second = parts.next();

        let path = if let Some(path_candidate) = second {
            if HTTP_METHODS.contains(&first.to_ascii_uppercase().as_str()) {
                path_candidate
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        if path.starts_with('/') {
            logger = logger.exclude(path);
        }
    }

    logger
}
