use actix_web::{test, web, App};
use hex;
use sha2::{Digest, Sha256};
use sha2::{Digest, Sha256};
use tempfile::tempdir;
use tempfile::tempdir;
use mitote_v026_bike_connect_backend::models::{LoginRequest, RegisterRequest};
use mitote_v026_bike_connect_backend::{
    config::{AppConfig, DatabaseConfig, LoggingConfig, SecurityConfig, ServerConfig, TokenMode},
    db::Database,
    handlers,
};

fn cfg() -> AppConfig {
    AppConfig {
        security: SecurityConfig {
            access_token: "".into(),
            rate_limit_enabled: false,
            rate_limit_rpm: 100,
            auth_token_expiry_hours: 24,
            token_iss: "e2e_iss".into(),
            token_aud: "e2e_aud".into(),
            token_ttl_seconds: 3600,
            paseto_v4_local_key_hex:
                "142f46b1b4acb0946e0d9413f29b331db345cf664b9307165eab7531fa32d8bd".into(),
            token_mode: TokenMode::JwtHmac,
            debug_mode: false,
            health_check_enabled: true,
            metrics_enabled: false,
        },
        server: ServerConfig {
            host: "localhost".into(),
            port: 0,
            name: "e2e".into(),
        },
        database: DatabaseConfig {
            host: "localhost".into(),
            port: 5432,
            database: "e2e".into(),
            username: "e2e".into(),
            password: "e2e".into(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            ssl_mode: "prefer".into(),
        },
        sled_path: "e2e.db".into(),
        backup_dir: "backups".into(),
        backup_name_template: "backup_{{timestamp}}".into(),
        uploads_path: "./uploads".into(),
        backup_interval: None,
        backup_retention: 3,
        pg_conns: vec![],
        cors_rules: vec![],
        logging: LoggingConfig {
            level: "info".into(),
            file_enabled: false,
            file_path: None,
        },
        database_sync_on: false,
    }
}

fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[actix_web::test]
async fn flow_register_login_logout_me() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::register)
            .service(handlers::auth::login)
            .service(handlers::auth::logout)
            .service(handlers::auth::me),
    )
    .await;

    // Use strong password that meets validation requirements
    let reg = RegisterRequest {
        email: "test@example.com".into(),
        password: "SecurePass123!@#".into(),
        invite_token: None,
    };
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&reg)
        .to_request();
    let reg_resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let token = reg_resp["token"].as_str().unwrap();

    let me_req = test::TestRequest::get()
        .uri("/me")
        .insert_header(("authorization", format!("Bearer {}", token)))
        .to_request();
    let me: serde_json::Value = test::call_and_read_body_json(&app, me_req).await;
    assert_eq!(me["email"], "test@example.com");

    let login = LoginRequest {
        email: Some("test@example.com".into()),
        password: Some("SecurePass123!@#".into()),
        hashed_email: None,
        hashed_password: None,
    };
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login)
        .to_request();
    let login_resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(login_resp.get("token").is_some());

    let req = test::TestRequest::post().uri("/logout").to_request();
    let logout_body = test::call_and_read_body(&app, req).await;
    assert!(std::str::from_utf8(&logout_body)
        .unwrap()
        .contains("logged out"));
}

#[actix_web::test]
async fn register_with_weak_password_fails() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::register),
    )
    .await;

    // Test various weak passwords
    let weak_passwords = vec![
        "short1!A",          // Too short
        "NoSpecialChar123",  // Missing special char
        "no-uppercase-123!", // Missing uppercase
        "NO-LOWERCASE-123!", // Missing lowercase
        "NoNumbers!@#Abc",   // Missing number
        "password123!A",     // Contains common pattern
    ];

    for weak_pass in weak_passwords {
        let reg = RegisterRequest {
            email: "test@example.com".into(),
            password: weak_pass.into(),
            invite_token: None,
        };
        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(&reg)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            400,
            "Weak password '{}' should be rejected",
            weak_pass
        );
    }
}

#[actix_web::test]
async fn register_with_invalid_email_fails() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::register),
    )
    .await;

    let invalid_emails = vec![
        "notanemail".to_string(),
        "@nodomain.com".to_string(),
        "no@domain".to_string(),
        "spaces in@email.com".to_string(),
        "a".repeat(65) + "@example.com", // Local part too long
    ];

    for invalid_email in invalid_emails {
        let reg = RegisterRequest {
            email: invalid_email.clone(),
            password: "ValidPass123!@#".into(),
            invite_token: None,
        };
        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(&reg)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            400,
            "Invalid email '{}' should be rejected",
            invalid_email
        );
    }
}

#[actix_web::test]
async fn register_duplicate_email_fails() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::register),
    )
    .await;

    let reg = RegisterRequest {
        email: "duplicate@test.com".into(),
        password: "SecurePass123!@#".into(),
        invite_token: None,
    };

    // First registration should succeed
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&reg)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Second registration with same email should fail
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&reg)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409); // Conflict
}

#[actix_web::test]
async fn login_with_wrong_password_fails() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::register)
            .service(handlers::auth::login),
    )
    .await;

    // Register a user
    let reg = RegisterRequest {
        email: "user@test.com".into(),
        password: "CorrectPass123!@#".into(),
        invite_token: None,
    };
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&reg)
        .to_request();
    test::call_service(&app, req).await;

    // Try to login with wrong password
    let login = LoginRequest {
        email: None,
        password: None,
        hashed_email: Some(hash_sha256("user@test.com")),
        hashed_password: Some(hash_sha256("WrongPass123!@#")),
    };
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401); // Unauthorized
}

#[actix_web::test]
async fn login_with_nonexistent_user_fails() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::login),
    )
    .await;

    let login = LoginRequest {
        email: None,
        password: None,
        hashed_email: Some(hash_sha256("nonexistent@test.com")),
        hashed_password: Some(hash_sha256("AnyPass123!@#")),
    };
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401); // Unauthorized
}

#[actix_web::test]
async fn me_endpoint_requires_valid_token() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::me),
    )
    .await;

    // Test without token
    let req = test::TestRequest::get().uri("/me").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test with invalid token
    let req = test::TestRequest::get()
        .uri("/me")
        .insert_header(("authorization", "Bearer invalid_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn email_normalization_works() {
    let mut cfg = cfg();
    let dir = tempdir().unwrap();
    cfg.sled_path = dir.path().join("sled").to_string_lossy().to_string();
    let db = Database::new(&cfg.sled_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(handlers::auth::register)
            .service(handlers::auth::login),
    )
    .await;

    // Register with mixed case email
    let reg = RegisterRequest {
        email: "Test@Example.COM".into(),
        password: "SecurePass123!@#".into(),
        invite_token: None,
    };
    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(&reg)
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert!(resp.get("token").is_some());

    // Login with different case should work (email normalized to lowercase)
    let login = LoginRequest {
        email: None,
        password: None,
        hashed_email: Some(hash_sha256("test@example.com")),
        hashed_password: Some(hash_sha256("SecurePass123!@#")),
    };
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(&login)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
