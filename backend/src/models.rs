use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub organizer_id: Option<Uuid>,   // set server-side from Bearer token
    pub title: String,
    pub description: Option<String>,
    pub event_type: Option<EventType>,
    pub dance_styles: Vec<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub venue_name: Option<String>,
    pub address: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub ticket_price: Option<f64>,
    pub max_attendees: Option<u32>,
    pub current_attendees: u32,
    pub image_url: Option<String>,
    pub website_url: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Workshop,
    Social,
    Competition,
    Festival,
    Showcase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Draft,
    Public,
    Completed,
    Cancelled,
}

// Registration — moved here from main.rs, expanded for creator flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_email: String,
    pub dancer_name: Option<String>,
    pub status: String,           // "active" | "cancelled"
    pub check_in_status: String,  // "pending" | "checked_in"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub event_type: Option<EventType>,
    pub dance_styles: Vec<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub venue_name: Option<String>,
    pub address: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub ticket_price: Option<f64>,
    pub max_attendees: Option<u32>,
    pub image_url: Option<String>,
    pub website_url: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub status: Option<String>,   // "draft" | "public" — defaults to public
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub password_hash: String,
    pub role: UserRole,
    pub user_type: String,         // "dancer" | "creator" | "both" | "admin"
    pub dance_styles: Vec<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Creator,
    Dancer,
}

// Used for PUT /users/:id — all fields optional for partial update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub user_type: Option<String>,
    pub dance_styles: Option<Vec<String>>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

// Used for PUT /events/:id — all fields optional, includes status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<EventType>,
    pub dance_styles: Option<Vec<String>>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub venue_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub ticket_price: Option<f64>,
    pub max_attendees: Option<u32>,
    pub image_url: Option<String>,
    pub website_url: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub status: Option<String>,
}

// Used for POST /users (admin create)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventQuery {
    pub sort: Option<String>,
    pub limit: Option<usize>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub dance_styles: Option<String>,
    pub organizer_id: Option<Uuid>,   // filter by creator
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationQuery {
    pub event_id: Option<Uuid>,       // filter registrations by event
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub file_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRegistrationRequest {
    pub event_id: Uuid,
    pub user_email: String,
    pub dancer_name: Option<String>,
}
