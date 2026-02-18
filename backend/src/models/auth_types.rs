// src/models/auth_types.rs
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub roles: Vec<String>, // e.g. ["user"], ["admin"], ["super_admin"]
    #[serde(default)]
    pub permissions: Vec<String>, // granular permissions
    #[serde(default)]
    pub access_level: Option<String>, // "read", "write", "admin"
    #[serde(default)]
    pub subscription_tier: Option<String>, // free, freelancer, pro, business, teams, enterprise
    #[serde(default)]
    pub tenant_id: Option<String>, // for multi-tenant support
    pub iss: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PasswordEncoding {
    PlainArgon2,
    Sha256ThenArgon2,
}

impl Default for PasswordEncoding {
    fn default() -> Self {
        PasswordEncoding::PlainArgon2
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub invite_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub hashed_email: Option<String>,
    #[serde(default)]
    pub hashed_password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub created_at: String,
    #[serde(default = "default_true")]
    pub approved: bool,
    #[serde(default)]
    pub approved_at: Option<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default = "default_true")]
    pub verified: bool,
    #[serde(default)]
    pub verified_at: Option<String>,
    #[serde(default)]
    pub verified_by: Option<String>,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub blocked_at: Option<String>,
    #[serde(default)]
    pub blocked_by: Option<String>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub locked_at: Option<String>,
    #[serde(default)]
    pub locked_by: Option<String>,
    #[serde(default)]
    pub email_hash: Option<String>,
    #[serde(default = "PasswordEncoding::default")]
    pub password_encoding: PasswordEncoding,

    // Profile fields
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub photo_url: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub secondary_email: Option<String>,

    // Security
    #[serde(default)]
    pub mfa_enabled: bool,
    #[serde(default)]
    pub mfa_secret: Option<String>,
    #[serde(default)]
    pub trusted_ips: Vec<String>,
    #[serde(default)]
    pub security_questions: Vec<SecurityQuestion>,

    // Social identities
    #[serde(default)]
    pub social_links: SocialLinks,

    // Biometrics (placeholder for future implementation)
    #[serde(default)]
    pub biometric_enabled: bool,

    // Permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub preferences: Option<serde_json::Value>,

    // Subscription & feature visibility
    #[serde(default)]
    pub subscription_tier: Option<String>, // free, freelancer, pro, business, teams, enterprise
    #[serde(default)]
    pub feature_visibility: FeatureVisibility, // user-level toggles for menu items
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecurityQuestion {
    pub question: String,
    pub answer_hash: String,
}

/// User-level feature visibility toggles
/// These control which menu items/features the user wants to see
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FeatureVisibility {
    #[serde(default = "default_true")]
    pub invoice_designer: bool,
    #[serde(default = "default_true")]
    pub quotes: bool,
    #[serde(default = "default_true")]
    pub invoices: bool,
    #[serde(default = "default_true")]
    pub blogs: bool,
    #[serde(default = "default_true")]
    pub products: bool,
    #[serde(default = "default_true")]
    pub books: bool,
    #[serde(default = "default_true")]
    pub fahrrad: bool,
    #[serde(default = "default_true")]
    pub timeline_organizer: bool,
    #[serde(default = "default_true")]
    pub cv_designer: bool,
    #[serde(default = "default_true")]
    pub court_timeline_designer: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SocialLinks {
    #[serde(default)]
    pub google: Option<String>,
    #[serde(default)]
    pub github: Option<String>,
    #[serde(default)]
    pub gitlab: Option<String>,
    #[serde(default)]
    pub linkedin: Option<String>,
    #[serde(default)]
    pub twitter: Option<String>,
    #[serde(default)]
    pub facebook: Option<String>,
    #[serde(default)]
    pub slack: Option<String>,
    #[serde(default)]
    pub keybase: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
}

impl UserRecord {
    pub fn new_admin(email: &str, password_hash: String) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.to_lowercase(),
            password_hash,
            roles: vec!["super_admin".into()],
            created_at: timestamp.clone(),
            approved: true,
            approved_at: Some(timestamp.clone()),
            approved_by: None,
            verified: true,
            verified_at: Some(timestamp),
            verified_by: None,
            blocked: false,
            blocked_at: None,
            blocked_by: None,
            locked: false,
            locked_at: None,
            locked_by: None,
            email_hash: None,
            password_encoding: PasswordEncoding::PlainArgon2,
            full_name: None,
            photo_url: None,
            phone: None,
            secondary_email: None,
            mfa_enabled: false,
            mfa_secret: None,
            trusted_ips: vec![],
            security_questions: vec![],
            social_links: SocialLinks::default(),
            biometric_enabled: false,
            permissions: vec!["*".into()], // super_admin has all permissions
            preferences: None,
            subscription_tier: Some("enterprise".into()),
            feature_visibility: FeatureVisibility::default(),
        }
    }

    pub fn new_user(email: &str, password_hash: String) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.to_lowercase(),
            password_hash,
            roles: vec!["user".into()],
            created_at: timestamp.clone(),
            approved: true,
            approved_at: Some(timestamp.clone()),
            approved_by: None,
            verified: true,
            verified_at: Some(timestamp),
            verified_by: None,
            blocked: false,
            blocked_at: None,
            blocked_by: None,
            locked: false,
            locked_at: None,
            locked_by: None,
            email_hash: None,
            password_encoding: PasswordEncoding::PlainArgon2,
            full_name: None,
            photo_url: None,
            phone: None,
            secondary_email: None,
            mfa_enabled: false,
            mfa_secret: None,
            trusted_ips: vec![],
            security_questions: vec![],
            social_links: SocialLinks::default(),
            biometric_enabled: false,
            permissions: vec![],
            preferences: None,
            subscription_tier: Some("free".into()),
            feature_visibility: FeatureVisibility::default(),
        }
    }
}
