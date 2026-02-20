use axum::{
    extract::{Path, Query, State, Multipart},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use uuid::Uuid;
use chrono::Utc;

use crate::{models::*, AppState};
use crate::models::{LoginRequest, LoginResponse, RegisterRequest, RemoteTokenResponse, UpdateUserRequest, UpdateEventRequest};

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
    Json(payload): Json<UpdateEventRequest>,
) -> Result<Json<Event>, StatusCode> {
    let mut data = state.lock().unwrap();

    match data.events.get_mut(&id) {
        Some(event) => {
            if let Some(title) = payload.title { event.title = title; }
            if let Some(description) = payload.description { event.description = Some(description); }
            if let Some(event_type) = payload.event_type { event.event_type = Some(event_type); }
            if let Some(dance_styles) = payload.dance_styles { event.dance_styles = dance_styles; }
            if let Some(start_date) = payload.start_date { event.start_date = start_date; }
            if let Some(end_date) = payload.end_date { event.end_date = Some(end_date); }
            if let Some(venue_name) = payload.venue_name { event.venue_name = Some(venue_name); }
            if let Some(address) = payload.address { event.address = Some(address); }
            if let Some(city) = payload.city { event.city = city; }
            if let Some(state_val) = payload.state { event.state = state_val; }
            if let Some(zip_code) = payload.zip_code { event.zip_code = Some(zip_code); }
            if payload.latitude.is_some() { event.latitude = payload.latitude; }
            if payload.longitude.is_some() { event.longitude = payload.longitude; }
            if payload.ticket_price.is_some() { event.ticket_price = payload.ticket_price; }
            if payload.max_attendees.is_some() { event.max_attendees = payload.max_attendees; }
            if let Some(image_url) = payload.image_url { event.image_url = Some(image_url); }
            if let Some(website_url) = payload.website_url { event.website_url = Some(website_url); }
            if let Some(contact_email) = payload.contact_email { event.contact_email = Some(contact_email); }
            if let Some(contact_phone) = payload.contact_phone { event.contact_phone = Some(contact_phone); }
            if let Some(status_str) = payload.status {
                event.status = match status_str.as_str() {
                    "draft"     => EventStatus::Draft,
                    "cancelled" => EventStatus::Cancelled,
                    "completed" => EventStatus::Completed,
                    _           => EventStatus::Public,
                };
            }
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

pub async fn get_current_user(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = state.lock().unwrap();
    let &(user_id, _) = data.tokens.get(token).ok_or(StatusCode::UNAUTHORIZED)?;
    let user = data.users.get(&user_id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(user.clone()))
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
        full_name: payload.name,
        password_hash: "default".to_string(),
        role: payload.role,
        user_type: "dancer".to_string(),
        dance_styles: vec![],
        city: None,
        state: None,
        zip_code: None,
        latitude: None,
        longitude: None,
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
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    let mut data = state.lock().unwrap();

    match data.users.get_mut(&id) {
        Some(user) => {
            if let Some(name) = payload.name {
                user.full_name = name;
            }
            if let Some(email) = payload.email {
                user.email = email;
            }
            if let Some(role_str) = payload.role {
                user.role = match role_str.as_str() {
                    "admin" => UserRole::Admin,
                    "creator" => UserRole::Creator,
                    _ => UserRole::Dancer,
                };
            }
            if let Some(ut) = payload.user_type {
                user.user_type = ut;
            }
            if let Some(styles) = payload.dance_styles {
                user.dance_styles = styles;
            }
            if let Some(city) = payload.city {
                user.city = Some(city);
            }
            if let Some(state_val) = payload.state {
                user.state = Some(state_val);
            }
            if let Some(zip) = payload.zip_code {
                user.zip_code = Some(zip);
            }
            if payload.latitude.is_some() {
                user.latitude = payload.latitude;
            }
            if payload.longitude.is_some() {
                user.longitude = payload.longitude;
            }
            user.updated_at = Utc::now();

            let updated_user = user.clone();

            // Persist to sled
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

pub async fn delete_user(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let mut data = state.lock().unwrap();
    match data.users.remove(&id) {
        Some(user) => {
            let key = format!("user:{}", user.id);
            let _ = data.db.remove(key.as_bytes());
            let _ = data.db.flush();
            Ok(StatusCode::NO_CONTENT)
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
    // Reject blank email or name
    if payload.email.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let mut data = state.lock().unwrap();

    if data.users.values().any(|u| u.email == payload.email) {
        return Err(StatusCode::CONFLICT);
    }

    let role_str = payload.role.as_deref().unwrap_or("dancer");
    let (role, user_type) = match role_str {
        "admin"   => (UserRole::Admin,   "admin".to_string()),
        "creator" => (UserRole::Creator, "creator".to_string()),
        "both"    => (UserRole::Creator, "both".to_string()),
        _         => (UserRole::Dancer,  "dancer".to_string()),
    };

    let user = User {
        id: Uuid::new_v4(),
        email: payload.email.clone(),
        full_name: payload.name.clone(),
        password_hash: payload.password.clone(),
        role,
        user_type,
        dance_styles: vec![],
        city: None,
        state: None,
        zip_code: None,
        latitude: None,
        longitude: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    println!("Registering user: email={}, role={}", user.email, role_str);
    
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