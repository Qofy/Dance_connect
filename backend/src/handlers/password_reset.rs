// handlers/password_reset.rs
use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand_core::OsRng;

use crate::authz::require_admin;
use crate::{
    db::Database,
    models::{
        auth_types::UserRecord,
        password_reset::{
            ForgotPasswordRequest, ForgotPasswordResponse, PasswordResetRequest,
            ResetPasswordRequest, ResetRequestStatus,
        },
    },
};

// User submits forgot password request
/// #feature[auth]
#[post("/forgot-password")]
pub async fn forgot_password(
    req: HttpRequest,
    db: web::Data<Database>,
    body: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse> {
    let email = body.email.trim().to_lowercase();

    // Check if user exists - users are stored by ID, not email, so we need to list and search
    let users: Vec<UserRecord> = db.list("users").unwrap_or_default();
    let user_exists = users.iter().any(|u| u.email == email);

    // Always create a request entry for consistency (even if user doesn't exist)
    // This prevents email enumeration attacks and ensures consistent API behavior
    let reset_request = if user_exists {
        let ip_address = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        PasswordResetRequest::new(email.clone(), ip_address, user_agent)
    } else {
        // Create a dummy request for non-existent users (security best practice)
        // This doesn't get saved to DB, just used for response
        PasswordResetRequest::new(email.clone(), None, None)
    };

    let request_id = reset_request.id.clone();

    // Save to database only if user exists (for real requests)
    if user_exists {
        if let Err(err) = db
            .insert("password_reset_requests", &request_id, &reset_request)
            .map_err(actix_web::error::ErrorInternalServerError)
        {
            return Err(err);
        }
    }

    // Always return same response for security (prevent email enumeration)
    Ok(HttpResponse::Ok().json(ForgotPasswordResponse {
        message: "If that email exists in our system, a password reset request has been created. Please contact an administrator to approve your request.".to_string(),
        request_id,
    }))
}

// Manager lists all password reset requests
/// #feature[auth]
#[get("/password-reset-requests")]
pub async fn list_reset_requests(
    req: HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    // Check if user is authenticated and is admin/super_admin
    let _claims = match require_admin(&req) {
        Ok(claims) => claims,
        Err(e) => return Ok(e),
    };

    // Get all reset requests
    let all_requests: Vec<PasswordResetRequest> = match db.list("password_reset_requests") {
        Ok(requests) => requests,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(format!("Database error: {}", e)))
        }
    };

    // Filter out used requests
    let requests: Vec<PasswordResetRequest> = all_requests
        .into_iter()
        .filter(|req| req.status != ResetRequestStatus::Used)
        .collect();

    Ok(HttpResponse::Ok().json(requests))
}

// Manager approves a reset request
/// #feature[auth]
#[post("/password-reset-requests/{id}/approve")]
pub async fn approve_reset_request(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // Check if user is authenticated and is admin/super_admin
    let claims = match require_admin(&req) {
        Ok(value) => value,
        Err(e) => return Ok(e),
    };
    let user_id = claims.sub.clone();

    let request_id = path.into_inner();

    // Get reset request
    let mut reset_request: PasswordResetRequest = match db
        .get("password_reset_requests", &request_id)
    {
        Ok(Some(req)) => req,
        Ok(None) => return Ok(HttpResponse::NotFound().json("Reset request not found")),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(format!("Database error: {}", e)))
        }
    };

    if reset_request.status != ResetRequestStatus::Pending {
        return Ok(HttpResponse::BadRequest().json("Request has already been processed"));
    }

    // Approve the request
    reset_request.approve(user_id);

    // Save updated request
    if let Err(err) = db
        .insert("password_reset_requests", &request_id, &reset_request)
        .map_err(actix_web::error::ErrorInternalServerError)
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(reset_request))
}

// Manager rejects a reset request
/// #feature[auth]
#[post("/password-reset-requests/{id}/reject")]
pub async fn reject_reset_request(
    req: HttpRequest,
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // Check if user is authenticated and is admin/super_admin
    let claims = match require_admin(&req) {
        Ok(value) => value,
        Err(e) => return Ok(e),
    };
    let user_id = claims.sub.clone();

    let request_id = path.into_inner();

    // Get reset request
    let mut reset_request: PasswordResetRequest = match db
        .get("password_reset_requests", &request_id)
    {
        Ok(Some(req)) => req,
        Ok(None) => return Ok(HttpResponse::NotFound().json("Reset request not found")),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(format!("Database error: {}", e)))
        }
    };

    if reset_request.status != ResetRequestStatus::Pending {
        return Ok(HttpResponse::BadRequest().json("Request has already been processed"));
    }

    // Reject the request
    reset_request.reject(user_id);

    // Save updated request
    if let Err(err) = db
        .insert("password_reset_requests", &request_id, &reset_request)
        .map_err(actix_web::error::ErrorInternalServerError)
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(reset_request))
}

// User uses one-time token to reset password
/// #feature[auth]
#[post("/reset-password")]
pub async fn reset_password(
    db: web::Data<Database>,
    body: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse> {
    let reset_token = body.reset_token.trim();
    let new_password = body.new_password.trim();

    if new_password.len() < 8 {
        return Ok(HttpResponse::BadRequest().json("Password must be at least 8 characters long"));
    }

    // Find the reset request by reset_token
    let all_requests: Vec<PasswordResetRequest> = match db.list("password_reset_requests") {
        Ok(requests) => requests,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(format!("Database error: {}", e)))
        }
    };

    let mut found_request: Option<(String, PasswordResetRequest)> = None;
    for mut req in all_requests {
        if req.reset_token.as_ref() == Some(&reset_token.to_string()) {
            // Check if token is valid
            if !req.can_be_used() {
                if req.is_expired() {
                    req.status = ResetRequestStatus::Expired;
                    let _ = db.insert("password_reset_requests", &req.id, &req);
                    return Ok(HttpResponse::BadRequest()
                        .json("Reset token has expired. Please request a new password reset."));
                }
                return Ok(HttpResponse::BadRequest().json("Invalid or already used reset token"));
            }
            let id = req.id.clone();
            found_request = Some((id, req));
            break;
        }
    }

    let (request_id, mut reset_request) = match found_request {
        Some(r) => r,
        None => return Ok(HttpResponse::BadRequest().json("Invalid reset token")),
    };

    // Hash the new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(new_password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return Err(actix_web::error::ErrorInternalServerError(
                "Failed to hash password",
            ))
        }
    };

    // Update user's password - users are stored by ID, not email, so we need to find by email first
    let users: Vec<UserRecord> = match db.list("users") {
        Ok(users) => users,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(format!("Database error: {}", e)))
        }
    };

    let mut user = match users
        .into_iter()
        .find(|u| u.email == reset_request.user_email)
    {
        Some(u) => u,
        None => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    user.password_hash = password_hash;
    match db.insert("users", &user.id, &user) {
        Ok(_) => {}
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(format!("Failed to update user: {}", e))
            )
        }
    }

    // Mark reset request as used
    reset_request.mark_used();
    if let Err(err) = db
        .insert("password_reset_requests", &request_id, &reset_request)
        .map_err(actix_web::error::ErrorInternalServerError)
    {
        return Err(err);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password has been successfully reset. You can now login with your new password."
    })))
}
