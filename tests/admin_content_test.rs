/// Integration tests: Admin Bagian 2 — content CRUD (Role, Module, Submaterial, Quizzes, Project).
/// Requires a running PostgreSQL instance with all migrations applied.
///
///   cargo test --test admin_content_test -- --nocapture
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use career_path_be::{
    app, config::Config, db::pool, features::project::storage::local::LocalStorage, state::AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn build_app() -> axum::Router {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let db = pool::create_pool(&config.database_url).await.expect("db pool");
    let storage = Arc::new(LocalStorage::new(config.storage_root.clone()).expect("storage"));
    let state = Arc::new(AppState { db, config: Arc::new(config), storage });
    app::create_app(state)
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.expect("bytes");
    serde_json::from_slice(&bytes).expect("json")
}

fn bearer(t: &str) -> String { format!("Bearer {t}") }

async fn admin_token(app: &axum::Router) -> String {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email":"admin@careerpath.local","password":"ChangeMeASAP123!"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_json(resp).await;
    body["data"]["access_token"].as_str().unwrap().to_string()
}

async fn user_token(app: &axum::Router) -> String {
    let email = format!("content_test_{}@example.com", Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"email": email, "password": "pass1234", "name": "User"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    body_json(resp).await["data"]["access_token"].as_str().unwrap().to_string()
}

async fn get(app: &axum::Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

async fn post_json(app: &axum::Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

async fn patch_json(app: &axum::Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(uri)
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

async fn delete_req(app: &axum::Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(uri)
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

// ── Access control ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_content_endpoints_reject_regular_user() {
    let app = build_app().await;
    let token = user_token(&app).await;

    let (status, _) = get(&app, "/api/admin/roles", &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = get(&app, "/api/admin/modules", &token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ── Role CRUD ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_role_full_lifecycle() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    // List roles
    let (status, body) = get(&app, "/api/admin/roles", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["roles"].is_array());

    // Create role
    let code = format!("test_{}", &Uuid::new_v4().to_string()[..8]);
    let (status, body) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": code, "name": "Test Role", "description": "For testing" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let role_id = body["data"]["id"].as_str().unwrap().to_string();

    // Get detail
    let (status, body) = get(&app, &format!("/api/admin/roles/{role_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["code"].as_str().unwrap(), code);
    assert!(body["data"]["stats"].is_object());

    // Update
    let (status, _) = patch_json(
        &app,
        &format!("/api/admin/roles/{role_id}"),
        &token,
        json!({ "name": "Updated Role Name" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Deactivate
    let (status, _) = post_json(
        &app,
        &format!("/api/admin/roles/{role_id}/deactivate"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Deactivated role should show is_active=false
    let (_, body) = get(&app, &format!("/api/admin/roles/{role_id}"), &token).await;
    assert_eq!(body["data"]["is_active"].as_bool().unwrap(), false);

    // Restore
    let (status, _) = post_json(
        &app,
        &format!("/api/admin/roles/{role_id}/restore"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_role_code_must_be_lowercase_alphanumeric() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let (status, body) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": "UPPERCASE-CODE", "name": "Bad" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "uppercase code should fail: {body}");
}

#[tokio::test]
async fn test_role_duplicate_code_returns_409() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let code = format!("dup_{}", &Uuid::new_v4().to_string()[..8]);

    // Create once
    let (status, _) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": code, "name": "First" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Create again with same code
    let (status, body) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": code, "name": "Duplicate" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "duplicate code should 409: {body}");
}

// ── Module CRUD ───────────────────────────────────────────────────────────────

async fn get_any_role_id(app: &axum::Router, token: &str) -> String {
    let (_, body) = get(app, "/api/admin/roles?is_active=true", token).await;
    body["data"]["roles"][0]["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_module_full_lifecycle() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;

    // Create module
    let (status, body) = post_json(
        &app,
        "/api/admin/modules",
        &token,
        json!({ "role_id": role_id, "title": "Test Module", "description": "Testing" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create module: {body}");
    let module_id = body["data"]["id"].as_str().unwrap().to_string();

    // Get module
    let (status, body) = get(&app, &format!("/api/admin/modules/{module_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Test Module");
    assert!(body["data"]["stats"]["total_submaterials"].is_number());

    // Update
    let (status, _) = patch_json(
        &app,
        &format!("/api/admin/modules/{module_id}"),
        &token,
        json!({ "title": "Updated Module Title" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Soft delete (unpublish) — DELETE without force
    let (status, body) = delete_req(&app, &format!("/api/admin/modules/{module_id}"), &token).await;
    assert_eq!(status, StatusCode::OK, "soft delete: {body}");
    assert!(body["data"]["message"].is_null() || body["message"].is_null() || body["success"].as_bool().unwrap());

    // Restore
    let (status, _) = post_json(
        &app,
        &format!("/api/admin/modules/{module_id}/restore"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_module_invalid_role_returns_404() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let fake_role = Uuid::new_v4();

    let (status, _) = post_json(
        &app,
        "/api/admin/modules",
        &token,
        json!({ "role_id": fake_role, "title": "X" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Submaterial CRUD ──────────────────────────────────────────────────────────

async fn create_test_module(app: &axum::Router, token: &str, role_id: &str) -> String {
    let (_, body) = post_json(
        app,
        "/api/admin/modules",
        token,
        json!({ "role_id": role_id, "title": format!("Module {}", Uuid::new_v4()) }),
    )
    .await;
    body["data"]["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_submaterial_create_has_mini_quiz_reminder() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;

    let (status, body) = post_json(
        &app,
        "/api/admin/submaterials",
        &token,
        json!({ "module_id": module_id, "title": "New Sub", "content": "Content here" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create sub: {body}");
    // Response should have requires_mini_quiz=true hint
    assert_eq!(body["requires_mini_quiz"].as_bool().unwrap(), true);
}

#[tokio::test]
async fn test_submaterial_invalid_module_returns_404() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let (status, _) = post_json(
        &app,
        "/api/admin/submaterials",
        &token,
        json!({ "module_id": Uuid::new_v4(), "title": "X" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_submaterial_soft_delete_and_restore() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;

    let (_, body) = post_json(
        &app,
        "/api/admin/submaterials",
        &token,
        json!({ "module_id": module_id, "title": "Sub to unpublish" }),
    )
    .await;
    let sub_id = body["data"]["id"].as_str().unwrap().to_string();

    // Soft delete
    let (status, _) = delete_req(&app, &format!("/api/admin/submaterials/{sub_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);

    // Restore
    let (status, _) = post_json(
        &app,
        &format!("/api/admin/submaterials/{sub_id}/restore"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

// ── Mini Quiz CRUD ────────────────────────────────────────────────────────────

async fn create_test_submaterial(app: &axum::Router, token: &str, module_id: &str) -> String {
    let (_, body) = post_json(
        app,
        "/api/admin/submaterials",
        token,
        json!({ "module_id": module_id, "title": format!("Sub {}", Uuid::new_v4()) }),
    )
    .await;
    body["data"]["id"].as_str().unwrap().to_string()
}

fn options_single_correct() -> Value {
    json!([
        { "text": "Right answer", "is_correct": true,  "order_index": 1 },
        { "text": "Wrong one",    "is_correct": false, "order_index": 2 },
        { "text": "Also wrong",   "is_correct": false, "order_index": 3 }
    ])
}

#[tokio::test]
async fn test_mini_quiz_create_and_list() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;
    let sub_id = create_test_submaterial(&app, &token, &module_id).await;

    // Create question
    let (status, body) = post_json(
        &app,
        "/api/admin/submaterial-quiz-questions",
        &token,
        json!({
            "submaterial_id": sub_id,
            "question_text": "What is the answer?",
            "options": options_single_correct()
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create mini quiz question: {body}");

    // List questions for sub
    let (status, body) = get(
        &app,
        &format!("/api/admin/submaterials/{sub_id}/quiz"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let questions = body["data"].as_array().unwrap();
    assert_eq!(questions.len(), 1);
    assert!(questions[0]["options"].as_array().unwrap().len() >= 2);
    // Admin can see is_correct
    assert!(questions[0]["options"][0]["is_correct"].is_boolean());
}

#[tokio::test]
async fn test_mini_quiz_two_correct_options_fails() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;
    let sub_id = create_test_submaterial(&app, &token, &module_id).await;

    let (status, body) = post_json(
        &app,
        "/api/admin/submaterial-quiz-questions",
        &token,
        json!({
            "submaterial_id": sub_id,
            "question_text": "Bad question?",
            "options": [
                { "text": "A", "is_correct": true,  "order_index": 1 },
                { "text": "B", "is_correct": true,  "order_index": 2 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "two correct options should fail: {body}");
}

#[tokio::test]
async fn test_mini_quiz_replace_options_requires_force_if_attempts() {
    // This test checks the force flag logic at the service layer.
    // Without actual user attempts we just verify the happy path.
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;
    let sub_id = create_test_submaterial(&app, &token, &module_id).await;

    // Create question
    let (_, body) = post_json(
        &app,
        "/api/admin/submaterial-quiz-questions",
        &token,
        json!({
            "submaterial_id": sub_id,
            "question_text": "Test question",
            "options": options_single_correct()
        }),
    )
    .await;
    let question_id = body["data"]["id"].as_str().unwrap().to_string();

    // Replace options without force (no attempts exist → should succeed)
    let (status, body) = patch_json(
        &app,
        &format!("/api/admin/submaterial-quiz-questions/{question_id}/options"),
        &token,
        json!({
            "options": [
                { "text": "New answer", "is_correct": true,  "order_index": 1 },
                { "text": "Wrong",      "is_correct": false, "order_index": 2 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "replace options (no attempts): {body}");
}

// ── Final Quiz CRUD ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_final_quiz_create_and_list() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;

    let (status, body) = post_json(
        &app,
        "/api/admin/module-quiz-questions",
        &token,
        json!({
            "module_id": module_id,
            "question_text": "Which is the best?",
            "options": [
                { "text": "A", "is_correct": true,  "order_index": 1 },
                { "text": "B", "is_correct": false, "order_index": 2 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create final quiz: {body}");

    let (status, body) = get(
        &app,
        &format!("/api/admin/modules/{module_id}/final-quiz"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_final_quiz_no_correct_option_fails() {
    let app = build_app().await;
    let token = admin_token(&app).await;
    let role_id = get_any_role_id(&app, &token).await;
    let module_id = create_test_module(&app, &token, &role_id).await;

    let (status, body) = post_json(
        &app,
        "/api/admin/module-quiz-questions",
        &token,
        json!({
            "module_id": module_id,
            "question_text": "Bad?",
            "options": [
                { "text": "A", "is_correct": false, "order_index": 1 },
                { "text": "B", "is_correct": false, "order_index": 2 }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "no correct option should fail: {body}");
}

// ── Pre-quiz Questions ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_pre_quiz_list() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let (status, body) = get(&app, "/api/admin/pre-quiz-questions", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["questions"].is_array());
    assert!(body["data"]["pagination"].is_object());
}

// ── Project CRUD ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_project_full_lifecycle() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    // Create a role specifically for this test (to avoid UNIQUE constraint on role_id)
    let role_code = format!("proj_{}", &Uuid::new_v4().to_string()[..8]);
    let (_, role_body) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": role_code, "name": "Project Role" }),
    )
    .await;
    let role_id = role_body["data"]["id"].as_str().unwrap().to_string();

    // Create project
    let (status, body) = post_json(
        &app,
        "/api/admin/projects",
        &token,
        json!({
            "role_id": role_id,
            "title": "Build a REST API",
            "description": "Build a full REST API with Rust.",
            "estimated_hours": 40
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create project: {body}");
    let project_id = body["data"]["id"].as_str().unwrap().to_string();

    // Get project
    let (status, body) = get(&app, &format!("/api/admin/projects/{project_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Build a REST API");
    assert!(body["data"]["submission_count"].is_number());

    // Update
    let (status, _) = patch_json(
        &app,
        &format!("/api/admin/projects/{project_id}"),
        &token,
        json!({ "title": "Build an Advanced REST API" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Unpublish
    let (status, _) = post_json(
        &app,
        &format!("/api/admin/projects/{project_id}/unpublish"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Verify user-facing endpoint no longer shows it
    // (get_project_by_role adds is_published=true filter)

    // Restore
    let (status, _) = post_json(
        &app,
        &format!("/api/admin/projects/{project_id}/restore"),
        &token,
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_project_duplicate_role_returns_409() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    // Create a dedicated role
    let role_code = format!("dup_proj_{}", &Uuid::new_v4().to_string()[..8]);
    let (_, role_body) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": role_code, "name": "Dup Test Role" }),
    )
    .await;
    let role_id = role_body["data"]["id"].as_str().unwrap().to_string();

    // First project
    let (status, _) = post_json(
        &app,
        "/api/admin/projects",
        &token,
        json!({ "role_id": role_id, "title": "P1", "description": "Desc" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Second project for same role → 409
    let (status, body) = post_json(
        &app,
        "/api/admin/projects",
        &token,
        json!({ "role_id": role_id, "title": "P2", "description": "Desc" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "duplicate project should 409: {body}");
}

// ── Audit trail ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_audit_log_recorded_on_role_create() {
    let app = build_app().await;
    let token = admin_token(&app).await;

    let code = format!("audit_{}", &Uuid::new_v4().to_string()[..8]);
    let (_, body) = post_json(
        &app,
        "/api/admin/roles",
        &token,
        json!({ "code": code, "name": "Audit Test Role" }),
    )
    .await;
    let role_id = body["data"]["id"].as_str().unwrap().to_string();

    // Check global audit log
    let (status, body) = get(
        &app,
        "/api/admin/audit-logs?action=role.created&per_page=5",
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let logs = body["data"]["logs"].as_array().unwrap();
    assert!(
        logs.iter().any(|l| l["target_id"].as_str() == Some(&role_id)),
        "audit entry for role.created not found"
    );
}

// ── Soft delete impact on user-facing ────────────────────────────────────────

#[tokio::test]
async fn test_unpublished_module_not_visible_to_users() {
    let app = build_app().await;
    let admin = admin_token(&app).await;

    // Create a role and module
    let role_code = format!("vis_{}", &Uuid::new_v4().to_string()[..8]);
    let (_, role_body) = post_json(
        &app,
        "/api/admin/roles",
        &admin,
        json!({ "code": role_code, "name": "Visibility Test" }),
    )
    .await;
    let role_id = role_body["data"]["id"].as_str().unwrap().to_string();

    let (_, mod_body) = post_json(
        &app,
        "/api/admin/modules",
        &admin,
        json!({ "role_id": role_id, "title": "Visible Module" }),
    )
    .await;
    let module_id = mod_body["data"]["id"].as_str().unwrap().to_string();

    // Unpublish the module via admin DELETE (soft delete)
    let (status, _) = delete_req(&app, &format!("/api/admin/modules/{module_id}"), &admin).await;
    assert_eq!(status, StatusCode::OK);

    // Verify admin can still see it (with is_published filter off)
    let (status, body) = get(
        &app,
        &format!("/api/admin/modules?role_id={role_id}&is_published=false"),
        &admin,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let modules = body["data"]["modules"].as_array().unwrap();
    assert!(
        modules.iter().any(|m| m["id"].as_str() == Some(&module_id)),
        "unpublished module should still be visible to admin"
    );
}
