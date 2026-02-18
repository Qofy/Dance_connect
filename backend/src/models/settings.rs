use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Global authentication-related toggles persisted in sled.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthSettings {
    pub registration_open: bool,
    pub invite_only: bool,
    pub invite_message: Option<String>,
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            registration_open: true,
            invite_only: false,
            invite_message: Some("Registration is by private invite only".to_string()),
        }
    }
}

impl AuthSettings {
    pub fn effective_message(&self) -> String {
        self.invite_message
            .clone()
            .unwrap_or_else(|| "Registration is by private invite only".to_string())
    }
}

/// Single-use invite token that can unlock registration when public signup is disabled.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteToken {
    pub token: String,
    pub created_at: String,
    pub created_by: String,
    pub email: Option<String>,
    pub expires_at: Option<String>,
    pub used_at: Option<String>,
    pub used_by: Option<String>,
}

/// Global UI settings persisted in sled.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiSettings {
    pub default_language: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            default_language: "en".to_string(),
        }
    }
}

impl InviteToken {
    pub fn new(created_by: &str, email: Option<String>, ttl_hours: i64) -> Self {
        let now = Utc::now();
        let expires_at = if ttl_hours > 0 {
            Some((now + Duration::hours(ttl_hours)).to_rfc3339())
        } else {
            None
        };

        Self {
            token: Uuid::new_v4().simple().to_string(),
            created_at: now.to_rfc3339(),
            created_by: created_by.to_string(),
            email,
            expires_at,
            used_at: None,
            used_by: None,
        }
    }

    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    pub fn is_expired(&self) -> bool {
        if let Some(exp) = &self.expires_at {
            if let Ok(exp_dt) = exp.parse::<chrono::DateTime<Utc>>() {
                return exp_dt < Utc::now();
            }
        }
        false
    }

    pub fn can_be_used(&self) -> bool {
        !self.is_used() && !self.is_expired()
    }

    pub fn mark_used(&mut self, user_id: &str) {
        self.used_at = Some(Utc::now().to_rfc3339());
        self.used_by = Some(user_id.to_string());
    }
}
