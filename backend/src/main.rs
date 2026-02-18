use axum::{
    routing::{get, post},
    Router, response::{Response, IntoResponse}, http::{StatusCode, Uri},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tower::ServiceExt;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sled::Db;

mod models;
mod handlers;

use models::*;
use handlers::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Registration {
    id: Uuid,
    event_id: Uuid,
    user_email: String,
    created_at: DateTime<Utc>,
}

type AppState = Arc<Mutex<AppData>>;

struct AppData {
    events: HashMap<Uuid, Event>,
    users: HashMap<Uuid, User>,
    registrations: HashMap<Uuid, Registration>,
    tokens: HashMap<String, (Uuid, DateTime<Utc>)>,
    remote_tokens: HashMap<String, DateTime<Utc>>,
    db: Db,
}

impl AppData {
    fn with_sample_data() -> Self {
        let db = sled::open("danceconnect.db").expect("Failed to open database");
        
        let mut data = Self {
            events: HashMap::new(),
            users: HashMap::new(),
            registrations: HashMap::new(),
            tokens: HashMap::new(),
            remote_tokens: HashMap::new(),
            db: db.clone(),
        };
        
        // Load users from database
        data.load_users_from_db();
        
        // Add default admin if no users exist
        if data.users.is_empty() {
            data.add_default_users();
        }
        
        // Add sample events
        let events = vec![
            Event {
                id: Uuid::new_v4(),
                title: "Salsa Night Extravaganza".to_string(),
                description: Some("Join us for an epic night of salsa dancing!".to_string()),
                event_type: Some(models::EventType::Social),
                dance_styles: vec!["salsa".to_string(), "bachata".to_string()],
                start_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
                end_date: None,
                venue_name: Some("Dance Studio Central".to_string()),
                address: Some("123 Dance St".to_string()),
                city: "New York".to_string(),
                state: "NY".to_string(),
                zip_code: Some("10001".to_string()),
                latitude: Some(40.7128),
                longitude: Some(-74.0060),
                ticket_price: Some(25.0),
                max_attendees: Some(100),
                current_attendees: 45,
                image_url: None,
                website_url: None,
                contact_email: Some("info@danceconnect.com".to_string()),
                contact_phone: None,
                status: models::EventStatus::Public,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Event {
                id: Uuid::new_v4(),
                title: "Hip-Hop Workshop".to_string(),
                description: Some("Learn the latest hip-hop moves from professional dancers!".to_string()),
                event_type: Some(models::EventType::Workshop),
                dance_styles: vec!["hip-hop".to_string()],
                start_date: chrono::NaiveDate::from_ymd_opt(2024, 12, 20).unwrap(),
                end_date: None,
                venue_name: Some("Urban Dance Academy".to_string()),
                address: Some("456 Beat Ave".to_string()),
                city: "Los Angeles".to_string(),
                state: "CA".to_string(),
                zip_code: Some("90210".to_string()),
                latitude: Some(34.0522),
                longitude: Some(-118.2437),
                ticket_price: Some(35.0),
                max_attendees: Some(50),
                current_attendees: 23,
                image_url: None,
                website_url: None,
                contact_email: Some("workshops@danceconnect.com".to_string()),
                contact_phone: None,
                status: models::EventStatus::Public,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];
        
        for event in events {
            data.events.insert(event.id, event);
        }
        
        data
    }
    
    fn load_users_from_db(&mut self) {
        for result in self.db.scan_prefix(b"user:") {
            if let Ok((_key, value)) = result {
                if let Ok(user) = bincode::deserialize::<User>(&value) {
                    self.users.insert(user.id, user);
                }
            }
        }
    }
    
    fn add_default_users(&mut self) {
        let admin_user = User {
            id: Uuid::new_v4(),
            email: "admin@danceconnect.com".to_string(),
            name: "Admin User".to_string(),
            password_hash: "admin123".to_string(),
            role: models::UserRole::Admin,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let demo_user = User {
            id: Uuid::new_v4(),
            email: "demo@danceconnect.com".to_string(),
            name: "Demo User".to_string(),
            password_hash: "demo123".to_string(),
            role: models::UserRole::Dancer,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.save_user(&admin_user);
        self.save_user(&demo_user);
        self.users.insert(admin_user.id, admin_user);
        self.users.insert(demo_user.id, demo_user);
    }
    
    fn save_user(&self, user: &User) {
        let key = format!("user:{}", user.id);
        if let Ok(serialized) = bincode::serialize(user) {
            let _ = self.db.insert(key.as_bytes(), serialized);
            let _ = self.db.flush();
        }
    }
}

async fn spa_handler(uri: Uri) -> Result<Response, StatusCode> {
    let path = uri.path().trim_start_matches('/');
    
    // If it's an API route, return 404
    if path.starts_with("api/") {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // For all other routes, serve index.html
    match ServeDir::new("../dist").oneshot(axum::http::Request::builder().uri("/index.html").body(axum::body::Body::empty()).unwrap()).await {
        Ok(res) => Ok(res.into_response()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(AppData::with_sample_data()));

    let app = Router::new()
        .route("/api/events", get(get_events).post(create_event))
        .route("/api/events/:id", get(get_event).put(update_event).delete(delete_event))
        .route("/api/users", get(get_users).post(create_user))
        .route("/api/users/:id", get(get_user).put(update_user))
        .route("/api/users/me", get(get_current_user))
        .route("/api/registrations", get(get_registrations).post(create_registration))
        .route("/api/upload", post(upload_file))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/logout", post(logout))
        .route("/api/login_remotely", post(login_remotely))
        .route("/api/register_event_remotely", post(register_event_remotely))
        .nest_service("/assets", ServeDir::new("../dist/assets"))
        .fallback_service(ServeDir::new("../dist").append_index_html_on_directories(true))
        .fallback(spa_handler)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8061").await.unwrap();
    println!("Server running on http://localhost:8061");
    axum::serve(listener, app).await.unwrap();
}