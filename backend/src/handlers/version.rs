use actix_web::{get, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub version: String,
    pub build: String,
    pub git_commit: String,
    pub build_timestamp: String,
    pub rust_version: String,
}

impl VersionInfo {
    pub fn new() -> Self {
        Self {
            version: std::env::var("APP_BUILD_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
            build: std::env::var("VERGEN_BUILD_TIMESTAMP")
                .unwrap_or_else(|_| "unknown".to_string()),
            git_commit: std::env::var("VERGEN_GIT_SHA").unwrap_or_else(|_| "unknown".to_string()),
            build_timestamp: std::env::var("VERGEN_BUILD_TIMESTAMP")
                .unwrap_or_else(|_| "unknown".to_string()),
            rust_version: std::env::var("VERGEN_RUSTC_SEMVER")
                .unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    pub fn to_text(&self) -> String {
        format!(
            "Version: {}\nBuild: {}\nGit Commit: {}\nBuild Timestamp: {}\nRust Version: {}",
            self.version, self.build, self.git_commit, self.build_timestamp, self.rust_version
        )
    }

    pub fn to_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<version>
    <version>{}</version>
    <build>{}</build>
    <git_commit>{}</git_commit>
    <build_timestamp>{}</build_timestamp>
    <rust_version>{}</rust_version>
</version>"#,
            self.version, self.build, self.git_commit, self.build_timestamp, self.rust_version
        )
    }
}

/// #feature[system]
#[get("/version")]
pub async fn get_version(req: HttpRequest) -> HttpResponse {
    let version_info = VersionInfo::new();

    // Content negotiation based on Accept header
    let accept = req
        .headers()
        .get("accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/json");

    if accept.contains("application/xml") || accept.contains("text/xml") {
        HttpResponse::Ok()
            .content_type("application/xml")
            .body(version_info.to_xml())
    } else if accept.contains("text/plain") {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(version_info.to_text())
    } else {
        // Default to JSON
        HttpResponse::Ok().json(version_info)
    }
}
