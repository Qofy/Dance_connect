// handlers/auth.rs
use actix_web::{delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Result};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use pasetors::{
    claims::{Claims as PasetoClaims, ClaimsValidationRules},
    keys::SymmetricKey,
    local,
    token::UntrustedToken,
    version4::V4,
};
use rand_core::OsRng;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration as StdDuration, Instant};
use time::{format_description::well_known::Rfc3339, Duration as TimeDuration, OffsetDateTime};

use crate::config::{AppConfig, TokenMode};
use crate::{
    authz::{can_manage_system_settings, is_admin, require_claims, user_is_admin},
    db::Database,
    models::{
        auth_types::{Claims, LoginRequest, PasswordEncoding, RegisterRequest, UserRecord},
        settings::{AuthSettings, InviteToken},
        Company, CompanyInvite, CompanyUserAccess, UpdateCompanyRequest,
    },
};
use tracing::warn;

#[allow(dead_code)]
type HmacSha256 = Hmac<Sha256>;

#[allow(dead_code)]
fn base64url(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

#[allow(dead_code)]
fn sign_hs256(secret: &[u8], header_b64: &str, payload_b64: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    let signing_input = format!("{}.{}", header_b64, payload_b64);
    mac.update(signing_input.as_bytes());
    let sig = mac.finalize().into_bytes();
    base64url(&sig)
}

#[allow(dead_code)]
fn verify_hs256(secret: &[u8], token: &str) -> Option<(serde_json::Value, serde_json::Value)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let (h, p, s) = (parts[0], parts[1], parts[2]);
    let expected_sig = sign_hs256(secret, h, p);
    if expected_sig != s {
        return None;
    }
    let header_bytes = match URL_SAFE_NO_PAD.decode(h) {
        Ok(value) => value,
        Err(_) => return None,
    };
    let payload_bytes = match URL_SAFE_NO_PAD.decode(p) {
        Ok(value) => value,
        Err(_) => return None,
    };
    let header = match serde_json::from_slice(&header_bytes) {
        Ok(value) => value,
        Err(_) => return None,
    };
    let payload = match serde_json::from_slice(&payload_bytes) {
        Ok(value) => value,
        Err(_) => return None,
    };
    Some((header, payload))
}

#[allow(dead_code)]
fn default_secret(cfg: &AppConfig) -> Vec<u8> {
    if !cfg.security.paseto_v4_local_key_hex.is_empty() {
        hex::decode(cfg.security.paseto_v4_local_key_hex.trim())
            .unwrap_or_else(|_| cfg.security.paseto_v4_local_key_hex.clone().into_bytes())
    } else if !cfg.security.access_token.is_empty() {
        cfg.security.access_token.clone().into_bytes()
    } else {
        // Last resort
        b"mitote_v026_bike_connect_backend_default_secret".to_vec()
    }
}

#[allow(dead_code)]
fn make_token_hmac(cfg: &AppConfig, claims: &Claims) -> String {
    let header = json!({"alg":"HS256","typ":"JWT"});
    let header_b64 = base64url(&serde_json::to_vec(&header).unwrap());
    let payload_b64 = base64url(&serde_json::to_vec(claims).unwrap());
    let key = default_secret(cfg);
    let sig = sign_hs256(&key, &header_b64, &payload_b64);
    format!("{}.{}.{}", header_b64, payload_b64, sig)
}

#[allow(dead_code)]
fn validate_token_hmac(cfg: &AppConfig, token: &str) -> Option<Claims> {
    let key = default_secret(cfg);
    if let Some((_h, p)) = verify_hs256(&key, token) {
        let claims: Claims = match serde_json::from_value(p) {
            Ok(value) => value,
            Err(_) => return None,
        };
        let now = Utc::now().timestamp();
        if claims.exp < now {
            return None;
        }
        if claims.iss != cfg.security.token_iss || claims.aud != cfg.security.token_aud {
            return None;
        }
        Some(claims)
    } else if !cfg.security.access_token.is_empty() && cfg.security.access_token == token {
        // Static access token fallback (admin privileges)
        Some(Claims {
            sub: "access".into(),
            email: "access@local".into(),
            roles: vec!["admin".into()],
            permissions: vec!["*".into()],
            access_level: Some("admin".into()),
            subscription_tier: Some("enterprise".into()),
            tenant_id: None,
            iss: cfg.security.token_iss.clone(),
            aud: cfg.security.token_aud.clone(),
            iat: Utc::now().timestamp(),
            exp: (Utc::now() + Duration::hours(cfg.security.auth_token_expiry_hours as i64))
                .timestamp(),
        })
    } else {
        None
    }
}

#[allow(dead_code)]
fn extract_bearer_or_query(req: &actix_web::HttpRequest) -> Option<String> {
    if let Some(h) = req.headers().get("authorization") {
        if let Ok(s) = h.to_str() {
            if let Some(rest) = s.strip_prefix("Bearer ") {
                return Some(rest.to_string());
            }
        }
    }
    if let Some(q) = req
        .query_string()
        .split('&')
        .find(|p| p.starts_with("access_token="))
    {
        if let Some(val) = q.split('=').nth(1) {
            return Some(val.to_string());
        }
    }
    None
}

fn extract_token(req: &actix_web::HttpRequest) -> Option<String> {
    if let Some(h) = req.headers().get("authorization") {
        if let Ok(s) = h.to_str() {
            if let Some(rest) = s
                .strip_prefix("Bearer ")
                .or_else(|| s.strip_prefix("bearer "))
            {
                if !rest.trim().is_empty() {
                    return Some(rest.trim().to_string());
                }
            }
        }
    }
    if let Some(q) = req
        .query_string()
        .split('&')
        .find(|p| p.starts_with("access_token="))
    {
        if let Some(val) = q.split('=').nth(1) {
            if !val.trim().is_empty() {
                return Some(val.trim().to_string());
            }
        }
    }
    None
}

#[allow(dead_code)]
fn paseto_key(cfg: &AppConfig) -> Option<SymmetricKey<V4>> {
    let hex = cfg.security.paseto_v4_local_key_hex.trim();
    if hex.len() < 64 {
        return None;
    }
    let bytes = match hex::decode(hex) {
        Ok(value) => value,
        Err(_) => return None,
    };
    match SymmetricKey::<V4>::from(&bytes).ok() {
        Some(value) => Some(value),
        None => None,
    }
}

#[allow(dead_code)]
fn make_token_paseto(cfg: &AppConfig, claims: &Claims) -> Option<String> {
    let key = match paseto_key(cfg) {
        Some(value) => value,
        None => return None,
    };
    let mut pclaims = match PasetoClaims::new() {
        Ok(value) => value,
        Err(_) => return None,
    };
    if pclaims.issuer(&cfg.security.token_iss).is_err() {
        return None;
    }
    if pclaims.audience(&cfg.security.token_aud).is_err() {
        return None;
    }
    if pclaims.subject(&claims.sub).is_err() {
        return None;
    }
    let now = OffsetDateTime::now_utc();
    let iat_str = match now.format(&Rfc3339) {
        Ok(value) => value,
        Err(_) => return None,
    };
    if pclaims.issued_at(&iat_str).is_err() {
        return None;
    }
    let exp = now + TimeDuration::seconds(cfg.security.token_ttl_seconds as i64);
    let exp_str = match exp.format(&Rfc3339) {
        Ok(value) => value,
        Err(_) => return None,
    };
    if pclaims.expiration(&exp_str).is_err() {
        return None;
    }
    // Additional claims
    if pclaims
        .add_additional("email", serde_json::Value::String(claims.email.clone()))
        .is_err()
    {
        return None;
    }
    let roles_json = match serde_json::to_value(&claims.roles) {
        Ok(value) => value,
        Err(_) => return None,
    };
    if pclaims.add_additional("roles", roles_json).is_err() {
        return None;
    }
    match local::encrypt(&key, &pclaims, None, None) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

#[allow(dead_code)]
fn validate_token_paseto(cfg: &AppConfig, token: &str) -> Option<Claims> {
    let key = match paseto_key(cfg) {
        Some(value) => value,
        None => return None,
    };
    let utok = match UntrustedToken::try_from(token) {
        Ok(value) => value,
        Err(_) => return None,
    };
    let rules = ClaimsValidationRules::new();
    let trusted = match local::decrypt(&key, &utok, &rules, None, None) {
        Ok(value) => value,
        Err(_) => return None,
    };
    let pc = match trusted.payload_claims() {
        Some(value) => value,
        None => return None,
    };
    let iss = pc
        .get_claim("iss")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let aud = pc
        .get_claim("aud")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if iss != cfg.security.token_iss || aud != cfg.security.token_aud {
        return None;
    }
    let sub = pc
        .get_claim("sub")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    // Additional
    let email = pc
        .get_claim("email")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let roles = pc
        .get_claim("roles")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let permissions = pc
        .get_claim("permissions")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let access_level = pc
        .get_claim("access_level")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let tenant_id = pc
        .get_claim("tenant_id")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let subscription_tier = pc
        .get_claim("subscription_tier")
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    Some(Claims {
        sub,
        email,
        roles,
        permissions,
        access_level,
        subscription_tier,
        tenant_id,
        iss,
        aud,
        iat: 0,
        exp: 0,
    })
}

pub fn make_token(cfg: &AppConfig, claims: &Claims) -> Option<String> {
    match cfg.security.token_mode {
        TokenMode::JwtHmac => Some(make_token_hmac(cfg, claims)),
        TokenMode::PasetoV4Local => make_token_paseto(cfg, claims),
    }
}

pub fn validate_token(cfg: &AppConfig, token: &str) -> Option<Claims> {
    match cfg.security.token_mode {
        TokenMode::JwtHmac => validate_token_hmac(cfg, token),
        TokenMode::PasetoV4Local => validate_token_paseto(cfg, token),
    }
}

const LOGIN_WINDOW: StdDuration = StdDuration::from_secs(45);
const LOGIN_MAX_ATTEMPTS: usize = 8;

static LOGIN_ATTEMPTS: Lazy<Mutex<HashMap<String, VecDeque<Instant>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn record_login_attempt(ip: &str, cfg: &AppConfig) -> bool {
    // Skip rate limiting in debug mode or if rate limiting is disabled
    if cfg.security.debug_mode || !cfg.security.rate_limit_enabled {
        return false;
    }

    let mut attempts = LOGIN_ATTEMPTS.lock();
    let entry = attempts.entry(ip.to_string()).or_insert_with(VecDeque::new);
    let now = Instant::now();
    while let Some(front) = entry.front() {
        if now.duration_since(*front) > LOGIN_WINDOW {
            entry.pop_front();
        } else {
            break;
        }
    }
    entry.push_back(now);
    entry.len() > LOGIN_MAX_ATTEMPTS
}

fn looks_like_hash(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn normalized_email(value: &str) -> String {
    value.trim().to_lowercase()
}

pub(crate) fn hash_email(value: &str) -> String {
    sha256_hex(&normalized_email(value))
}

fn hash_secret(value: &str) -> String {
    sha256_hex(value.trim())
}

#[derive(Clone)]
pub struct TokenSnapshot {
    pub claims: Claims,
    pub expired: bool,
}

pub fn inspect_token(cfg: &AppConfig, token: &str) -> Option<TokenSnapshot> {
    if let Some(claims) = validate_token(cfg, token) {
        return Some(TokenSnapshot {
            claims,
            expired: false,
        });
    }

    match cfg.security.token_mode {
        TokenMode::JwtHmac => {
            let key = default_secret(cfg);
            if let Some((_h, payload)) = verify_hs256(&key, token) {
                if let Ok(claims) = serde_json::from_value::<Claims>(payload) {
                    let expired = claims.exp < Utc::now().timestamp();
                    return Some(TokenSnapshot { claims, expired });
                }
            }
        }
        TokenMode::PasetoV4Local => {
            let key = match paseto_key(cfg) {
                Some(value) => value,
                None => return None,
            };
            let utok = match UntrustedToken::try_from(token) {
                Ok(value) => value,
                Err(_) => return None,
            };
            let trusted =
                match local::decrypt(&key, &utok, &ClaimsValidationRules::new(), None, None) {
                    Ok(value) => value,
                    Err(_) => return None,
                };
            let payload = match trusted.payload_claims() {
                Some(value) => value,
                None => return None,
            };
            let email = payload
                .get_claim("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let roles = payload
                .get_claim("roles")
                .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
                .unwrap_or_default();
            let sub = payload
                .get_claim("sub")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let iss = payload
                .get_claim("iss")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let aud = payload
                .get_claim("aud")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let iat_ts = payload
                .get_claim("iat")
                .and_then(|v| v.as_str())
                .and_then(|ts| OffsetDateTime::parse(ts, &Rfc3339).ok())
                .map(|odt| odt.unix_timestamp())
                .unwrap_or(0);
            let exp_ts = payload
                .get_claim("exp")
                .and_then(|v| v.as_str())
                .and_then(|ts| OffsetDateTime::parse(ts, &Rfc3339).ok())
                .map(|odt| odt.unix_timestamp())
                .unwrap_or(0);
            let permissions = payload
                .get_claim("permissions")
                .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
                .unwrap_or_default();
            let access_level = payload
                .get_claim("access_level")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let tenant_id = payload
                .get_claim("tenant_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let subscription_tier = payload
                .get_claim("subscription_tier")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let claims = Claims {
                sub,
                email,
                roles,
                permissions,
                access_level,
                subscription_tier,
                tenant_id,
                iss,
                aud,
                iat: iat_ts,
                exp: exp_ts,
            };
            return Some(TokenSnapshot {
                expired: exp_ts < Utc::now().timestamp(),
                claims,
            });
        }
    }

    None
}

fn persist_user(db: &Database, user: &UserRecord) -> Result<(), actix_web::Error> {
    db.insert("users", &user.id, user)
        .map_err(|_| actix_web::error::ErrorInternalServerError("failed to persist user"))
}

fn ensure_user_email_hash(db: &Database, user: &mut UserRecord) -> Result<(), actix_web::Error> {
    if user.email_hash.is_none() {
        user.email_hash = Some(hash_email(&user.email));
        if let Err(err) = persist_user(db, user) {
            return Err(err);
        }
    }
    Ok(())
}

fn upgrade_user_password(
    db: &Database,
    user: &mut UserRecord,
    plain_password: &str,
) -> Result<(), actix_web::Error> {
    let prehash = hash_secret(plain_password);
    let salt = SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(prehash.as_bytes(), &salt) {
        Ok(value) => value.to_string(),
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("hash error")),
    };
    user.password_hash = hash;
    user.password_encoding = PasswordEncoding::Sha256ThenArgon2;
    if let Err(err) = ensure_user_email_hash(db, user) {
        return Err(err);
    }
    if let Err(err) = persist_user(db, user) {
        return Err(err);
    }
    Ok(())
}

fn verify_argon2(user: &UserRecord, candidate: &[u8]) -> Result<bool, actix_web::Error> {
    let parsed = match PasswordHash::new(&user.password_hash) {
        Ok(value) => value,
        Err(_) => {
            return Err(actix_web::error::ErrorInternalServerError(
                "hash read error",
            ))
        }
    };
    Ok(Argon2::default()
        .verify_password(candidate, &parsed)
        .is_ok())
}

fn verify_plain_password(user: &UserRecord, plain: &str) -> Result<bool, actix_web::Error> {
    let input = match user.password_encoding {
        PasswordEncoding::PlainArgon2 => plain.as_bytes().to_vec(),
        PasswordEncoding::Sha256ThenArgon2 => hash_secret(plain).into_bytes(),
    };
    verify_argon2(user, &input)
}

fn verify_hashed_password(user: &UserRecord, hashed_hex: &str) -> Result<bool, actix_web::Error> {
    if user.password_encoding != PasswordEncoding::Sha256ThenArgon2 {
        return Ok(false);
    }
    verify_argon2(user, hashed_hex.as_bytes())
}

fn find_user_by_email_hash(
    db: &Database,
    hashed: &str,
) -> Result<Option<UserRecord>, actix_web::Error> {
    let users: Vec<UserRecord> = match db.list("users").map_err(|e| {
        tracing::error!("Failed to list users from database: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    }) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    for mut user in users {
        let current_hash = match &user.email_hash {
            Some(existing) => existing.clone(),
            None => {
                user.email_hash = Some(hash_email(&user.email));
                if let Err(err) = persist_user(db, &user) {
                    return Err(err);
                }
                match user.email_hash.clone() {
                    Some(value) => value,
                    None => {
                        return Err(actix_web::error::ErrorInternalServerError(
                            "hash write error",
                        ))
                    }
                }
            }
        };
        if current_hash == hashed {
            return Ok(Some(user));
        }
    }
    Ok(None)
}

fn issue_login_response(
    cfg: &AppConfig,
    user: &UserRecord,
) -> Result<HttpResponse, actix_web::Error> {
    if !user.approved {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Account is awaiting approval",
            "code": "account_unapproved"
        })));
    }
    if !user.verified {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Account email is not verified",
            "code": "account_unverified"
        })));
    }
    if user.blocked {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Account is blocked",
            "code": "account_blocked"
        })));
    }
    if user.locked {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Account is locked",
            "code": "account_locked"
        })));
    }

    let now = Utc::now();
    let is_admin = user_is_admin(user);
    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        roles: user.roles.clone(),
        permissions: user.permissions.clone(),
        access_level: if is_admin {
            Some("admin".to_string())
        } else {
            Some("write".to_string())
        },
        subscription_tier: user.subscription_tier.clone(),
        tenant_id: None,
        iss: cfg.security.token_iss.clone(),
        aud: cfg.security.token_aud.clone(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(cfg.security.token_ttl_seconds as i64)).timestamp(),
    };
    let token = match make_token(cfg, &claims) {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorInternalServerError("token error")),
    };
    Ok(HttpResponse::Ok().json(json!({
        "token": token,
        "claims": claims,
        "user": {
            "id": user.id,
            "email": user.email,
            "roles": user.roles,
        }
    })))
}

fn access_denial_message(user: &UserRecord) -> Option<&'static str> {
    if user.blocked {
        return Some("Account is blocked. Contact an administrator.");
    }
    if user.locked {
        return Some("Account is locked. Contact an administrator.");
    }
    if !user.approved {
        return Some("Account pending approval.");
    }
    if !user.verified {
        return Some("Account pending verification.");
    }
    None
}

fn load_auth_settings(db: &Database) -> AuthSettings {
    db.get::<AuthSettings>("settings", "auth")
        .unwrap_or(None)
        .unwrap_or_default()
}

fn save_auth_settings(db: &Database, settings: &AuthSettings) -> Result<(), actix_web::Error> {
    db.insert("settings", "auth", settings)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to persist auth settings"))
}

fn load_invite(db: &Database, token: &str) -> Option<InviteToken> {
    db.get::<InviteToken>("invites", token).ok().flatten()
}

fn save_invite(db: &Database, invite: &InviteToken) -> Result<(), actix_web::Error> {
    db.insert("invites", &invite.token, invite)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to persist invite"))
}

fn validate_invite_token(db: &Database, token: &str) -> Option<InviteToken> {
    load_invite(db, token).and_then(|inv| if inv.can_be_used() { Some(inv) } else { None })
}

fn load_company_invite(db: &Database, token: &str) -> Option<CompanyInvite> {
    db.get::<CompanyInvite>("company_invites", token)
        .ok()
        .flatten()
}

fn save_company_invite(db: &Database, invite: &CompanyInvite) -> Result<(), actix_web::Error> {
    db.insert("company_invites", &invite.token, invite)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to persist invite"))
}

fn validate_company_invite(db: &Database, token: &str) -> Option<CompanyInvite> {
    load_company_invite(db, token).and_then(|inv| if inv.can_be_used() { Some(inv) } else { None })
}

fn attach_user_to_company(
    db: &Database,
    invite: &CompanyInvite,
    user_id: &str,
) -> Result<(), actix_web::Error> {
    let mut company = match db
        .get::<Company>("companies", &invite.company_id)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to load company"))
    {
        Ok(Some(value)) => value,
        Ok(None) => return Err(actix_web::error::ErrorNotFound("Company not found")),
        Err(err) => return Err(err),
    };

    let mut next_access = company.user_access.clone();
    next_access.push(CompanyUserAccess {
        user_id: user_id.to_string(),
        role: invite.role.clone(),
    });

    if let Err(err) = company.apply_update(UpdateCompanyRequest {
        user_access: Some(next_access),
        ..Default::default()
    }) {
        return Err(actix_web::error::ErrorBadRequest(err.message));
    }

    if let Err(err) = db
        .insert("companies", &company.id, &company)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to update company"))
    {
        return Err(err);
    }

    Ok(())
}

fn ensure_admin(req: &actix_web::HttpRequest) -> Result<Claims, actix_web::Error> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(_) => {
            return Err(actix_web::error::ErrorUnauthorized("Unauthorized"));
        }
    };
    if is_admin(&claims) {
        Ok(claims)
    } else {
        Err(actix_web::error::ErrorForbidden(
            "Admin privileges required",
        ))
    }
}

fn ensure_manage_system_settings(req: &actix_web::HttpRequest) -> Result<Claims, actix_web::Error> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(_) => {
            return Err(actix_web::error::ErrorUnauthorized("Unauthorized"));
        }
    };

    if can_manage_system_settings(&claims) {
        Ok(claims)
    } else {
        Err(actix_web::error::ErrorForbidden(
            "System settings privileges required",
        ))
    }
}

/// #feature[auth]
#[post("/register")]
pub async fn register(
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    use crate::validation as v;

    let email = body.email.trim().to_lowercase();

    // Strict email validation
    if let Err(e) = v::validate_email_strict(&email) {
        return Ok(HttpResponse::BadRequest().json(json!({"error": e})));
    }

    // Strong password validation
    if let Err(e) = v::password_strength(&body.password) {
        return Ok(HttpResponse::BadRequest().json(json!({"error": e})));
    }

    // Existing user?
    let users: Vec<UserRecord> = db.list("users").unwrap_or_default();
    if users.iter().any(|u| u.email == email) {
        return Ok(HttpResponse::Conflict().json(json!({"error": "Email already registered"})));
    }

    let has_existing_users = !users.is_empty();
    let provided_token = body.invite_token.clone().unwrap_or_default();
    let mut invite_to_mark: Option<InviteToken> = None;
    let mut company_invite_to_mark: Option<CompanyInvite> = None;
    if !provided_token.is_empty() {
        invite_to_mark = validate_invite_token(&db, &provided_token);
        company_invite_to_mark = validate_company_invite(&db, &provided_token);
        if invite_to_mark.is_none() && company_invite_to_mark.is_none() {
            return Ok(HttpResponse::Forbidden()
                .json(json!({"error": "Invite link has expired or was already used"})));
        }
        if let Some(invite) = company_invite_to_mark.as_ref() {
            if !invite.email_matches(&email) {
                return Ok(HttpResponse::Forbidden()
                    .json(json!({"error": "Invite does not match this account"})));
            }
        }
    }
    if has_existing_users {
        let settings = load_auth_settings(&db);
        if !settings.registration_open {
            if settings.invite_only {
                if provided_token.is_empty() {
                    return Ok(HttpResponse::Forbidden()
                        .json(json!({"error": settings.effective_message()})));
                }
                if invite_to_mark.is_none() && company_invite_to_mark.is_none() {
                    return Ok(HttpResponse::Forbidden()
                        .json(json!({"error": "Invite link has expired or was already used"})));
                }
            } else {
                return Ok(
                    HttpResponse::Forbidden().json(json!({"error": settings.effective_message()}))
                );
            }
        }
    }

    let prehash = hash_secret(&body.password);
    let salt = SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(prehash.as_bytes(), &salt) {
        Ok(value) => value.to_string(),
        Err(_) => return Err(actix_web::error::ErrorInternalServerError("hash error")),
    };
    let mut user = if users.is_empty() {
        UserRecord::new_admin(&email, hash)
    } else {
        UserRecord::new_user(&email, hash)
    };
    user.email_hash = Some(hash_email(&email));
    user.password_encoding = PasswordEncoding::Sha256ThenArgon2;
    if let Err(err) = db
        .insert("users", &user.id, &user)
        .map_err(|_| actix_web::error::ErrorInternalServerError("db error"))
    {
        return Err(err);
    }

    if let Some(mut invite) = invite_to_mark {
        invite.mark_used(&user.id);
        if let Err(err) = save_invite(&db, &invite) {
            warn!("Failed to mark invite {} as used: {}", invite.token, err);
        }
    }
    let mut company_invite_status: Option<serde_json::Value> = None;
    if let Some(mut invite) = company_invite_to_mark {
        let attach_result = attach_user_to_company(&db, &invite, &user.id);
        let invite_company_id = invite.company_id.clone();
        let invite_role = invite.role.clone();
        if let Err(ref err) = attach_result {
            warn!(
                "Failed to attach user {} to company {}: {}",
                user.id, invite_company_id, err
            );
        }
        let linked = attach_result.is_ok();
        company_invite_status = Some(json!({
            "company_id": invite_company_id,
            "role": invite_role,
            "linked": linked,
        }));
        invite.mark_used(&user.id);
        if let Err(err) = save_company_invite(&db, &invite) {
            warn!(
                "Failed to mark company invite {} as used: {}",
                invite.token, err
            );
        }
    }

    let now = Utc::now();
    let is_admin = user_is_admin(&user);
    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        roles: user.roles.clone(),
        permissions: user.permissions.clone(),
        access_level: if is_admin {
            Some("admin".to_string())
        } else {
            Some("write".to_string())
        },
        subscription_tier: user.subscription_tier.clone(),
        tenant_id: None,
        iss: cfg.security.token_iss.clone(),
        aud: cfg.security.token_aud.clone(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(cfg.security.token_ttl_seconds as i64)).timestamp(),
    };
    let token = match make_token(&cfg, &claims) {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorInternalServerError("token error")),
    };
    let mut response = json!({"token": token, "claims": claims, "user": user });
    if let Some(status) = company_invite_status {
        response["company_invite"] = status;
    }
    Ok(HttpResponse::Ok().json(response))
}

#[derive(serde::Deserialize)]
pub struct RegistrationStatusQuery {
    invite: Option<String>,
}

/// #feature[auth]
#[get("/registration-settings")]
pub async fn get_registration_settings(
    db: web::Data<Database>,
    query: web::Query<RegistrationStatusQuery>,
) -> Result<HttpResponse> {
    let settings = load_auth_settings(&db);
    let users: Vec<UserRecord> = db.list("users").unwrap_or_default();
    let has_users = !users.is_empty();
    let invite_token = query.invite.as_deref().filter(|token| !token.is_empty());
    let system_invite = invite_token.and_then(|token| validate_invite_token(&db, token));
    let mut company_invite = invite_token.and_then(|token| validate_company_invite(&db, token));
    let mut invite_company: Option<serde_json::Value> = None;

    if let Some(invite) = company_invite.as_ref() {
        let company = db
            .get::<Company>("companies", &invite.company_id)
            .ok()
            .flatten();
        if let Some(company) = company {
            invite_company = Some(json!({
                "id": company.id,
                "name": company.name,
                "role": invite.role.clone(),
                "email": invite.email.clone(),
            }));
        } else {
            company_invite = None;
        }
    }

    let invite_token_valid = system_invite.is_some() || company_invite.is_some();
    let invite_email = company_invite
        .as_ref()
        .and_then(|inv| inv.email.clone())
        .or_else(|| system_invite.as_ref().and_then(|inv| inv.email.clone()));
    let invite_type = if company_invite.is_some() {
        "company"
    } else if system_invite.is_some() {
        "system"
    } else {
        "none"
    };
    let invite_required = has_users && !settings.registration_open && settings.invite_only;
    let can_register = !has_users || settings.registration_open || invite_token_valid;

    Ok(HttpResponse::Ok().json(json!({
        "registration_open": settings.registration_open || !has_users,
        "registration_flag": settings.registration_open,
        "invite_only": settings.invite_only,
        "invite_required": invite_required,
        "invite_token_valid": invite_token_valid,
        "message": settings.invite_message,
        "has_users": has_users,
        "can_register": can_register,
        "invite_email": invite_email,
        "invite_type": invite_type,
        "invite_company": invite_company,
        "mode": if settings.invite_only {
            "invite_only"
        } else if settings.registration_open {
            "public"
        } else {
            "closed"
        },
    })))
}

#[derive(serde::Deserialize)]
pub struct UpdateRegistrationSettingsPayload {
    pub registration_open: bool,
    pub invite_only: bool,
    pub invite_message: Option<String>,
}

/// #feature[auth]
#[put("/registration-settings")]
pub async fn update_registration_settings(
    req: actix_web::HttpRequest,
    db: web::Data<Database>,
    body: web::Json<UpdateRegistrationSettingsPayload>,
) -> Result<HttpResponse> {
    let _claims = match ensure_manage_system_settings(&req) {
        Ok(claims) => claims,
        Err(err) => return Err(err),
    };

    let mut settings = load_auth_settings(&db);
    let payload = body.into_inner();
    settings.registration_open = payload.registration_open;
    settings.invite_only = payload.invite_only;
    if let Some(message) = payload.invite_message {
        let trimmed = message.trim();
        settings.invite_message = if trimmed.is_empty() {
            Some(AuthSettings::default().effective_message())
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Err(err) = save_auth_settings(&db, &settings) {
        return Err(err);
    }
    Ok(HttpResponse::Ok().json(settings))
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InviteRequest {
    pub email: Option<String>,
    pub expires_in_hours: Option<i64>,
}

/// #feature[teams]
#[post("/invites")]
pub async fn create_invite(
    req: actix_web::HttpRequest,
    db: web::Data<Database>,
    body: web::Json<InviteRequest>,
) -> Result<HttpResponse> {
    let claims = match ensure_admin(&req) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    let payload = body.into_inner();
    let ttl = payload.expires_in_hours.unwrap_or(24 * 7).clamp(1, 24 * 30);
    let email = payload
        .email
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty());

    let invite = InviteToken::new(&claims.sub, email, ttl);
    if let Err(err) = save_invite(&db, &invite) {
        return Err(err);
    }
    Ok(HttpResponse::Ok().json(&invite))
}

/// #feature[teams]
#[get("/invites")]
pub async fn list_invites(
    req: actix_web::HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let _claims = match ensure_admin(&req) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    let mut invites: Vec<InviteToken> = db.list("invites").unwrap_or_default();
    invites.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(HttpResponse::Ok().json(json!({ "invites": invites })))
}

/// #feature[teams]
#[delete("/invites/{token}")]
pub async fn revoke_invite(
    req: actix_web::HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let _claims = match ensure_admin(&req) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    let token = path.into_inner();
    let removed = match db
        .delete("invites", &token)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to revoke invite"))
    {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    if removed {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().json(json!({"error": "Invite not found"})))
    }
}

/// #feature[auth]
#[post("/login")]
pub async fn login(
    req: HttpRequest,
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    use crate::validation as v;

    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".into());

    let payload = body.into_inner();
    let attempt_fingerprint = payload
        .hashed_email
        .as_deref()
        .map(|h| h.to_string())
        .or_else(|| payload.email.as_deref().map(hash_email));

    if record_login_attempt(&client_ip, &cfg) {
        let hashed = attempt_fingerprint
            .clone()
            .unwrap_or_else(|| "unknown".into());
        tracing::warn!(
            target: "security",
            ip = %client_ip,
            hashed = %hashed,
            "possible login attack IP {} email login",
            client_ip
        );
        return Ok(HttpResponse::TooManyRequests()
            .json(json!({"error": "Too many login attempts, please slow down"})));
    }

    if let (Some(hashed_email_raw), Some(hashed_password_raw)) = (
        payload.hashed_email.as_deref(),
        payload.hashed_password.as_deref(),
    ) {
        let hashed_email = hashed_email_raw.trim().to_lowercase();
        let hashed_password = hashed_password_raw.trim().to_lowercase();

        if !looks_like_hash(&hashed_email) || !looks_like_hash(&hashed_password) {
            tracing::warn!(
                target: "security",
                ip = %client_ip,
                "login payload included non-hash values where hashes were required"
            );
            return Ok(
                HttpResponse::BadRequest().json(json!({"error": "Malformed hashed credentials"}))
            );
        }

        let user_opt = match find_user_by_email_hash(&db, &hashed_email) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        if let Some(mut user) = user_opt {
            if let Err(err) = ensure_user_email_hash(&db, &mut user) {
                return Err(err);
            }
            if user.password_encoding != PasswordEncoding::Sha256ThenArgon2 {
                tracing::warn!(
                    target: "security",
                    ip = %client_ip,
                    user_id = %user.id,
                    "hashed login attempted for legacy password strategy"
                );
                return Ok(HttpResponse::Conflict().json(json!({
                    "error": "legacy_credentials",
                    "upgrade_required": true
                })));
            }

            let verified = match verify_hashed_password(&user, &hashed_password) {
                Ok(value) => value,
                Err(err) => return Err(err),
            };
            if verified {
                if let Some(msg) = access_denial_message(&user) {
                    return Ok(HttpResponse::Forbidden().json(json!({ "error": msg })));
                }
                tracing::info!(
                    target: "security",
                    ip = %client_ip,
                    user_id = %user.id,
                    user_hash = %user.email_hash.clone().unwrap_or_default(),
                    "login success via hashed credentials"
                );
                return issue_login_response(&cfg, &user);
            } else {
                return Ok(HttpResponse::Unauthorized()
                    .json(json!({"error": "Invalid email or password"})));
            }
        } else {
            return Ok(
                HttpResponse::Unauthorized().json(json!({"error": "Invalid email or password"}))
            );
        }
    }

    let email = payload
        .email
        .as_deref()
        .map(normalized_email)
        .unwrap_or_default();
    let password = payload.password.unwrap_or_default();

    if email.is_empty() || password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Hashed credentials required"
        })));
    }

    tracing::warn!(
        target: "security",
        ip = %client_ip,
        "plain credentials received during login (legacy fallback path)"
    );

    if let Err(e) = v::validate_email_strict(&email) {
        return Ok(HttpResponse::BadRequest().json(json!({"error": e})));
    }

    if password.is_empty() || password.len() > 128 {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Invalid password"})));
    }

    let users: Vec<UserRecord> = match db.list("users").map_err(|e| {
        tracing::error!("Failed to list users from database: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    }) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    if let Some(mut user) = users.into_iter().find(|u| u.email == email) {
        let verified = match verify_plain_password(&user, &password) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        if verified {
            if let Some(msg) = access_denial_message(&user) {
                return Ok(HttpResponse::Forbidden().json(json!({ "error": msg })));
            }
            if user.password_encoding != PasswordEncoding::Sha256ThenArgon2 {
                if let Err(err) = upgrade_user_password(&db, &mut user, &password) {
                    return Err(err);
                }
            }
            tracing::info!(
                target: "security",
                ip = %client_ip,
                user_id = %user.id,
                user_hash = %user.email_hash.clone().unwrap_or_else(|| hash_email(&user.email)),
                "login success via legacy fallback (account upgraded)"
            );
            return issue_login_response(&cfg, &user);
        }
    }

    Ok(HttpResponse::Unauthorized().json(json!({"error": "Invalid email or password"})))
}

/// #feature[auth]
#[post("/logout")]
pub async fn logout(_db: web::Data<Database>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("logged out"))
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconfirmRequest {
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    hashed_password: Option<String>,
}

/// #feature[auth]
#[post("/reconfirm")]
pub async fn reconfirm(
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    req: actix_web::HttpRequest,
    body: web::Json<ReconfirmRequest>,
) -> Result<HttpResponse> {
    // Extract token from request (Bearer token or query param)
    let token = match extract_token(&req) {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorUnauthorized("No token provided")),
    };

    // Validate token - we accept expired tokens to allow reconfirmation
    // Try to decode the token even if expired to get user ID
    let user_id = match cfg.security.token_mode {
        TokenMode::JwtHmac => {
            let key = default_secret(&cfg);
            if let Some((_h, p)) = verify_hs256(&key, &token) {
                let claims: Claims = match serde_json::from_value(p) {
                    Ok(value) => value,
                    Err(_) => {
                        return Err(actix_web::error::ErrorUnauthorized("Invalid token format"))
                    }
                };
                // Don't check expiration - we want to allow expired tokens
                claims.sub
            } else {
                return Ok(HttpResponse::Unauthorized().json(json!({"error": "Invalid token"})));
            }
        }
        TokenMode::PasetoV4Local => {
            let key = match paseto_key(&cfg) {
                Some(value) => value,
                None => {
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Paseto key not configured",
                    ))
                }
            };
            let utok = match UntrustedToken::try_from(token.as_str()) {
                Ok(value) => value,
                Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid token format")),
            };
            // Decrypt without expiration validation
            let trusted =
                match local::decrypt(&key, &utok, &ClaimsValidationRules::new(), None, None) {
                    Ok(value) => value,
                    Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid token")),
                };
            let pc = match trusted.payload_claims() {
                Some(value) => value,
                None => return Err(actix_web::error::ErrorUnauthorized("Invalid token claims")),
            };
            match pc.get_claim("sub").and_then(|v| v.as_str()) {
                Some(value) => value.to_string(),
                None => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Missing user ID in token",
                    ))
                }
            }
        }
    };

    // Find user in database
    let users: Vec<UserRecord> = db.list("users").unwrap_or_default();
    let mut user = match users.into_iter().find(|u| u.id == user_id) {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorUnauthorized("User not found")),
    };

    if let Some(hashed) = body
        .hashed_password
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !looks_like_hash(hashed) {
            return Ok(
                HttpResponse::BadRequest().json(json!({"error": "Malformed hashed credentials"}))
            );
        }
        if user.password_encoding != PasswordEncoding::Sha256ThenArgon2 {
            return Ok(HttpResponse::Conflict().json(json!({
                "error": "legacy_credentials",
                "upgrade_required": true
            })));
        }
        let verified = match verify_hashed_password(&user, hashed) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        if !verified {
            return Ok(HttpResponse::Unauthorized().json(json!({"error": "Invalid password"})));
        }
        if let Some(msg) = access_denial_message(&user) {
            return Ok(HttpResponse::Forbidden().json(json!({ "error": msg })));
        }
        return issue_login_response(&cfg, &user);
    }

    let plain = match body
        .password
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(value) => value,
        None => return Err(actix_web::error::ErrorBadRequest("Password required")),
    };

    let verified = match verify_plain_password(&user, plain) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    if !verified {
        return Ok(HttpResponse::Unauthorized().json(json!({"error": "Invalid password"})));
    }

    if user.password_encoding != PasswordEncoding::Sha256ThenArgon2 {
        if let Err(err) = upgrade_user_password(&db, &mut user, plain) {
            return Err(err);
        }
    }

    if let Some(msg) = access_denial_message(&user) {
        return Ok(HttpResponse::Forbidden().json(json!({ "error": msg })));
    }
    issue_login_response(&cfg, &user)
}

/// #feature[auth]
#[get("/me")]
pub async fn me(
    db: web::Data<Database>,
    cfg: web::Data<AppConfig>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse> {
    if let Some(tok) = extract_token(&req) {
        if let Some(claims) = validate_token(&cfg, &tok) {
            if let Ok(Some(user)) = db.get::<UserRecord>("users", &claims.sub) {
                if let Some(msg) = access_denial_message(&user) {
                    return Ok(HttpResponse::Forbidden().json(json!({ "error": msg })));
                }
            }
            return Ok(HttpResponse::Ok().json(claims));
        }
    }
    Ok(HttpResponse::Unauthorized().json("unauthorized"))
}

use actix_web::body::BoxBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};

pub async fn guard_api(
    req: ServiceRequest,
    next: actix_web::middleware::Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    // Get config
    let cfg = req.app_data::<web::Data<AppConfig>>().cloned();
    if let Some(cfg) = cfg {
        if let Some(tok) = extract_token(req.request()) {
            // if let Some(tok) = extract_bearer_or_query(req.request()) {
            if let Some(claims) = validate_token(&cfg, &tok) {
                if let Some(db) = req.app_data::<web::Data<Database>>().cloned() {
                    if let Ok(Some(user)) = db.get::<UserRecord>("users", &claims.sub) {
                        if let Some(msg) = access_denial_message(&user) {
                            let (req, _pl) = req.into_parts();
                            let resp = HttpResponse::Forbidden().json(json!({ "error": msg }));
                            return Ok(ServiceResponse::new(req, resp.map_into_boxed_body()));
                        }
                    }
                }
                req.extensions_mut().insert(claims);
                return next.call(req).await;
            }
        }
    }
    let (req, _pl) = req.into_parts();
    let resp = HttpResponse::Unauthorized().json("unauthorized");
    Ok(ServiceResponse::new(req, resp.map_into_boxed_body()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AppConfig, DatabaseConfig, LoggingConfig, SecurityConfig, ServerConfig, TokenMode,
    };
    use actix_web::{test, web, App};
    use tempfile::tempdir;

    fn make_test_config(mode: TokenMode) -> AppConfig {
        // Create a minimal test config focused on security settings
        AppConfig {
            security: SecurityConfig {
                access_token: "test_access".into(),
                rate_limit_enabled: false,
                rate_limit_rpm: 100,
                auth_token_expiry_hours: 24,
                token_iss: "test_iss".into(),
                token_aud: "test_aud".into(),
                token_ttl_seconds: 3600,
                paseto_v4_local_key_hex:
                    "142f46b1b4acb0946e0d9413f29b331db345cf664b9307165eab7531fa32d8bd".into(),
                token_mode: mode,
                debug_mode: false,
                health_check_enabled: true,
                metrics_enabled: false,
            },
            server: ServerConfig {
                host: "localhost".into(),
                port: 8061,
                name: "test".into(),
            },
            database: DatabaseConfig {
                host: "localhost".into(),
                port: 5432,
                database: "test".into(),
                username: "test".into(),
                password: "test".into(),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 3600,
                ssl_mode: "prefer".into(),
            },
            sled_path: "test.db".into(),
            backup_dir: "backups".into(),
            backup_name_template: "backup_{{timestamp}}".into(),
            uploads_path: "./uploads".into(),
            backup_interval: None,
            backup_retention: 10,
            pg_conns: vec![],
            cors_rules: vec![],
            logging: LoggingConfig {
                level: "info".into(),
                file_enabled: false,
                file_path: None,
            },
            database_sync_on: false,
        }
    }

    fn make_test_claims() -> Claims {
        Claims {
            sub: "test_user".into(),
            email: "test@example.com".into(),
            roles: vec!["user".into()],
            permissions: vec![],
            access_level: Some("write".into()),
            subscription_tier: Some("pro".into()),
            tenant_id: None,
            iss: "test_iss".into(),
            aud: "test_aud".into(),
            iat: chrono::Utc::now().timestamp(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        }
    }

    #[test]
    async fn test_token_roundtrip_hmac() {
        let cfg = make_test_config(TokenMode::JwtHmac);
        let claims = make_test_claims();
        let token = make_token(&cfg, &claims).expect("make token");
        let decoded = validate_token(&cfg, &token).expect("validate token");
        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
        assert_eq!(decoded.roles, claims.roles);
    }

    #[test]
    async fn test_token_roundtrip_paseto() {
        let cfg = make_test_config(TokenMode::PasetoV4Local);
        let claims = make_test_claims();
        let token = make_token(&cfg, &claims).expect("make token");
        let decoded = validate_token(&cfg, &token).expect("validate token");
        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
        assert_eq!(decoded.roles, claims.roles);
    }

    #[test]
    async fn test_static_access_token() {
        let cfg = make_test_config(TokenMode::JwtHmac);
        let decoded =
            validate_token(&cfg, &cfg.security.access_token).expect("validate static token");
        assert_eq!(decoded.sub, "access");
        assert_eq!(decoded.roles, vec!["admin"]);
    }

    #[test]
    async fn test_hmac_token_rejects_tampered() {
        let cfg = make_test_config(TokenMode::JwtHmac);
        let claims = make_test_claims();
        let mut token = make_token(&cfg, &claims).expect("make token");
        token.push('x'); // tamper
        assert!(validate_token(&cfg, &token).is_none());
    }

    #[actix_web::test]
    async fn e2e_register_login_logout_me() {
        let dir = tempdir().unwrap();
        let mut cfg = make_test_config(TokenMode::JwtHmac);
        cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
        let db = Database::new(&cfg.sled_path).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(cfg.clone()))
                .service(register)
                .service(login)
                .service(logout)
                .service(me),
        )
        .await;

        let reg = RegisterRequest {
            email: "user1@test.dev".into(),
            password: "SecurePass123!".into(),
            invite_token: None,
        };
        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(&reg)
            .to_request();
        let reg_resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        let token = reg_resp["token"].as_str().unwrap().to_string();

        let me_req = test::TestRequest::get()
            .uri("/me")
            .insert_header(("authorization", format!("Bearer {}", token)))
            .to_request();
        let me_resp: serde_json::Value = test::call_and_read_body_json(&app, me_req).await;
        assert_eq!(me_resp["email"], "user1@test.dev");

        let login_req = LoginRequest {
            email: None,
            password: None,
            hashed_email: Some(hash_email("user1@test.dev")),
            hashed_password: Some(hash_secret("SecurePass123!")),
        };
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&login_req)
            .to_request();
        let login_resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert!(login_resp.get("token").is_some());

        let req = test::TestRequest::post().uri("/logout").to_request();
        let logout_resp = test::call_and_read_body(&app, req).await;
        assert!(std::str::from_utf8(&logout_resp)
            .unwrap()
            .contains("logged out"));
    }

    #[actix_web::test]
    async fn hashed_login_requires_upgrade_for_legacy_accounts() {
        use actix_web::{test, App};

        let dir = tempdir().unwrap();
        let mut cfg = make_test_config(TokenMode::JwtHmac);
        cfg.sled_path = dir.path().join("legacy").to_string_lossy().to_string();
        let db = Database::new(&cfg.sled_path).unwrap();

        let salt = SaltString::generate(&mut OsRng);
        let legacy_hash = Argon2::default()
            .hash_password("LegacyPass123!".as_bytes(), &salt)
            .unwrap()
            .to_string();
        let legacy = UserRecord::new_admin("legacy@example.com", legacy_hash);
        db.insert("users", &legacy.id, &legacy).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(cfg.clone()))
                .service(login),
        )
        .await;

        let hashed_req = LoginRequest {
            email: None,
            password: None,
            hashed_email: Some(hash_email("legacy@example.com")),
            hashed_password: Some(hash_secret("LegacyPass123!")),
        };
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&hashed_req)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CONFLICT);

        let fallback_req = LoginRequest {
            email: Some("legacy@example.com".into()),
            password: Some("LegacyPass123!".into()),
            hashed_email: None,
            hashed_password: None,
        };
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&fallback_req)
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert!(body.get("token").is_some());

        let users: Vec<UserRecord> = db.list("users").unwrap();
        let upgraded = users
            .into_iter()
            .find(|u| u.email == "legacy@example.com")
            .unwrap();
        assert_eq!(
            upgraded.password_encoding,
            PasswordEncoding::Sha256ThenArgon2
        );
        assert!(upgraded.email_hash.is_some());
    }

    #[actix_web::test]
    async fn hash_secret_matches_sha256_hex() {
        use sha2::{Digest, Sha256};
        let sample = "MySecret123!";
        let expected = hex::encode(Sha256::digest(sample.trim().as_bytes()));
        assert_eq!(hash_secret(sample), expected);
    }

    #[actix_web::test]
    async fn guard_accepts_bearer_and_query() {
        use actix_web::web;
        use actix_web::{test, App, HttpRequest, HttpResponse};

        async fn protected(req: HttpRequest) -> HttpResponse {
            let claims = req.extensions().get::<Claims>().cloned();
            match claims {
                Some(c) => HttpResponse::Ok().json(serde_json::json!({"sub": c.sub})),
                None => HttpResponse::Unauthorized().finish(),
            }
        }

        let cfg = make_test_config(TokenMode::JwtHmac);
        let claims = make_test_claims();
        let token = make_token(&cfg, &claims).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(cfg.clone()))
                .wrap(actix_web::middleware::from_fn(guard_api))
                .route("/p", web::get().to(protected)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/p")
            .insert_header(("authorization", format!("Bearer {}", token)))
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp["sub"], claims.sub);

        let req = test::TestRequest::get()
            .uri(&format!("/p?access_token={}", token))
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp["sub"], claims.sub);
    }
}
