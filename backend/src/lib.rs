
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sled::Db;

pub mod models;
pub mod handlers;

pub type AppState = Arc<Mutex<AppData>>;

pub struct AppData {
	pub events: HashMap<Uuid, models::Event>,
	pub users: HashMap<Uuid, models::User>,
	pub registrations: HashMap<Uuid, models::Registration>,
	pub tokens: HashMap<String, (Uuid, DateTime<Utc>)>,
	pub remote_tokens: HashMap<String, DateTime<Utc>>,
	pub db: Db,
}

impl AppData {
	pub fn with_sample_data() -> Self {
		let db = sled::open("danceconnect.db").expect("Failed to open database");

		let mut data = Self {
			events: HashMap::new(),
			users: HashMap::new(),
			registrations: HashMap::new(),
			tokens: HashMap::new(),
			remote_tokens: HashMap::new(),
			db: db.clone(),
		};

		data.load_users_from_db();

		if data.users.is_empty() {
			data.add_default_users();
		}

		let events = vec![
			models::Event {
				id: Uuid::new_v4(),
				organizer_id: None,
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
			models::Event {
				id: Uuid::new_v4(),
				organizer_id: None,
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

	pub fn load_users_from_db(&mut self) {
		for result in self.db.scan_prefix(b"user:") {
			if let Ok((_key, value)) = result {
				if let Ok(user) = bincode::deserialize::<models::User>(&value) {
					self.users.insert(user.id, user);
				}
			}
		}
	}

	pub fn add_default_users(&mut self) {
		let admin_user = models::User {
			id: Uuid::new_v4(),
			email: "admin@danceconnect.com".to_string(),
			full_name: "Admin User".to_string(),
			password_hash: "admin123".to_string(),
			role: models::UserRole::Admin,
			user_type: "admin".to_string(),
			dance_styles: vec![],
			city: None,
			state: None,
			zip_code: None,
			latitude: None,
			longitude: None,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		let demo_user = models::User {
			id: Uuid::new_v4(),
			email: "demo@danceconnect.com".to_string(),
			full_name: "Demo Dancer".to_string(),
			password_hash: "demo123".to_string(),
			role: models::UserRole::Dancer,
			user_type: "dancer".to_string(),
			dance_styles: vec!["salsa".to_string(), "bachata".to_string()],
			city: Some("San Francisco".to_string()),
			state: Some("CA".to_string()),
			zip_code: Some("94105".to_string()),
			latitude: Some(37.7749),
			longitude: Some(-122.4194),
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		self.save_user(&admin_user);
		self.save_user(&demo_user);
		self.users.insert(admin_user.id, admin_user);
		self.users.insert(demo_user.id, demo_user);
	}

	pub fn save_user(&self, user: &models::User) {
		let key = format!("user:{}", user.id);
		if let Ok(serialized) = bincode::serialize(user) {
			let _ = self.db.insert(key.as_bytes(), serialized);
			let _ = self.db.flush();
		}
	}
}
