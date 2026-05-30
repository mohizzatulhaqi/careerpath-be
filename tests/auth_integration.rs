//! Integration tests for the auth flow.
//!
//! Requires a running PostgreSQL server. Set DATABASE_URL before running:
//!
//!   DATABASE_URL="postgres://user:pass@localhost/postgres" cargo test
//!
//! `sqlx::test` creates a fresh temporary database per test and drops it
//! afterwards, so tests are fully isolated even when run in parallel.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use career_path_be::{
    app::create_app,
    config::{Config, StorageBackend},
    features::project::storage::local::LocalStorage,
    state::AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

// ── Test harness ──────────────────────────────────────────────────────────────

struct TestApp {
    router: axum::Router,
    _tmp: TempDir,
}

impl TestApp {
    fn new(pool: sqlx::PgPool) -> Self {
        let tmp = TempDir::new().expect("create tempdir");
        let config = Config {
            database_url: String::new(),
            jwt_secret: "test-secret-must-be-at-least-32-characters".to_string(),
            jwt_expires_in: 900,
            refresh_token_expires_in: 604_800,
            server_port: 0,
            storage_root: tmp.path().to_path_buf(),
            max_upload_size_bytes: 26_214_400,
            app_base_url: "http://localhost:3000".to_string(),
            storage_backend: StorageBackend::Local,
            resend_api_key: None,
        };
        let storage =
            Arc::new(LocalStorage::new(config.storage_root.clone()).expect("local storage"));
        let state = Arc::new(AppState {
            db: pool,
            config: Arc::new(config),
            storage,
        });
        Self { router: create_app(state), _tmp: tmp }
    }

    async fn post(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let bytes = serde_json::to_vec(&body).unwrap();
        let req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(bytes))
            .unwrap();
        let res = self.router.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
        (status, json)
    }
}

fn alice() -> Value {
    json!({ "email": "alice@example.com", "password": "Password123!", "name": "Alice" })
}

// ── Register ──────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_201_with_tokens(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, body) = app.post("/api/auth/register", alice()).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["success"], true);
    assert!(body["data"]["access_token"].is_string());
    assert!(body["data"]["refresh_token"].is_string());
    assert_eq!(body["data"]["user"]["email"], "alice@example.com");
}

#[sqlx::test(migrations = "./migrations")]
async fn duplicate_email_returns_409(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    app.post("/api/auth/register", alice()).await;
    let (status, _) = app.post("/api/auth/register", alice()).await;

    assert_eq!(status, StatusCode::CONFLICT);
}

// ── Login ─────────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn login_with_correct_password_returns_200(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    app.post("/api/auth/register", alice()).await;
    let (status, body) = app
        .post("/api/auth/login", json!({ "email": "alice@example.com", "password": "Password123!" }))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["access_token"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_wrong_password_returns_401(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    app.post("/api/auth/register", alice()).await;
    let (status, _) = app
        .post("/api/auth/login", json!({ "email": "alice@example.com", "password": "wrong" }))
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_unknown_email_returns_401(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app
        .post("/api/auth/login", json!({ "email": "nobody@example.com", "password": "Password123!" }))
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Refresh ───────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn refresh_returns_new_tokens(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, reg) = app.post("/api/auth/register", alice()).await;
    let token_a = reg["data"]["refresh_token"].as_str().unwrap().to_string();

    let (status, body) =
        app.post("/api/auth/refresh", json!({ "refresh_token": token_a })).await;

    assert_eq!(status, StatusCode::OK);
    let token_b = body["data"]["refresh_token"].as_str().unwrap();
    assert_ne!(token_b, token_a, "refresh token must be rotated");
    assert!(body["data"]["access_token"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_with_fake_token_returns_401(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app
        .post("/api/auth/refresh", json!({ "refresh_token": "not-a-real-token" }))
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Theft detection ───────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn replaying_used_token_revokes_entire_family(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);

    // Step 1: register → token_a
    let (_, reg) = app.post("/api/auth/register", alice()).await;
    let token_a = reg["data"]["refresh_token"].as_str().unwrap().to_string();

    // Step 2: legitimate rotation → token_a is now used, token_b is live
    let (status, body) =
        app.post("/api/auth/refresh", json!({ "refresh_token": token_a })).await;
    assert_eq!(status, StatusCode::OK);
    let token_b = body["data"]["refresh_token"].as_str().unwrap().to_string();

    // Step 3: attacker replays token_a → must be rejected and family revoked
    let (status, _) =
        app.post("/api/auth/refresh", json!({ "refresh_token": token_a })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "used token must be rejected");

    // Step 4: legitimate user tries token_b → also rejected (family was revoked)
    let (status, _) =
        app.post("/api/auth/refresh", json!({ "refresh_token": token_b })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "whole family must be revoked after replay");
}

// ── Logout ────────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn logout_invalidates_refresh_token(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    let (_, reg) = app.post("/api/auth/register", alice()).await;
    let refresh_token = reg["data"]["refresh_token"].as_str().unwrap().to_string();
    let access_token = reg["data"]["access_token"].as_str().unwrap().to_string();

    // Logout using the access token in the Authorization header
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::from(
            serde_json::to_vec(&json!({ "refresh_token": refresh_token })).unwrap(),
        ))
        .unwrap();
    let res = app.router.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Refresh after logout must fail
    let (status, _) =
        app.post("/api/auth/refresh", json!({ "refresh_token": refresh_token })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "token must be invalid after logout");
}
