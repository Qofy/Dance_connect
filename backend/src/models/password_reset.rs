// src/models/password_reset.rs
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResetRequestStatus {
    Pending,
    Approved,
    Used,
    Expired,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordResetRequest {
    pub id: String,
    pub user_email: String,
    pub request_token: String,       // Manager views this in dashboard
    pub reset_token: Option<String>, // One-time token generated after approval
    pub status: ResetRequestStatus,
    pub requested_at: String,
    pub approved_at: Option<String>,
    pub approved_by: Option<String>, // Manager user_id who approved
    pub used_at: Option<String>,
    pub expires_at: Option<String>, // Reset token expiration
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl PasswordResetRequest {
    pub fn new(email: String, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_email: email.to_lowercase(),
            request_token: Uuid::new_v4().to_string(),
            reset_token: None,
            status: ResetRequestStatus::Pending,
            requested_at: Utc::now().to_rfc3339(),
            approved_at: None,
            approved_by: None,
            used_at: None,
            expires_at: None,
            ip_address,
            user_agent,
        }
    }

    pub fn approve(&mut self, manager_id: String) {
        self.status = ResetRequestStatus::Approved;
        self.approved_at = Some(Utc::now().to_rfc3339());
        self.approved_by = Some(manager_id);
        self.reset_token = Some(Uuid::new_v4().to_string());
        // Reset token expires in 24 hours
        self.expires_at = Some((Utc::now() + chrono::Duration::hours(24)).to_rfc3339());
    }

    pub fn reject(&mut self, manager_id: String) {
        self.status = ResetRequestStatus::Rejected;
        self.approved_at = Some(Utc::now().to_rfc3339());
        self.approved_by = Some(manager_id);
    }

    pub fn mark_used(&mut self) {
        self.status = ResetRequestStatus::Used;
        self.used_at = Some(Utc::now().to_rfc3339());
    }

    pub fn is_expired(&self) -> bool {
        if let Some(exp) = &self.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(exp) {
                return Utc::now() > expiry.with_timezone(&Utc);
            }
        }
        false
    }

    pub fn can_be_used(&self) -> bool {
        self.status == ResetRequestStatus::Approved
            && self.reset_token.is_some()
            && !self.is_expired()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordRequest {
    pub reset_token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
    pub request_id: String,
}
