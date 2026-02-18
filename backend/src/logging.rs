use crate::{
    config::AppConfig,
    handlers::auth::{hash_email, inspect_token},
    log_format::{colors_enabled, current_log_format, LogFormat},
    models::auth_types::Claims,
};
use actix_http::{Payload, Version};
use actix_web::{
    body::{self, BodySize, BodyStream, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderMap, AUTHORIZATION, REFERER, USER_AGENT},
    web, Error, HttpMessage,
};
use base64::Engine;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde_json::json;
use serde_yaml;
use std::{
    future::{ready, Ready},
    io,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{Event, Level, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{self, format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    prelude::*,
    registry::LookupSpan,
    EnvFilter,
};

#[derive(Copy, Clone, Eq, PartialEq)]
enum AppEnv {
    Development,
    Testing,
    Staging,
    Production,
}

const COLOR_RESET: &str = "\x1b[0m";
const COLOR_GREY_80: &str = "\x1b[38;5;244m";
const COLOR_GREY_70: &str = "\x1b[38;5;240m";
const COLOR_INFO: &str = "\x1b[38;5;28m";
const COLOR_WARN: &str = "\x1b[33m";
const COLOR_ERROR: &str = "\x1b[31m";
const COLOR_DEBUG: &str = "\x1b[38;5;135m";
const COLOR_TRACE: &str = "\x1b[38;5;213m";
const COLOR_HTTP_VERB: &str = "\x1b[34m";
const COLOR_HTTP_ROUTE: &str = "\x1b[38;5;25m";
const COLOR_ALERT_RED: &str = "\x1b[38;5;196m";

const IGNORE_LIST_SPLIT_CHARS: &[char] = &[',', '\n', '\r'];
const HTTP_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD", "TRACE", "CONNECT",
];

/// Helper to return color code if LOG_COLORS=yes, otherwise empty string
fn color(code: &str) -> &str {
    if colors_enabled() {
        code
    } else {
        ""
    }
}

static HOST_SIGNATURE: Lazy<String> = Lazy::new(|| {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".into());
    let host = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "localhost".into());
    format!("{}@{}", user, host)
});

#[derive(Debug)]
struct RequestLogIgnoreRule {
    method: Option<String>,
    path: String,
}

static REQUEST_LOG_IGNORE_RULES: Lazy<Vec<RequestLogIgnoreRule>> = Lazy::new(|| {
    let _ = dotenvy::dotenv();
    std::env::var("LOG_IGNORE_LIST")
        .unwrap_or_default()
        .split(|c| IGNORE_LIST_SPLIT_CHARS.contains(&c))
        .filter_map(|entry| {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                return None;
            }

            let mut segments = trimmed.splitn(2, |c: char| c.is_whitespace());
            let primary = segments.next().unwrap_or_default();
            let secondary = segments.next().map(str::trim);

            if let Some(path_segment) = secondary {
                if HTTP_METHODS.contains(&primary.to_ascii_uppercase().as_str()) {
                    return Some(RequestLogIgnoreRule {
                        method: Some(primary.to_ascii_uppercase()),
                        path: path_segment.to_string(),
                    });
                }
            }

            Some(RequestLogIgnoreRule {
                method: None,
                path: trimmed.to_string(),
            })
        })
        .collect()
});

fn should_ignore_request(method: &str, path: &str) -> bool {
    if REQUEST_LOG_IGNORE_RULES.is_empty() {
        return false;
    }
    let normalized_method = method.to_ascii_uppercase();
    REQUEST_LOG_IGNORE_RULES.iter().any(|rule| {
        if rule.path != path {
            return false;
        }
        match &rule.method {
            Some(rule_method) => rule_method.eq_ignore_ascii_case(&normalized_method),
            None => true,
        }
    })
}

#[derive(Clone)]
pub(crate) enum BearerLogState {
    Missing,
    Invalid,
    Valid {
        hashed_email: String,
        user_id: String,
        exp: i64,
    },
    Expired {
        hashed_email: Option<String>,
        user_id: Option<String>,
        exp: Option<i64>,
    },
}

impl BearerLogState {
    fn actor_hash(&self) -> Option<&str> {
        match self {
            BearerLogState::Valid { hashed_email, .. } => Some(hashed_email),
            BearerLogState::Expired {
                hashed_email: Some(hash),
                ..
            } => Some(hash),
            _ => None,
        }
    }
}

impl FromStr for AppEnv {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_ascii_lowercase();
        Ok(match s.as_str() {
            "development" | "dev" => AppEnv::Development,
            "testing" | "test" => AppEnv::Testing,
            "staging" | "stage" => AppEnv::Staging,
            "production" | "prod" => AppEnv::Production,
            _ => AppEnv::Production,
        })
    }
}

fn detect_env() -> AppEnv {
    let _ = dotenvy::dotenv();
    std::env::var("APP_ENV")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(AppEnv::Production)
}

/// Check if development logging is explicitly enabled
/// Default: false (production-safe logging)
fn is_development_logging() -> bool {
    std::env::var("LOG_DEVELOPMENT")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Check if request body should be logged
/// LOG_BODY_SHOW=false disables body logging
/// Default: true (show body)
fn should_show_request_body() -> bool {
    std::env::var("LOG_BODY_SHOW")
        .ok()
        .map(|v| v != "false" && v != "0" && !v.eq_ignore_ascii_case("no"))
        .unwrap_or(true)
}

/// Check if path contains sensitive operations that shouldn't log request body
fn is_sensitive_path(path: &str) -> bool {
    path.contains("/auth/")
        || path.contains("/login")
        || path.contains("/register")
        || path.contains("/password")
        || path.contains("/mfa/")
        || path.contains("/verify")
}

/// Sanitize JSON object by redacting sensitive fields
fn sanitize_json(value: &mut serde_json::Value) {
    if let Some(obj) = value.as_object_mut() {
        // List of sensitive field names to redact
        let sensitive_fields = [
            // "password",
            // "new_password",
            // "old_password",
            "hashed_password",
            "password_hash",
            "secret",
            "token",
            "api_key",
            "api_secret",
            "hashed_email",
            "email_hash",
            "totp_code",
            "mfa_code",
            "access_token",
        ];

        for field in &sensitive_fields {
            if obj.contains_key(*field) {
                obj.insert(field.to_string(), json!("[REDACTED]"));
            }
        }

        // Recursively sanitize nested objects
        for (_, v) in obj.iter_mut() {
            sanitize_json(v);
        }
    } else if let Some(arr) = value.as_array_mut() {
        for item in arr {
            sanitize_json(item);
        }
    }
}

#[derive(Default)]
struct DevFormatter;

#[derive(Default)]
struct ColoredFieldVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl tracing::field::Visit for ColoredFieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let v = format!("{:?}", value);
        self.record_str(field, &v);
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.trim().to_string());
        } else {
            self.fields
                .push((field.name().to_string(), value.trim().to_string()));
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}

impl<S, N> FormatEvent<S, N> for DevFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let timestamp = Utc::now().to_rfc3339();
        let level_color = match *event.metadata().level() {
            Level::ERROR => color(COLOR_ERROR),
            Level::WARN => color(COLOR_WARN),
            Level::INFO => color(COLOR_INFO),
            Level::DEBUG => color(COLOR_DEBUG),
            Level::TRACE => color(COLOR_TRACE),
        };

        let host = HOST_SIGNATURE.as_str();
        write!(
            writer,
            "{}{}{} {}{}{} {}{}{} ",
            color(COLOR_GREY_80),
            timestamp,
            color(COLOR_RESET),
            color(COLOR_GREY_80),
            host,
            color(COLOR_RESET),
            level_color,
            event.metadata().level(),
            color(COLOR_RESET)
        )?;

        // Only show file:line in development logging mode
        if is_development_logging() {
            let file = event.metadata().file().unwrap_or("unknown");
            let line = event.metadata().line().unwrap_or(0);
            write!(
                writer,
                "{}{}:{}{} ",
                color(COLOR_GREY_70),
                file,
                line,
                color(COLOR_RESET)
            )?;
        }

        if let Some(span) = ctx.lookup_current() {
            write!(
                writer,
                "{}{}{} ",
                color(COLOR_GREY_70),
                span.name(),
                color(COLOR_RESET)
            )?;
        }

        let mut visitor = ColoredFieldVisitor::default();
        event.record(&mut visitor);

        if let Some(message) = visitor.message {
            write!(
                writer,
                "{}{}{} ",
                color(COLOR_GREY_80),
                message,
                color(COLOR_RESET)
            )?;
        }

        for (key, value) in visitor.fields {
            write!(
                writer,
                "{}{}{}={}{}{} ",
                color(COLOR_GREY_70),
                key,
                color(COLOR_RESET),
                color(COLOR_GREY_80),
                value,
                color(COLOR_RESET)
            )?;
        }

        writeln!(writer)
    }
}

/// Initialize the logging system with colors and build information
pub fn init_logging(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let app_name = std::env::var("APP_NAME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());

    let filter = if verbose {
        "debug,mitote_v026_bike_connect_backend=trace,actix_web=debug,backup=trace"
    } else {
        "info,mitote_v026_bike_connect_backend=info,actix_web=info,backup=trace"
    };
    let env_filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(filter))?;

    let _ = LogTracer::init();

    let app_env = detect_env();

    match app_env {
        AppEnv::Development | AppEnv::Testing | AppEnv::Staging => {
            // Check LOG_COLORS first, then FORCE_COLOR, then atty detection
            let use_ansi = if std::env::var("LOG_COLORS").is_ok() {
                colors_enabled()
            } else {
                let force_color = std::env::var("FORCE_COLOR")
                    .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);
                force_color || atty::is(atty::Stream::Stdout)
            };
            let fmt_layer = fmt::layer()
                .event_format(DevFormatter::default())
                .with_ansi(use_ansi)
                .with_writer(io::stdout);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .ok();
        }
        AppEnv::Production => {
            let fmt_layer = fmt::layer()
                .event_format(Rfc5424 {
                    app: app_name,
                    facility: 1,
                })
                .with_ansi(false)
                .fmt_fields(fmt::format::DefaultFields::new())
                .with_writer(io::stdout);
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .ok();
        }
    }

    tracing::info!(pid = %std::process::id(), env = %app_env_name(app_env), "startup");
    Ok(())
}

fn app_env_name(e: AppEnv) -> &'static str {
    match e {
        AppEnv::Development => "development",
        AppEnv::Testing => "testing",
        AppEnv::Staging => "staging",
        AppEnv::Production => "production",
    }
}

struct Rfc5424 {
    app: String,
    facility: u8,
}

impl<S, N> FormatEvent<S, N> for Rfc5424
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut w: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let md = event.metadata();
        let _sev = match *md.level() {
            tracing::Level::ERROR => 3,
            tracing::Level::WARN => 4,
            tracing::Level::INFO => 6,
            tracing::Level::DEBUG | tracing::Level::TRACE => 7,
        };
        let _pri = (self.facility as u16) * 8 + _sev as u16;

        let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "-".to_string());
        let procid = std::process::id();
        let msgid = if md.target().is_empty() {
            "-"
        } else {
            md.target()
        };
        let file = md.file().unwrap_or("-");
        let line = md
            .line()
            .map(|l| l.to_string())
            .unwrap_or_else(|| "-".into());

        let current = std::thread::current();
        let thread_name = current.name().unwrap_or("-");
        let thread_id = format!("{:?}", current.id());

        let mut vis = KvVisitor::default();
        event.record(&mut vis);
        let requester_ip = vis.ip.as_deref().unwrap_or("-");

        write!(
            &mut w,
            "{} {} {} {} {} {} ",
            requester_ip, ts, host, self.app, procid, msgid
        )?;
        write!(
            &mut w,
            "[meta@47450 level=\"{}\" target=\"{}\" file=\"{}\" line=\"{}\" thread.name=\"{}\" thread.id=\"{}\"]",
            md.level(), escape(md.target()), escape(file), escape(&line), escape(thread_name), escape(&thread_id)
        )?;

        if !vis.kv.is_empty() {
            w.write_str("[event@47450")?;
            for (k, v) in &vis.kv {
                write!(&mut w, " {}=\"{}\"", k, escape(v))?;
            }
            w.write_str("]")?;
        } else {
            w.write_str("[event@47450]")?;
        }

        w.write_str(" ")?;
        if let Some(m) = vis.message {
            w.write_str(&m)?;
        } else {
            w.write_str("-")?;
        }
        w.write_str("\n")
    }
}

#[derive(Default)]
struct KvVisitor {
    kv: Vec<(String, String)>,
    message: Option<String>,
    ip: Option<String>,
}

impl tracing::field::Visit for KvVisitor {
    fn record_str(&mut self, f: &tracing::field::Field, v: &str) {
        if f.name() == "message" {
            self.message = Some(v.to_string());
        } else if f.name() == "ip" {
            self.ip = Some(v.to_string());
            self.kv.push((f.name().to_string(), v.to_string()));
        } else {
            self.kv.push((f.name().to_string(), v.to_string()));
        }
    }
    fn record_i64(&mut self, f: &tracing::field::Field, v: i64) {
        self.kv.push((f.name().to_string(), v.to_string()));
    }
    fn record_u64(&mut self, f: &tracing::field::Field, v: u64) {
        self.kv.push((f.name().to_string(), v.to_string()));
    }
    fn record_bool(&mut self, f: &tracing::field::Field, v: bool) {
        self.kv.push((f.name().to_string(), v.to_string()));
    }
    fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
        let s = format!("{:?}", v);
        if f.name() == "message" {
            self.message = Some(trim_quotes(&s).to_string());
        } else if f.name() == "ip" {
            let trimmed = trim_quotes(&s).to_string();
            self.ip = Some(trimmed.clone());
            self.kv.push((f.name().to_string(), trimmed));
        } else {
            self.kv
                .push((f.name().to_string(), trim_quotes(&s).to_string()));
        }
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ']' => out.push_str("\\]"),
            _ => out.push(ch),
        }
    }
    out
}

fn trim_quotes(s: &str) -> &str {
    if let Some(t) = s.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
        t
    } else {
        s
    }
}

/// Print build and version information with colors
pub fn print_build_info() {
    let name = env!("CARGO_PKG_NAME");
    let pkg_version = env!("CARGO_PKG_VERSION");
    let version = std::env::var("APP_BUILD_VERSION").unwrap_or_else(|_| pkg_version.to_string());
    let build_timestamp =
        std::env::var("VERGEN_BUILD_TIMESTAMP").unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());
    let git_branch = std::env::var("VERGEN_GIT_BRANCH").unwrap_or_else(|_| "no-git".into());
    let git_sha_full = std::env::var("VERGEN_GIT_SHA").unwrap_or_else(|_| "00000000".into());
    let git_commit = git_sha_full.chars().take(8).collect::<String>();
    let rust_version = std::env::var("VERGEN_RUSTC_SEMVER")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    let desc = std::env::var("APP_PKG_DESCRIPTION").unwrap_or_else(|_| String::new());
    let bin_name = std::env::var("APP_BIN_FILENAME").unwrap_or_else(|_| String::new());

    println!("{}", "═".repeat(60));
    println!("{} v{}", name, version);
    if !desc.is_empty() {
        println!("Description: {}", desc);
    }
    if !bin_name.is_empty() {
        println!("Binary: {}", bin_name);
    }
    println!("{}", "─".repeat(60));
    println!("Build: {}", build_timestamp);
    println!("Git: {} ({})", git_branch, git_commit);
    println!("Rust: {}", rust_version);
    println!("{}", "═".repeat(60));
    println!();
}

/// Log database connection status with colors
#[allow(dead_code)]
pub fn log_database_status(connected: bool, db_type: &str, details: &str) {
    if connected {
        tracing::info!("✓ Connected to {} ({})", db_type, details);
    } else {
        tracing::error!("✗ Failed to connect to {} ({})", db_type, details);
    }
}

/// Log server startup information
pub fn log_server_startup(host: &str, port: u16) {
    tracing::info!("🚀 Server starting on http://{}:{}", host, port);
}

/// Log CLI command execution
pub fn log_command_start(command: &str, description: &str) {
    tracing::info!("⚡ Executing: {} ({})", command, description);
}

/// Log command completion
pub fn log_command_complete(command: &str, success: bool, duration: std::time::Duration) {
    let status_icon = if success { "✅" } else { "❌" };
    let status_text = if success { "completed" } else { "failed" };
    let status_color = if success { "green" } else { "red" };

    match status_color {
        "green" => tracing::info!(
            "{} Command '{}' {} in {:.2?}",
            status_icon,
            command,
            status_text,
            duration
        ),
        _ => tracing::error!(
            "{} Command '{}' {} after {:.2?}",
            status_icon,
            command,
            status_text,
            duration
        ),
    }
}

/// Log table operation status
#[allow(dead_code)]
pub fn log_table_operation(operation: &str, table: &str, count: Option<usize>, success: bool) {
    let icon = match operation {
        "create" | "seed" => "📝",
        "drop" => "🗑️",
        "dump" => "💾",
        "import" => "📥",
        _ => "🔄",
    };

    if success {
        let count_text = count
            .map(|c| format!(" ({} records)", c))
            .unwrap_or_default();
        tracing::info!(
            "{} {} table {}{}",
            icon,
            operation.to_uppercase(),
            table,
            count_text
        );
    } else {
        tracing::error!("{} Failed to {} table {}", "❌", operation, table);
    }
}

/// Log configuration loading
#[allow(dead_code)]
pub fn log_config_loaded(config_file: &str, settings_count: usize) {
    tracing::info!(
        "⚙️ Configuration loaded from {} ({} settings)",
        config_file,
        settings_count
    );
}

/// Log warning with icon
pub fn log_warning(message: &str) {
    tracing::warn!("⚠️ {}", message);
}

/// Log error with icon
pub fn log_error(message: &str) {
    tracing::error!("❌ {}", message);
}

/// Log success with icon
#[allow(dead_code)]
pub fn log_success(message: &str) {
    tracing::info!("✅ {}", message);
}

pub(crate) fn log_request_access(
    method: &str,
    path: &str,
    http_version: &str,
    status: u16,
    bytes_sent: usize,
    query_string: &str,
    headers: &HeaderMap,
    client_ip: Option<&str>,
    rt: Duration,
    upstream_rt: Option<Duration>,
    body: Option<&[u8]>,
    bearer: BearerLogState,
    actor_hash: Option<String>,
) {
    let ip = client_ip.unwrap_or("-");
    let ts = fmt_apache_time(Utc::now());
    let qs_part = if query_string.is_empty() { "" } else { "?" };
    let referer = hdr(headers, REFERER).unwrap_or("-");
    let ua = hdr(headers, USER_AGENT).unwrap_or("-");
    let rt_s = dur_s(rt);
    let up_rt_s = upstream_rt.map(dur_s).unwrap_or(0.0);

    let body_b64 = body
        .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
        .unwrap_or_default();

    let actor_display = actor_hash
        .or_else(|| bearer.actor_hash().map(|s| s.to_string()))
        .unwrap_or_else(|| "anonymous".into());

    // Build structured JSON log for Splunk/ELK/security analysis
    let mut log_data = serde_json::Map::new();
    log_data.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
    log_data.insert("client_ip".to_string(), json!(ip));
    log_data.insert("method".to_string(), json!(method));
    log_data.insert("path".to_string(), json!(path));
    log_data.insert("query_string".to_string(), json!(query_string));
    log_data.insert("http_version".to_string(), json!(http_version));
    log_data.insert("status".to_string(), json!(status));
    log_data.insert("bytes_sent".to_string(), json!(bytes_sent));
    log_data.insert(
        "response_time_ms".to_string(),
        json!((rt_s * 1000.0) as u64),
    );
    log_data.insert(
        "upstream_response_time_ms".to_string(),
        json!((up_rt_s * 1000.0) as u64),
    );
    log_data.insert("referer".to_string(), json!(referer));
    log_data.insert("user_agent".to_string(), json!(ua));
    log_data.insert("actor".to_string(), json!(actor_display));

    // Add bearer token info (sanitized for production)
    let show_sensitive = is_development_logging();
    match &bearer {
        BearerLogState::Missing => {
            log_data.insert("auth_status".to_string(), json!("NO_TOKEN"));
        }
        BearerLogState::Invalid => {
            log_data.insert("auth_status".to_string(), json!("INVALID_TOKEN"));
        }
        BearerLogState::Valid {
            hashed_email,
            user_id,
            exp,
        } => {
            log_data.insert("auth_status".to_string(), json!("BEARER_AUTH"));
            // In production, only show auth method without sensitive user details
            if show_sensitive {
                log_data.insert("user_hash".to_string(), json!(hashed_email));
                log_data.insert("user_id".to_string(), json!(user_id));
                log_data.insert("token_expires".to_string(), json!(format_epoch(Some(*exp))));
            }
        }
        BearerLogState::Expired {
            hashed_email,
            user_id,
            exp,
        } => {
            log_data.insert("auth_status".to_string(), json!("EXPIRED_TOKEN"));
            if show_sensitive {
                if let Some(email) = hashed_email {
                    log_data.insert("user_hash".to_string(), json!(email));
                }
                if let Some(uid) = user_id {
                    log_data.insert("user_id".to_string(), json!(uid));
                }
                if let Some(e) = exp {
                    log_data.insert("token_expires".to_string(), json!(format_epoch(Some(*e))));
                }
            }
        }
    }

    // Add all headers as nested object (with sensitive header redaction)
    let mut headers_map = serde_json::Map::new();
    let sensitive_headers = [
        "access_token",
        "authorization",
        "cookie",
        "x-api-key",
        "x-api-secret",
    ];
    for (k, val) in headers.iter() {
        let key_lower = k.as_str().to_lowercase();
        let header_value = if sensitive_headers.contains(&key_lower.as_str()) {
            if !show_sensitive {
                "[REDACTED]"
            } else {
                val.to_str().unwrap_or("[invalid]")
            }
        } else {
            val.to_str().unwrap_or("[invalid]")
        };
        headers_map.insert(
            k.as_str().to_uppercase().replace("-", "_"),
            json!(header_value),
        );
    }
    log_data.insert("headers".to_string(), json!(headers_map));

    // Add request body (base64 encoded) if present and safe
    // For sensitive endpoints, skip body logging in production to avoid logging passwords
    // Also respect LOG_BODY_SHOW environment variable
    let show_body = should_show_request_body();
    let skip_body_log = (!show_sensitive && is_sensitive_path(path)) || !show_body;

    // if !body_b64.is_empty() && !skip_body_log {
    if !body_b64.is_empty() {
        log_data.insert("request_body_b64".to_string(), json!(body_b64));

        // Try to decode and parse as JSON for easier analysis
        if let Some(body_bytes) = body {
            if let Ok(body_str) = std::str::from_utf8(body_bytes) {
                // Try to parse as JSON
                if let Ok(mut body_json) = serde_json::from_str::<serde_json::Value>(body_str) {
                    // Sanitize sensitive fields in JSON
                    sanitize_json(&mut body_json);
                    log_data.insert("request_body_json".to_string(), body_json);
                } else {
                    // Not JSON, include as string (truncated if too long)
                    let body_preview = if body_str.len() > 1000 {
                        format!(
                            "{}... [truncated {} bytes]",
                            &body_str[..1000],
                            body_str.len()
                        )
                    } else {
                        body_str.to_string()
                    };
                    log_data.insert("request_body_text".to_string(), json!(body_preview));
                }
            }
        }
    } else if skip_body_log {
        let reason = if !show_body {
            "LOG_BODY_SHOW=false"
        } else {
            "Sensitive endpoint"
        };
        log_data.insert(
            "request_body".to_string(),
            json!(format!("[REDACTED - {}]", reason)),
        );
    }

    // Output structured log value
    let log_value = serde_json::Value::Object(log_data);
    let formatted_log = match current_log_format() {
        LogFormat::Json => serde_json::to_string(&log_value)
            .unwrap_or_else(|_| "{\"error\":\"formatting failure\"}".into()),
        LogFormat::Yaml => serde_yaml::to_string(&log_value)
            .map(|s| s.trim_end().to_string())
            .unwrap_or_else(|_| "---\nerror: formatting failure".into()),
    };

    // Log as structured JSON (single line)
    tracing::info!(target: "access", ip = %ip, "{}", formatted_log);

    // Also log Apache-style format for human readability (colored if LOG_COLORS=yes)
    let method_fmt = format!(
        "{}{:<6}{}",
        color(COLOR_HTTP_VERB),
        method,
        color(COLOR_RESET)
    );
    let route_fmt = format!(
        "{}{}{}{}{}",
        color(COLOR_HTTP_ROUTE),
        path,
        qs_part,
        query_string,
        color(COLOR_RESET)
    );
    let bearer_segment = format_bearer_segment(&bearer);

    let human_line = format!(
        r#"{ip} - - [{ts}] "{method_fmt} {route_fmt} {http_version}" {status} {bytes_sent} "{referer}" "{ua}" rt={rt_s:.3}s {bearer}"#,
        ip = ip,
        ts = ts,
        method_fmt = method_fmt,
        route_fmt = route_fmt,
        http_version = http_version,
        status = status,
        bytes_sent = bytes_sent,
        referer = referer,
        ua = ua,
        bearer = bearer_segment
    );

    tracing::debug!(target: "access_human", ip = %ip, "{}", human_line);
}

fn hdr<'a>(h: &'a HeaderMap, key: actix_web::http::header::HeaderName) -> Option<&'a str> {
    h.get(key).and_then(|v| v.to_str().ok())
}

fn dur_s(d: Duration) -> f64 {
    d.as_secs_f64()
}

fn fmt_apache_time(t: DateTime<Utc>) -> String {
    t.format("%d/%b/%Y:%H:%M:%S +0000").to_string()
}

fn format_epoch(exp: Option<i64>) -> String {
    exp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "-".into())
}

fn format_bearer_segment(bearer: &BearerLogState) -> String {
    match bearer {
        BearerLogState::Missing => format!(
            "{}Bearer=NOT FOUND{}",
            color(COLOR_ALERT_RED),
            color(COLOR_RESET)
        ),
        BearerLogState::Invalid => format!(
            "{}Bearer=INVALID{}",
            color(COLOR_ALERT_RED),
            color(COLOR_RESET)
        ),
        BearerLogState::Valid {
            hashed_email,
            user_id,
            exp,
        } => format!(
            "Bearer user={} user_id={} exp={}",
            hashed_email,
            user_id,
            format_epoch(Some(*exp))
        ),
        BearerLogState::Expired {
            hashed_email,
            user_id,
            exp,
        } => format!(
            "{}Bearer=EXPIRED{} user={} user_id={} exp={}",
            color(COLOR_ALERT_RED),
            color(COLOR_RESET),
            hashed_email.clone().unwrap_or_else(|| "unknown".into()),
            user_id.clone().unwrap_or_else(|| "unknown".into()),
            format_epoch(*exp)
        ),
    }
}

fn extract_bearer(header: &str) -> Option<&str> {
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn analyze_bearer(headers: &HeaderMap, cfg: Option<&AppConfig>) -> BearerLogState {
    let header_value = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let token = header_value.and_then(extract_bearer);
    if let Some(token) = token {
        if let Some(cfg) = cfg {
            if let Some(snapshot) = inspect_token(cfg, token) {
                if snapshot.expired {
                    return BearerLogState::Expired {
                        hashed_email: Some(hash_email(&snapshot.claims.email)),
                        user_id: Some(snapshot.claims.sub),
                        exp: Some(snapshot.claims.exp),
                    };
                }
                return BearerLogState::Valid {
                    hashed_email: hash_email(&snapshot.claims.email),
                    user_id: snapshot.claims.sub,
                    exp: snapshot.claims.exp,
                };
            }
        }
        BearerLogState::Invalid
    } else {
        BearerLogState::Missing
    }
}

/// Log detailed request information for tracking purposes
// pub fn log_request_details(
//     method: &str,
//     path: &str,
//     query_string: &str,
//     headers: &actix_web::http::header::HeaderMap,
//     client_ip: Option<String>,
// ) {
//     use chrono::Utc;
//
//     let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
//
//     tracing::info!("- - - - - - - - - - - - - - - - - Tracked on: {}", timestamp);
//     tracing::info!("Method: {} | Path: {}", method, path);
//
//     // Log client IP information
//     if let Some(ip) = client_ip {
//         let ip_type = if ip.contains(':') && !ip.starts_with("::ffff:") {
//             "IPv6"
//         } else if ip.starts_with("::ffff:") {
//             "IPv4 (mapped)"
//         } else {
//             "IPv4"
//         };
//         tracing::info!("Client IP: {} ({})", ip, ip_type);
//     }
//
//     // Log query parameters
//     tracing::info!("- - - Query Parameters:");
//     if query_string.is_empty() {
//         tracing::info!("  (none)");
//     } else {
//         for param in query_string.split('&') {
//             if let Some((key, value)) = param.split_once('=') {
//                 tracing::info!("  {}: {}", key, value);
//             } else {
//                 tracing::info!("  {}", param);
//             }
//         }
//     }
//
//     // Log headers
//     tracing::info!("- - - Headers:");
//     for (key, value) in headers.iter() {
//         if let Ok(val_str) = value.to_str() {
//             tracing::info!("  {}: {}", key.as_str(), val_str);
//         } else {
//             tracing::info!("  {}: <binary data>", key.as_str());
//         }
//     }
// }

/// Middleware for tracking all incoming requests
pub struct RequestTracker;
impl<S, B> Transform<S, ServiceRequest> for RequestTracker
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestTrackerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestTrackerMiddleware {
            service: Arc::new(service),
        }))
    }
}
// impl<S, B> Transform<S, ServiceRequest> for RequestTracker
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type InitError = ();
//     type Transform = RequestTrackerMiddleware<S>;
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;
//
//     fn new_transform(&self, service: S) -> Self::Future {
//         ready(Ok(RequestTrackerMiddleware { service }))
//     }
// }

pub struct RequestTrackerMiddleware<S> {
    service: Arc<S>,
}

/// log_request_details -version
// impl<S, B> Service<ServiceRequest> for RequestTrackerMiddleware<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: 'static,
// {
//     type Response = ServiceResponse<B>;
//     type Error = Error;
//     type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;
//
//     forward_ready!(service);
//
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         let method = req.method().to_string();
//         let path = req.path().to_string();
//         let query_string = req.query_string().to_string();
//         let headers = req.headers().clone();
//
//         // Extract client IP from connection info
//         let client_ip = req
//             .connection_info()
//             .realip_remote_addr()
//             .map(|s| s.to_string());
//
//         // Log the request details
//         log_request_details(&method, &path, &query_string, &headers, client_ip);
//
//         let fut = self.service.call(req);
//
//         Box::pin(async move {
//             let res = fut.await?;
//             Ok(res)
//         })
//     }
// }
/// log_request_access -version
impl<S, B> Service<ServiceRequest> for RequestTrackerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let started = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query_string = req.query_string().to_string();
        let headers = req.headers().clone();
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .map(str::to_string);
        let http_version = match req.version() {
            Version::HTTP_09 => "HTTP/0.9",
            Version::HTTP_10 => "HTTP/1.0",
            Version::HTTP_11 => "HTTP/1.1",
            Version::HTTP_2 => "HTTP/2.0",
            Version::HTTP_3 => "HTTP/3.0",
            _ => "HTTP/?",
        }
        .to_string();

        let cfg_data = req.app_data::<web::Data<AppConfig>>().cloned();
        let service = Arc::clone(&self.service);

        Box::pin(async move {
            let (http_req, payload) = req.into_parts();
            let stream = BodyStream::new(payload);
            let body_bytes = match body::to_bytes(stream).await {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::warn!("Failed to capture request body for logging: {}", err);
                    Bytes::new()
                }
            };

            let body_vec = if body_bytes.is_empty() {
                None
            } else {
                Some(body_bytes.clone().to_vec())
            };

            let new_payload = if body_bytes.is_empty() {
                Payload::None
            } else {
                Payload::from(body_bytes.clone())
            };

            let service_req = ServiceRequest::from_parts(http_req, new_payload);
            let fut = service.call(service_req);
            let res = fut.await?;

            if should_ignore_request(&method, &path) {
                return Ok(res);
            }

            let rt = started.elapsed();
            let status = res.status().as_u16();
            let bytes_sent = match res.response().body().size() {
                BodySize::Sized(n) => n as usize,
                _ => 0,
            };

            let cfg_ref = cfg_data.as_ref().map(|data| data.get_ref());
            let bearer = analyze_bearer(res.request().headers(), cfg_ref);
            let actor_hash = res
                .request()
                .extensions()
                .get::<Claims>()
                .map(|claims| hash_email(&claims.email));

            log_request_access(
                &method,
                &path,
                &http_version,
                status,
                bytes_sent,
                &query_string,
                &headers,
                client_ip.as_deref(),
                rt,
                None,
                body_vec.as_deref(),
                bearer,
                actor_hash,
            );

            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_initialization() {
        // Test that logging initialization doesn't panic
        let result = init_logging(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_info_display() {
        // This test just ensures the function doesn't panic
        // In a real scenario, we'd capture stdout and verify output
        print_build_info();
    }
}
