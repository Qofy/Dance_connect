use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use chrono::Utc;

use crate::{models::*, AppState};
use crate::models::{LoginRequest, LoginResponse, RegisterRequest, RemoteTokenResponse};

pub async fn get_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    let data = state.lock().unwrap();
    let mut events: Vec<Event> = data.events.values().cloned().collect();

    if let Some(city) = &params.city {
        events.retain(|e| e.city.to_lowercase() == city.to_lowercase());
    }
    if let Some(state_filter) = &params.state {
        events.retain(|e| e.state.to_lowercase() == state_filter.to_lowercase());
    }

    if let Some(sort) = &params.sort {
        match sort.as_str() {
            "-start_date" => events.sort_by(|a, b| b.start_date.cmp(&a.start_date)),
            "start_date" => events.sort_by(|a, b| a.start_date.cmp(&b.start_date)),
            "-current_attendees" => events.sort_by(|a, b| b.current_attendees.cmp(&a.current_attendees)),
            _ => {}
        }
    }

    if let Some(limit) = params.limit {
        events.truncate(limit);
    }

    Ok(Json(events))
}

pub async fn get_event(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Event>, StatusCode> {
    let data = state.lock().unwrap();
    match data.events.get(&id) {
        Some(event) => Ok(Json(event.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<Event>, StatusCode> {
    let mut data = state.lock().unwrap();
    let now = Utc::now();
    
    let event = Event {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        event_type: payload.event_type,
        dance_styles: payload.dance_styles,
        start_date: payload.start_date,
        end_date: payload.end_date,
        venue_name: payload.venue_name,
        address: payload.address,
        city: payload.city,
        state: payload.state,
        zip_code: payload.zip_code,
        latitude: payload.latitude,
        longitude: payload.longitude,
        ticket_price: payload.ticket_price,
        max_attendees: payload.max_attendees,
        current_attendees: 0,
        image_url: payload.image_url,
        website_url: payload.website_url,
        contact_email: payload.contact_email,
        contact_phone: payload.contact_phone,
        status: EventStatus::Public,
        created_at: now,
        updated_at: now,
    };

    data.events.insert(event.id, event.clone());
    Ok(Json(event))
}

pub async fn update_event(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<Event>, StatusCode> {
    let mut data = state.lock().unwrap();
    
    match data.events.get_mut(&id) {
        Some(event) => {
            event.title = payload.title;
            event.description = payload.description;
            event.event_type = payload.event_type;
            event.dance_styles = payload.dance_styles;
            event.start_date = payload.start_date;
            event.end_date = payload.end_date;
            event.venue_name = payload.venue_name;
            event.address = payload.address;
            event.city = payload.city;
            event.state = payload.state;
            event.zip_code = payload.zip_code;
            event.latitude = payload.latitude;
            event.longitude = payload.longitude;
            event.ticket_price = payload.ticket_price;
            event.max_attendees = payload.max_attendees;
            event.image_url = payload.image_url;
            event.website_url = payload.website_url;
            event.contact_email = payload.contact_email;
            event.contact_phone = payload.contact_phone;
            event.updated_at = Utc::now();
            
            Ok(Json(event.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_event(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let mut data = state.lock().unwrap();
    match data.events.remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    let data = state.lock().unwrap();
    let users: Vec<User> = data.users.values().cloned().collect();
    Ok(Json(users))
}

pub async fn get_user(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    let data = state.lock().unwrap();
    match data.users.get(&id) {
        Some(user) => Ok(Json(user.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_current_user(State(state): State<AppState>) -> Result<Json<User>, StatusCode> {
    let data = state.lock().unwrap();
    if let Some(user) = data.users.values().next() {
        Ok(Json(user.clone()))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let mut data = state.lock().unwrap();
    let now = Utc::now();
    
    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        name: payload.name,
        password_hash: "default".to_string(),
        role: payload.role,
        created_at: now,
        updated_at: now,
    };

    // Save to database
    let key = format!("user:{}", user.id);
    if let Ok(serialized) = bincode::serialize(&user) {
        let _ = data.db.insert(key.as_bytes(), serialized);
        let _ = data.db.flush();
    }
    
    data.users.insert(user.id, user.clone());
    Ok(Json(user))
}

pub async fn update_user(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let mut data = state.lock().unwrap();
    
    let user_id = id;
    match data.users.get_mut(&user_id) {
        Some(user) => {
            user.email = payload.email;
            user.name = payload.name;
            user.role = payload.role;
            user.updated_at = Utc::now();
            
            let updated_user = user.clone();
            
            // Save to database
            let key = format!("user:{}", updated_user.id);
            if let Ok(serialized) = bincode::serialize(&updated_user) {
                let _ = data.db.insert(key.as_bytes(), serialized);
                let _ = data.db.flush();
            }
            
            Ok(Json(updated_user))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn upload_file(mut multipart: Multipart) -> Result<Json<UploadResponse>, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("upload").to_string();
            let _data = field.bytes().await.unwrap();
            
            let file_url = format!("/uploads/{}", filename);
            return Ok(Json(UploadResponse { file_url }));
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

pub async fn get_registrations(State(state): State<AppState>) -> Result<Json<Vec<crate::Registration>>, StatusCode> {
    let data = state.lock().unwrap();
    let registrations: Vec<crate::Registration> = data.registrations.values().cloned().collect();
    Ok(Json(registrations))
}

pub async fn create_registration(
    State(state): State<AppState>,
    Json(payload): Json<CreateRegistrationRequest>,
) -> Result<Json<crate::Registration>, StatusCode> {
    let mut data = state.lock().unwrap();
    let now = Utc::now();
    
    let registration = crate::Registration {
        id: Uuid::new_v4(),
        event_id: payload.event_id,
        user_email: payload.user_email,
        created_at: now,
    };

    data.registrations.insert(registration.id, registration.clone());
    Ok(Json(registration))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut data = state.lock().unwrap();
    
    println!("Login attempt: email={}, password={}", payload.email, payload.password);
    println!("Available users:");
    for user in data.users.values() {
        println!("  - email: {}, password: {}", user.email, user.password_hash);
    }
    
    let user = data.users.values().find(|u| u.email == payload.email && u.password_hash == payload.password)
        .ok_or(StatusCode::UNAUTHORIZED)?.clone();
    
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + chrono::Duration::hours(24);
    data.tokens.insert(token.clone(), (user.id, expires_at));
    
    Ok(Json(LoginResponse { token, user }))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let mut data = state.lock().unwrap();
    
    if data.users.values().any(|u| u.email == payload.email) {
        return Err(StatusCode::CONFLICT);
    }
    
    let user = User {
        id: Uuid::new_v4(),
        email: payload.email.clone(),
        name: payload.name.clone(),
        password_hash: payload.password.clone(),
        role: UserRole::Dancer,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    println!("Registering user: email={}, password={}", user.email, user.password_hash);
    
    // Save to database
    let key = format!("user:{}", user.id);
    if let Ok(serialized) = bincode::serialize(&user) {
        let _ = data.db.insert(key.as_bytes(), serialized);
        let _ = data.db.flush();
        println!("User saved to database");
    } else {
        println!("Failed to serialize user for database");
    }
    
    data.users.insert(user.id, user.clone());
    println!("User added to memory. Total users: {}", data.users.len());
    
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + chrono::Duration::hours(24);
    data.tokens.insert(token.clone(), (user.id, expires_at));
    
    Ok(Json(LoginResponse { token, user }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let mut data = state.lock().unwrap();
    
    if let Some(token) = payload.get("token").and_then(|t| t.as_str()) {
        data.tokens.remove(token);
    }
    
    Ok(StatusCode::OK)
}

pub async fn login_remotely(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<RemoteTokenResponse>, StatusCode> {
    let mut data = state.lock().unwrap();
    
    let user = data.users.values().find(|u| u.email == payload.email && u.password_hash == payload.password)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    if !matches!(user.role, UserRole::Admin) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let token = format!("remote_{}", Uuid::new_v4());
    let expires_at = Utc::now() + chrono::Duration::hours(1);
    data.remote_tokens.insert(token.clone(), expires_at);
    
    Ok(Json(RemoteTokenResponse { token, expires_at }))
}

pub async fn register_event_remotely(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<Event>, StatusCode> {
    create_event(State(state), Json(payload)).await
}