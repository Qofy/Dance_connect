use actix_web::{HttpMessage, HttpRequest, HttpResponse};

use crate::handlers::errors::ApiError;
use crate::models::{Claims, Company, UserRecord};

const ADMIN_ROLES: [&str; 4] = ["admin", "super_admin", "superuser", "super_user"];
const SUPER_ADMIN_ROLES: [&str; 3] = ["super_admin", "superuser", "super_user"];
const SETTINGS_ROLES: [&str; 5] = [
    "admin",
    "super_admin",
    "settings_manager",
    "superuser",
    "super_user",
];
const COMPANY_WRITE_ROLES: [&str; 2] = ["owner", "manager"];

pub fn require_claims(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    match req.extensions().get::<Claims>().cloned() {
        Some(claims) => Ok(claims),
        None => Err(HttpResponse::Unauthorized().json(ApiError::unauthorized("Unauthorized"))),
    }
}

pub fn require_user_id(req: &HttpRequest) -> Result<String, HttpResponse> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(resp) => return Err(resp),
    };
    Ok(claims.sub)
}

pub fn has_role(roles: &[String], role: &str) -> bool {
    for item in roles {
        if item == role {
            return true;
        }
    }
    false
}

pub fn has_any_role(roles: &[String], allowed: &[&str]) -> bool {
    for role in allowed {
        if has_role(roles, role) {
            return true;
        }
    }
    false
}

pub fn has_permission(permissions: &[String], perm: &str) -> bool {
    if perm.is_empty() {
        return false;
    }
    for item in permissions {
        if item == "*" || item == perm {
            return true;
        }
    }
    false
}

pub fn claims_has_permission(claims: &Claims, perm: &str) -> bool {
    has_permission(&claims.permissions, perm)
}

pub fn claims_has_any_permission(claims: &Claims, perms: &[&str]) -> bool {
    for perm in perms {
        if has_permission(&claims.permissions, perm) {
            return true;
        }
    }
    false
}

pub fn is_super_admin(claims: &Claims) -> bool {
    has_any_role(&claims.roles, &SUPER_ADMIN_ROLES) || has_permission(&claims.permissions, "*")
}

pub fn is_admin(claims: &Claims) -> bool {
    has_any_role(&claims.roles, &ADMIN_ROLES) || has_permission(&claims.permissions, "*")
}

pub fn user_is_admin(user: &UserRecord) -> bool {
    has_any_role(&user.roles, &ADMIN_ROLES) || has_permission(&user.permissions, "*")
}

pub fn require_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(resp) => return Err(resp),
    };
    if is_admin(&claims) {
        Ok(claims)
    } else {
        Err(HttpResponse::Forbidden().json(ApiError::forbidden()))
    }
}

pub fn require_super_admin(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(resp) => return Err(resp),
    };
    if is_super_admin(&claims) {
        Ok(claims)
    } else {
        Err(HttpResponse::Forbidden().json(ApiError::forbidden()))
    }
}

pub fn require_manage_system_settings(req: &HttpRequest) -> Result<Claims, HttpResponse> {
    let claims = match require_claims(req) {
        Ok(value) => value,
        Err(resp) => return Err(resp),
    };
    if can_manage_system_settings(&claims) {
        Ok(claims)
    } else {
        Err(HttpResponse::Forbidden().json(ApiError::forbidden()))
    }
}

pub fn can_manage_system_settings(claims: &Claims) -> bool {
    has_any_role(&claims.roles, &SETTINGS_ROLES)
        || claims_has_any_permission(claims, &["*", "manage_system_settings"])
}

pub fn has_company_access(company: &Company, claims: &Claims) -> bool {
    if is_admin(claims) {
        return true;
    }
    if company.user_id == claims.sub {
        return true;
    }
    for access in company.user_access.iter() {
        if access.user_id == claims.sub {
            return true;
        }
    }
    false
}

pub fn has_company_write_access(company: &Company, claims: &Claims) -> bool {
    if is_admin(claims) {
        return true;
    }
    if company.user_id == claims.sub {
        return true;
    }
    for access in company.user_access.iter() {
        if access.user_id == claims.sub && COMPANY_WRITE_ROLES.contains(&access.role.as_str()) {
            return true;
        }
    }
    false
}
