/// Integration tests: Admin Bagian 1 — user management + audit trail.
/// Requires a running PostgreSQL instance with all migrations applied.
///
///   cargo test --test admin_user_test -- --nocapture
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use career_path_be::{app, config::Config, db::pool, features::project::storage::local::LocalStorage, state::AppState};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

// ── Test helpers ─────────────────────────────────────────────────────────────

async fn build_app() -> axum::Router {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let db = pool::create_pool(&config.database_url).await.expect("db pool");
    let storage = Arc::new(LocalStorage::new(config.storage_root.clone()).expect("storage"));
    let state = Arc::new(AppState { db, config: Arc::new(config), storage });
    app::create_app(state)
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("body json")
}

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

/// Login with given credentials. Returns access_token on success.
async fn login(app: &axum::Router, email: &str, password: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"email": email, "password": password}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = body_json(resp).await;
    (status, body)
}

/// Register a fresh user and return their access token.
async fn register_user(app: &axum::Router) -> (String, String) {
    let email = format!("admin_test_{}@example.com", uuid::Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "password123", "name": "Test User"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "register failed");
    let body = body_json(resp).await;
    let token = body["data"]["access_token"].as_str().unwrap().to_string();
    (email, token)
}

/// Get admin token using the seeded admin credentials.
async fn admin_token(app: &axum::Router) -> String {
    let (status, body) = login(app, "admin@careerpath.local", "ChangeMeASAP123!").await;
    assert_eq!(status, StatusCode::OK, "admin login failed: {body}");
    body["data"]["access_token"].as_str().unwrap().to_string()
}

// ── Access control ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_endpoints_reject_unauthenticated() {
    let app = build_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_endpoints_reject_regular_user() {
    let app = build_app().await;
    let (_, user_token) = register_user(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users")
                .header(header::AUTHORIZATION, bearer(&user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_access_user_list() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["users"].is_array());
    assert!(body["data"]["pagination"].is_object());
}

// ── List filters ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_users_search_filter() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users?search=admin&role=admin")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let users = body["data"]["users"].as_array().unwrap();
    // Seeded admin should appear
    assert!(!users.is_empty(), "expected at least 1 admin user");
    for u in users {
        assert_eq!(u["role"].as_str().unwrap(), "admin");
    }
}

#[tokio::test]
async fn test_list_users_is_active_filter() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users?is_active=true&per_page=5")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let users = body["data"]["users"].as_array().unwrap();
    for u in users {
        assert_eq!(u["is_active"].as_bool().unwrap(), true);
    }
}

// ── User detail ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_user_detail() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (_, _user_token) = register_user(&app).await;

    // Get user list to find a valid user id
    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users?per_page=1")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let list_body = body_json(list_resp).await;
    let user_id = list_body["data"]["users"][0]["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["success"], true);
    let data = &body["data"];
    assert!(data["id"].is_string());
    assert!(data["stats"].is_object());
    assert!(data["module_progress"].is_array());
    assert!(data["recent_quiz_attempts"].is_array());
    assert!(data["project_submissions"].is_array());
}

#[tokio::test]
async fn test_get_nonexistent_user_returns_404() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/users/00000000-0000-0000-0000-000000000099")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Update user ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_update_user_name() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;

    // Find user id
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"name": "Updated Name"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn test_update_user_empty_body_returns_400() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_user_invalid_role_returns_400() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"role": "superadmin"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_promote_to_admin_and_demote_back() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    // Promote to admin
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"role": "admin"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify role changed
    let detail_body = get_user_detail(&app, &token, &user_id).await;
    assert_eq!(detail_body["data"]["role"].as_str().unwrap(), "admin");

    // Demote back
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"role": "user"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let detail_body = get_user_detail(&app, &token, &user_id).await;
    assert_eq!(detail_body["data"]["role"].as_str().unwrap(), "user");
}

// ── Self-protection ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_cannot_deactivate_self() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    // Admin's own id
    let (_, body) = login(&app, "admin@careerpath.local", "ChangeMeASAP123!").await;
    let admin_id = body["data"]["user"]["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{admin_id}/deactivate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "testing self deactivation"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_json(resp).await;
    assert_eq!(body["error"]["code"].as_str().unwrap(), "BAD_REQUEST");
}

#[tokio::test]
async fn test_admin_cannot_update_self() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let (_, body) = login(&app, "admin@careerpath.local", "ChangeMeASAP123!").await;
    let admin_id = body["data"]["user"]["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{admin_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"role": "user"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Deactivate / activate flow ────────────────────────────────────────────────

#[tokio::test]
async fn test_deactivate_user_login_fails_then_activate_login_succeeds() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    // Deactivate
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/deactivate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "Violated ToS section 3"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Login should now fail with ACCOUNT_DEACTIVATED
    let (status, body) = login(&app, &email, "password123").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"].as_str().unwrap(), "ACCOUNT_DEACTIVATED");

    // Activate
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/activate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "Appeal accepted"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Login should now succeed
    let (status, _) = login(&app, &email, "password123").await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_deactivate_already_deactivated_returns_409() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    // First deactivation
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/deactivate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "Spam account detected"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second deactivation — should 409
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/deactivate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "Trying again"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_activate_already_active_returns_409() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    // User is already active — activating should 409
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/activate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_deactivate_reason_too_short_returns_400() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/deactivate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "bad"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Audit log endpoints ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_audit_log_recorded_on_deactivate() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    // Deactivate user
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/users/{user_id}/deactivate"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reason": "Testing audit trail record"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check user-specific audit logs
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/users/{user_id}/audit-logs"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let logs = body["data"]["logs"].as_array().unwrap();
    assert!(!logs.is_empty(), "expected at least one audit log");
    assert_eq!(
        logs.iter()
            .find(|l| l["action"].as_str() == Some("user.deactivated"))
            .is_some(),
        true
    );
}

#[tokio::test]
async fn test_global_audit_log_with_action_filter() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/audit-logs?action=user.deactivated")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let logs = body["data"]["logs"].as_array().unwrap();
    for log in logs {
        assert_eq!(log["action"].as_str().unwrap(), "user.deactivated");
    }
}

#[tokio::test]
async fn test_audit_log_role_change_recorded() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let (email, _) = register_user(&app).await;
    let user_id = get_user_id_by_email(&app, &token, &email).await;

    // Promote to admin
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"role": "admin"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Check audit log for this user
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/users/{user_id}/audit-logs"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_json(resp).await;
    let logs = body["data"]["logs"].as_array().unwrap();
    let role_change_log = logs
        .iter()
        .find(|l| l["action"].as_str() == Some("user.role_changed"));
    assert!(role_change_log.is_some(), "role_changed audit entry not found");
    let meta = &role_change_log.unwrap()["metadata"];
    assert_eq!(meta["old_role"].as_str().unwrap(), "user");
    assert_eq!(meta["new_role"].as_str().unwrap(), "admin");
}

#[tokio::test]
async fn test_audit_log_pagination() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/audit-logs?page=1&per_page=5")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let pagination = &body["data"]["pagination"];
    assert!(pagination["page"].as_i64().unwrap() >= 1);
    assert!(pagination["per_page"].as_i64().unwrap() <= 5);
    assert!(pagination["total_items"].is_number());
    assert!(pagination["total_pages"].is_number());
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Find a user's UUID by searching for their email in the admin user list.
async fn get_user_id_by_email(app: &axum::Router, admin_token: &str, email: &str) -> String {
    // Email contains only alphanumerics, '@', '.', '_', '-' — safe to embed directly.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/users?search={email}&per_page=5"))
                .header(header::AUTHORIZATION, bearer(admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_json(resp).await;
    let users = body["data"]["users"].as_array().expect("users array");
    users
        .iter()
        .find(|u| u["email"].as_str() == Some(email))
        .expect("user not found by email")["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Get single user detail as Value.
async fn get_user_detail(app: &axum::Router, admin_token: &str, user_id: &str) -> Value {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/users/{user_id}"))
                .header(header::AUTHORIZATION, bearer(admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    body_json(resp).await
}

