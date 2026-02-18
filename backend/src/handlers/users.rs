use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Result};
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use rand_core::OsRng;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::authz::{is_super_admin, require_admin};
use crate::db::Database;
use crate::handlers::auth::hash_email;
use crate::handlers::errors::ApiError;
use crate::models::auth_types::{PasswordEncoding, UserRecord};
use crate::time::now;

const ALLOWED_ROLES: [&str; 7] = [
    "user",
    "admin",
    "super_admin",
    "superuser",
    "super_user",
    "user_manager",
    "settings_manager",
];

fn normalized_email(value: &str) -> String {
    value.trim().to_lowercase()
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn hash_secret(value: &str) -> String {
    sha256_hex(value.trim())
}

fn hash_password(plain: &str) -> Result<String, actix_web::Error> {
    let prehash = hash_secret(plain);
    let salt = SaltString::generate(&mut OsRng);
    match Argon2::default().hash_password(prehash.as_bytes(), &salt) {
        Ok(value) => Ok(value.to_string()),
        Err(_) => Err(actix_web::error::ErrorInternalServerError("hash error")),
    }
}

fn roles_from_request(roles: Option<Vec<String>>, role: Option<String>) -> Vec<String> {
    if let Some(list) = roles {
        return list
            .into_iter()
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
            .collect();
    }
    role.map(|r| vec![r.trim().to_string()]).unwrap_or_default()
}

fn validate_roles(roles: &[String], errors: &mut HashMap<String, String>) {
    if roles.is_empty() {
        return;
    }
    for role in roles {
        if !ALLOWED_ROLES.contains(&role.as_str()) {
            errors.insert("roles".into(), format!("unsupported role: {}", role));
        }
    }
}

fn user_payload(user: &UserRecord) -> serde_json::Value {
    json!({
        "id": user.id,
        "email": user.email,
        "full_name": user.full_name,
        "roles": user.roles,
        "permissions": user.permissions,
        "created_at": user.created_at,
        "subscription_tier": user.subscription_tier,
        "mfa_enabled": user.mfa_enabled,
        "approved": user.approved,
        "approved_at": user.approved_at,
        "approved_by": user.approved_by,
        "verified": user.verified,
        "verified_at": user.verified_at,
        "verified_by": user.verified_by,
        "blocked": user.blocked,
        "blocked_at": user.blocked_at,
        "blocked_by": user.blocked_by,
        "locked": user.locked,
        "locked_at": user.locked_at,
        "locked_by": user.locked_by,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub roles: Option<Vec<String>>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    #[serde(default)]
    pub subscription_tier: Option<String>,
    #[serde(default)]
    pub approved: Option<bool>,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub blocked: Option<bool>,
    #[serde(default)]
    pub locked: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserRequest {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub roles: Option<Vec<String>>,
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    #[serde(default)]
    pub subscription_tier: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub approved: Option<bool>,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub blocked: Option<bool>,
    #[serde(default)]
    pub locked: Option<bool>,
}

/// #feature[users]
#[get("/users")]
pub async fn list_users(req: HttpRequest, db: web::Data<Database>) -> Result<HttpResponse> {
    let _claims = match require_admin(&req) {
        Ok(value) => value,
        Err(resp) => return Ok(resp),
    };

    let users: Vec<UserRecord> = match db.list("users") {
        Ok(value) => value,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "USERS_LIST_FAILED",
                err,
                false,
            )))
        }
    };

    let payload: Vec<serde_json::Value> = users.into_iter().map(|u| user_payload(&u)).collect();
    Ok(HttpResponse::Ok().json(payload))
}

/// #feature[users]
#[post("/users")]
pub async fn create_user(
    req: HttpRequest,
    db: web::Data<Database>,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    let claims = match require_admin(&req) {
        Ok(value) => value,
        Err(resp) => return Ok(resp),
    };

    let mut errors: HashMap<String, String> = HashMap::new();
    let email = normalized_email(&body.email);
    if email.is_empty() {
        errors.insert("email".into(), "email is required".into());
    }
    if body.password.trim().len() < 8 {
        errors.insert(
            "password".into(),
            "password must be at least 8 characters".into(),
        );
    }

    let mut roles = roles_from_request(body.roles.clone(), body.role.clone());
    if roles.is_empty() {
        roles.push("user".to_string());
    }
    validate_roles(&roles, &mut errors);

    if !is_super_admin(&claims) && roles.iter().any(|r| r == "super_admin") {
        return Ok(HttpResponse::Forbidden().json(ApiError::forbidden()));
    }
    if body.permissions.is_some() && !is_super_admin(&claims) {
        return Ok(HttpResponse::Forbidden().json(ApiError::forbidden()));
    }

    let users: Vec<UserRecord> = match db.list("users") {
        Ok(value) => value,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "USERS_LIST_FAILED",
                err,
                false,
            )))
        }
    };

    if users.iter().any(|u| u.email == email) {
        errors.insert("email".into(), "email already exists".into());
    }

    if !errors.is_empty() {
        return Ok(
            HttpResponse::UnprocessableEntity().json(ApiError::validation(
                "User validation failed".into(),
                errors,
            )),
        );
    }

    let hash = match hash_password(&body.password) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };

    let mut user = UserRecord::new_user(&email, hash);
    user.email_hash = Some(hash_email(&email));
    user.password_encoding = PasswordEncoding::Sha256ThenArgon2;
    user.full_name = body.full_name.clone();
    user.subscription_tier = body.subscription_tier.clone();
    user.roles = roles;

    if user.roles.iter().any(|r| r == "super_admin") {
        user.permissions = vec!["*".into()];
    } else if let Some(perms) = body.permissions.clone() {
        user.permissions = perms;
    }

    let approved = body.approved.unwrap_or(true);
    let verified = body.verified.unwrap_or(true);
    user.approved = approved;
    if approved {
        user.approved_at = Some(now());
        user.approved_by = Some(claims.sub.clone());
    } else {
        user.approved_at = None;
        user.approved_by = None;
    }
    user.verified = verified;
    if verified {
        user.verified_at = Some(now());
        user.verified_by = Some(claims.sub.clone());
    } else {
        user.verified_at = None;
        user.verified_by = None;
    }

    let blocked = body.blocked.unwrap_or(false);
    let locked = body.locked.unwrap_or(false);
    user.blocked = blocked;
    if blocked {
        user.blocked_at = Some(now());
        user.blocked_by = Some(claims.sub.clone());
    } else {
        user.blocked_at = None;
        user.blocked_by = None;
    }
    user.locked = locked;
    if locked {
        user.locked_at = Some(now());
        user.locked_by = Some(claims.sub.clone());
    } else {
        user.locked_at = None;
        user.locked_by = None;
    }

    if let Err(err) = db
        .insert("users", &user.id, &user)
        .map_err(|_| actix_web::error::ErrorInternalServerError("db error"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Created().json(user_payload(&user)))
}

/// #feature[users]
#[put("/users/{id}")]
pub async fn update_user(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
    body: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    let claims = match require_admin(&req) {
        Ok(value) => value,
        Err(resp) => return Ok(resp),
    };

    let user_id = path.into_inner();
    let mut users: Vec<UserRecord> = match db.list("users") {
        Ok(value) => value,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "USERS_LIST_FAILED",
                err,
                false,
            )))
        }
    };

    let user_idx = match users.iter().position(|u| u.id == user_id) {
        Some(idx) => idx,
        None => return Ok(HttpResponse::NotFound().json(ApiError::not_found("User"))),
    };

    let mut user = users.remove(user_idx);

    if user.roles.iter().any(|r| r == "super_admin") && !is_super_admin(&claims) {
        return Ok(HttpResponse::Forbidden().json(ApiError::forbidden()));
    }

    if user.id == claims.sub && (body.blocked.unwrap_or(false) || body.locked.unwrap_or(false)) {
        return Ok(HttpResponse::BadRequest().json(ApiError::bad_request(
            "You cannot lock or block your own account",
        )));
    }

    let mut errors: HashMap<String, String> = HashMap::new();
    let roles = roles_from_request(body.roles.clone(), body.role.clone());
    if !roles.is_empty() {
        validate_roles(&roles, &mut errors);
        if roles.iter().any(|r| r == "super_admin") && !is_super_admin(&claims) {
            return Ok(HttpResponse::Forbidden().json(ApiError::forbidden()));
        }
    }

    if let Some(email) = body.email.as_ref() {
        let normalized = normalized_email(email);
        if normalized.is_empty() {
            errors.insert("email".into(), "email is required".into());
        } else if users
            .iter()
            .any(|u| u.email == normalized && u.id != user.id)
        {
            errors.insert("email".into(), "email already exists".into());
        }
    }

    if let Some(password) = body.password.as_ref() {
        if password.trim().len() < 8 {
            errors.insert(
                "password".into(),
                "password must be at least 8 characters".into(),
            );
        }
    }

    if !errors.is_empty() {
        return Ok(
            HttpResponse::UnprocessableEntity().json(ApiError::validation(
                "User validation failed".into(),
                errors,
            )),
        );
    }

    if let Some(email) = body.email.as_ref() {
        let normalized = normalized_email(email);
        user.email = normalized.clone();
        user.email_hash = Some(hash_email(&normalized));
    }

    if let Some(full_name) = body.full_name.as_ref() {
        user.full_name = Some(full_name.clone());
    }

    if let Some(password) = body.password.as_ref() {
        let hash = match hash_password(password) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        user.password_hash = hash;
        user.password_encoding = PasswordEncoding::Sha256ThenArgon2;
    }

    if let Some(tier) = body.subscription_tier.as_ref() {
        user.subscription_tier = Some(tier.clone());
    }

    if !roles.is_empty() {
        user.roles = roles;
        if user.roles.iter().any(|r| r == "super_admin") {
            if !user.permissions.contains(&"*".to_string()) {
                user.permissions.push("*".to_string());
            }
        } else {
            user.permissions.retain(|p| p != "*");
        }
    }

    if let Some(perms) = body.permissions.as_ref() {
        if !is_super_admin(&claims) {
            return Ok(HttpResponse::Forbidden().json(ApiError::forbidden()));
        }
        user.permissions = perms.clone();
    }

    if let Some(approved) = body.approved {
        user.approved = approved;
        if approved {
            user.approved_at = Some(now());
            user.approved_by = Some(claims.sub.clone());
        } else {
            user.approved_at = None;
            user.approved_by = None;
        }
    }

    if let Some(verified) = body.verified {
        user.verified = verified;
        if verified {
            user.verified_at = Some(now());
            user.verified_by = Some(claims.sub.clone());
        } else {
            user.verified_at = None;
            user.verified_by = None;
        }
    }

    if let Some(blocked) = body.blocked {
        user.blocked = blocked;
        if blocked {
            user.blocked_at = Some(now());
            user.blocked_by = Some(claims.sub.clone());
        } else {
            user.blocked_at = None;
            user.blocked_by = None;
        }
    }

    if let Some(locked) = body.locked {
        user.locked = locked;
        if locked {
            user.locked_at = Some(now());
            user.locked_by = Some(claims.sub.clone());
        } else {
            user.locked_at = None;
            user.locked_by = None;
        }
    }

    if let Err(err) = db
        .insert("users", &user.id, &user)
        .map_err(|_| actix_web::error::ErrorInternalServerError("db error"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(user_payload(&user)))
}

/// #feature[users]
#[delete("/users/{id}")]
pub async fn delete_user(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let claims = match require_admin(&req) {
        Ok(value) => value,
        Err(resp) => return Ok(resp),
    };

    let user_id = path.into_inner();
    if user_id == claims.sub {
        return Ok(HttpResponse::BadRequest()
            .json(ApiError::bad_request("You cannot delete your own account")));
    }

    let users: Vec<UserRecord> = match db.list("users") {
        Ok(value) => value,
        Err(err) => {
            return Ok(HttpResponse::InternalServerError().json(ApiError::internal(
                "USERS_LIST_FAILED",
                err,
                false,
            )))
        }
    };

    let target = match users.iter().find(|u| u.id == user_id) {
        Some(user) => user,
        None => return Ok(HttpResponse::NotFound().json(ApiError::not_found("User"))),
    };

    if target.roles.iter().any(|r| r == "super_admin") && !is_super_admin(&claims) {
        return Ok(HttpResponse::Forbidden().json(ApiError::forbidden()));
    }

    if target.roles.iter().any(|r| r == "super_admin") {
        let super_admin_count = users
            .iter()
            .filter(|u| u.roles.iter().any(|r| r == "super_admin"))
            .count();
        if super_admin_count <= 1 {
            return Ok(HttpResponse::BadRequest()
                .json(ApiError::bad_request("Cannot delete the last super admin")));
        }
    }

    if let Err(err) = db
        .delete("users", &user_id)
        .map_err(|_| actix_web::error::ErrorInternalServerError("db error"))
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(json!({ "status": "deleted" })))
}
