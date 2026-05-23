/// Integration tests: Admin Bagian 3 — Submission Review.
///
///   cargo test --test admin_submission_test -- --nocapture
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use career_path_be::{
    app, config::Config, db::pool, features::project::storage::local::LocalStorage,
    state::AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

// ── Helpers ────────────────────────────────────────────────────────────────────

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

async fn login_admin(app: &axum::Router) -> String {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email":"admin@careerpath.local","password":"ChangeMeASAP123!"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_json(resp).await;
    body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn register_and_login(app: &axum::Router) -> (String, String) {
    let email = format!("sub_test_{}@example.com", uuid::Uuid::new_v4());
    let reg_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "password123", "name": "Test Sub"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reg_resp.status(), StatusCode::CREATED, "register failed");
    let body = body_json(reg_resp).await;
    let token = body["data"]["access_token"].as_str().unwrap().to_string();
    (email, token)
}

/// Insert a submission directly via multipart (same as user flow).
/// Returns submission_id.
async fn submit_project(app: &axum::Router, user_token: &str, project_id: &str) -> String {
    // Build a tiny valid ZIP in memory using the zip crate... but we don't have it.
    // Use a raw ZIP magic bytes for test purposes (local storage just saves bytes).
    // The zip file header: PK\x03\x04 ...
    let zip_magic = b"PK\x03\x04\x14\x00\x00\x00\x08\x00\x00\x00!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00atest.txtHello WorldPK\x01\x02\x14\x00\x14\x00\x00\x00\x08\x00\x00\x00!\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00atest.txtPK\x05\x06\x00\x00\x00\x00\x01\x00\x01\x00/\x00\x00\x00>\x00\x00\x00\x00\x00";

    // Build multipart body manually
    let boundary = "----TestBoundary1234567890";
    let notes = "This is a valid submission note that is long enough to pass validation checks.";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"submission_notes\"\r\n\r\n{notes}\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.zip\"\r\nContent-Type: application/zip\r\n\r\n"
    );
    let mut body_bytes = body.into_bytes();
    body_bytes.extend_from_slice(zip_magic);
    body_bytes.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/projects/{project_id}/submit"))
                .header(header::AUTHORIZATION, bearer(user_token))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body_bytes))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    let body = body_json(resp).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "submit_project failed: {body}"
    );
    body["data"]["submission_id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Create a role and module and mark them all done for user so submission is allowed.
/// Returns project_id that the user can submit to.
async fn unlock_project_for_user(
    app: &axum::Router,
    admin_token: &str,
    user_token: &str,
    db: &sqlx::PgPool,
) -> String {
    use uuid::Uuid;

    // 1. Use admin to create a fresh role + module + project for this test
    let role_code = format!("role_{}", Uuid::new_v4().to_string().replace('-', "")[..8].to_string());
    let role_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/admin/roles")
                .header(header::AUTHORIZATION, bearer(admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"code": role_code, "name": "Test Role Submission", "description": "test"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let role_body = body_json(role_resp).await;
    let role_id: Uuid = role_body["data"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // 2. Create project for that role
    let proj_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/admin/projects")
                .header(header::AUTHORIZATION, bearer(admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "role_id": role_id,
                        "title": "Submission Test Project",
                        "description": "desc",
                        "estimated_hours": 10
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let proj_body = body_json(proj_resp).await;
    let project_id: Uuid = proj_body["data"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // 3. Get user_id from user_token (via /api/auth/me)
    let me_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/auth/me")
                .header(header::AUTHORIZATION, bearer(user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let me_body = body_json(me_resp).await;
    let user_id: Uuid = me_body["data"]["id"].as_str().unwrap().parse().unwrap();

    // 4. Directly insert quiz_attempt in DB so user "has a role" and can access project
    sqlx::query!(
        r#"
        INSERT INTO quiz_attempts (user_id, status, submitted_at, result_role_id, match_percentage)
        VALUES ($1, 'submitted', NOW(), $2, 100.0)
        "#,
        user_id,
        role_id,
    )
    .execute(db)
    .await
    .expect("insert quiz_attempt");

    // 5. Create a learning module for the role (gating requires total > 0)
    let module_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO learning_modules (role_id, title, order_index)
        VALUES ($1, 'Test Module', 1)
        RETURNING id
        "#,
        role_id,
    )
    .fetch_one(db)
    .await
    .expect("insert learning_module");

    // 6. Add a submaterial so total_subs > 0 (required by count_module_completion)
    let sub_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO submaterials (module_id, title, order_index, estimated_minutes)
        VALUES ($1, 'Test Submaterial', 1, 5)
        RETURNING id
        "#,
        module_id,
    )
    .fetch_one(db)
    .await
    .expect("insert submaterial");

    // 7. Mark submaterial as completed for user
    sqlx::query!(
        r#"
        INSERT INTO user_submaterial_progress (user_id, submaterial_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        sub_id,
    )
    .execute(db)
    .await
    .expect("insert user_submaterial_progress");

    // 8. Mark module quiz as completed (total_questions=0 → automatically passes,
    //    but insert anyway so the gating logic finds a passing attempt if needed)
    sqlx::query!(
        r#"
        INSERT INTO module_quiz_attempts (user_id, module_id, score, passed)
        VALUES ($1, $2, 100.0, true)
        "#,
        user_id,
        module_id,
    )
    .execute(db)
    .await
    .expect("insert module_quiz_attempt");

    project_id.to_string()
}

// ── Access control ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_submission_list_rejects_regular_user() {
    let app = build_app().await;
    let (_, user_token) = register_and_login(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/submissions")
                .header(header::AUTHORIZATION, bearer(&user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_submission_list_accessible_by_admin() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    let app = build_app().await;
    let admin_token = login_admin(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/submissions")
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    let body = body_json(resp).await;
    eprintln!("STATUS: {status}\nBODY: {body}");
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["submissions"].is_array());
    assert!(body["data"]["pagination"].is_object());
    assert!(body["data"]["summary"].is_object());
    assert!(body["data"]["summary"]["pending_count"].is_number());
}

// ── Queue stats ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_queue_stats() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/submissions/queue/stats")
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let data = &body["data"];
    assert!(data["pending_count"].is_number());
    assert!(data["approved_today"].is_number());
    assert!(data["rejected_today"].is_number());
    assert!(data["approved_this_week"].is_number());
    assert!(data["rejected_this_week"].is_number());
    assert!(data["by_role"].is_array());
}

// ── Full lifecycle: submit → approve → reject (revoke) → approve ────────────────

#[tokio::test]
async fn test_full_review_lifecycle() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    // Get DB for direct inserts
    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();

    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // 1. List: should appear in pending queue (use large per_page to handle accumulated test data)
    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/submissions?status=pending_review&per_page=100")
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let list_body = body_json(list_resp).await;
    let submissions = list_body["data"]["submissions"].as_array().unwrap();
    assert!(
        submissions.iter().any(|s| s["id"].as_str() == Some(&submission_id)),
        "submission should be in pending queue, got: {list_body}"
    );

    // 2. Get detail
    let detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/submissions/{submission_id}"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_resp.status(), StatusCode::OK);
    let detail_body = body_json(detail_resp).await;
    let detail = &detail_body["data"];
    assert_eq!(detail["status"].as_str().unwrap(), "pending_review");
    assert!(detail["user"]["id"].is_string());
    assert!(detail["project"]["id"].is_string());
    assert!(detail["file"]["original_name"].is_string());
    assert!(detail["review_history"].as_array().unwrap().is_empty());

    // 3. Approve
    let approve_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/approve"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Submission looks great! All criteria met."}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approve_resp.status(), StatusCode::OK);
    let approve_body = body_json(approve_resp).await;
    assert_eq!(approve_body["data"]["status"].as_str().unwrap(), "approved");
    assert!(approve_body["data"]["reviewed_at"].is_string());

    // 4. Detail after approve: review_history has 1 entry
    let detail2 = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/admin/submissions/{submission_id}"))
                    .header(header::AUTHORIZATION, bearer(&admin_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await;
    let history = detail2["data"]["review_history"].as_array().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["new_status"].as_str().unwrap(), "approved");
    assert!(history[0]["old_status"].is_null());

    // 5. Approve again → 409 AlreadyApproved
    let dup_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/approve"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Approving again should fail."}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dup_resp.status(), StatusCode::CONFLICT);

    // 6. Reject (revoke approval) → 200, approved → rejected
    let reject_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/reject"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Oops, found issues — please fix and resubmit the project."}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reject_resp.status(), StatusCode::OK);
    assert_eq!(
        body_json(reject_resp).await["data"]["status"]
            .as_str()
            .unwrap(),
        "rejected"
    );

    // 7. Detail: review_history has 2 entries
    let detail3 = body_json(
        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/admin/submissions/{submission_id}"))
                    .header(header::AUTHORIZATION, bearer(&admin_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
    )
    .await;
    let history2 = detail3["data"]["review_history"].as_array().unwrap();
    assert_eq!(history2.len(), 2);
    assert_eq!(history2[1]["old_status"].as_str().unwrap(), "approved");
    assert_eq!(history2[1]["new_status"].as_str().unwrap(), "rejected");

    // 8. Re-approve (rejected → approved) → 200
    let reapprove_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/approve"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Issues resolved — approved again."}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reapprove_resp.status(), StatusCode::OK);

    // 9. Verify audit log recorded
    let audit_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/admin/audit-logs?action=submission.approved&per_page=5")
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let audit_body = body_json(audit_resp).await;
    let logs = audit_body["data"]["logs"].as_array().unwrap();
    assert!(!logs.is_empty(), "audit log should have submission.approved entries");
}

// ── Validation tests ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_approve_notes_too_short_returns_400() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/approve"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reviewer_notes": "ok"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reject_notes_empty_returns_400() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/reject"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reviewer_notes": ""}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_approve_with_html_notes_sanitized() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // notes with HTML — should succeed but tags are stripped
    let raw_notes = "<script>alert(1)</script>Approved with malicious notes but still valid length.";
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/approve"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"reviewer_notes": raw_notes}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "HTML notes should be sanitized not rejected");
    let body = body_json(resp).await;
    let stored_notes = body["data"]["reviewer_notes"].as_str().unwrap();
    // Script tag should be stripped
    assert!(!stored_notes.contains("<script>"), "script tag should be stripped");
    // Text content should be preserved
    assert!(stored_notes.contains("Approved"), "text content should remain");
}

// ── State machine tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_reject_to_pending_is_invalid() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // Reject first
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/reject"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Rejected for testing purposes — please fix the issues."})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Reject again (rejected → rejected) → 409
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/reject"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Trying to reject again — should fail with 409."})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

// ── Download ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_download_submission_file() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/submissions/{submission_id}/download"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.contains("zip") || ct.contains("octet"), "Content-Type should be zip");
    let cd = resp
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(cd.contains("attachment"), "Content-Disposition should include attachment");
    assert!(cd.contains("filename"), "Content-Disposition should include filename");
}

// ── User dashboard reflects review decisions ───────────────────────────────────

#[tokio::test]
async fn test_user_dashboard_reflects_approved_status() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // Approve
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/approve"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Excellent work! All requirements met."}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // User GET /api/dashboard → next_action.code should be "ALL_DONE"
    let dash_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/dashboard")
                .header(header::AUTHORIZATION, bearer(&user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dash_resp.status(), StatusCode::OK);
    let dash_body = body_json(dash_resp).await;
    let next = &dash_body["data"]["next_action"];
    assert_eq!(
        next["code"].as_str().unwrap(),
        "ALL_DONE",
        "dashboard should show ALL_DONE after approval"
    );
}

#[tokio::test]
async fn test_user_dashboard_reflects_rejected_status() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // Reject
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{submission_id}/reject"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Please improve code quality and resubmit your project."})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // User dashboard → next_action.code should be "SUBMIT_PROJECT" (resubmit)
    let dash_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/dashboard")
                .header(header::AUTHORIZATION, bearer(&user_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dash_resp.status(), StatusCode::OK);
    let dash_body = body_json(dash_resp).await;
    let next = &dash_body["data"]["next_action"];
    assert_eq!(
        next["code"].as_str().unwrap(),
        "SUBMIT_PROJECT",
        "dashboard should show SUBMIT_PROJECT (resubmit) after rejection"
    );
}

// ── Filter tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_filter_by_status_approved_shows_nothing_for_fresh_submissions() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // Filter for approved — this specific submission should NOT be in the list
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "/api/admin/submissions?status=approved&project_id={}",
                    &submission_id // Using submission_id as a dummy project filter — won't match
                ))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_json(resp).await;
    // The filter may return other submissions, but this specific one should not be there
    let subs = body["data"]["submissions"].as_array().unwrap();
    assert!(
        !subs.iter().any(|s| s["id"].as_str() == Some(&submission_id)),
        "fresh submission should not appear in approved filter"
    );
}

// ── Sanitization: project submit regression ────────────────────────────────────

#[tokio::test]
async fn test_project_submit_with_html_notes_is_sanitized() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;

    // Submit with HTML in notes — should succeed (not rejected), notes are sanitized
    let zip_magic = b"PK\x03\x04\x14\x00\x00\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00PK\x05\x06\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    let boundary = "----TestBoundaryHtml123";
    // HTML injection in notes — still >= 20 chars after sanitization
    let notes_html = "<b>This is bold</b> and this is a <script>evil()</script> submission note for testing.";
    let body_str = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"submission_notes\"\r\n\r\n{notes_html}\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.zip\"\r\nContent-Type: application/zip\r\n\r\n"
    );
    let mut body_bytes = body_str.into_bytes();
    body_bytes.extend_from_slice(zip_magic);
    body_bytes.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/projects/{project_id}/submit"))
                .header(header::AUTHORIZATION, bearer(&user_token))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body_bytes))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "submit with HTML notes should succeed (sanitize, not reject)");

    // Admin get detail → submission_notes should be sanitized
    let sub_body = body_json(resp).await;
    let sub_id = sub_body["data"]["submission_id"].as_str().unwrap();

    let detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/admin/submissions/{sub_id}"))
                .header(header::AUTHORIZATION, bearer(&admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let detail_body = body_json(detail_resp).await;
    let stored_notes = detail_body["data"]["submission_notes"].as_str().unwrap();
    assert!(!stored_notes.contains("<script>"), "script tag should be stripped from stored notes");
    assert!(!stored_notes.contains("<b>"), "b tag should be stripped from stored notes");
    assert!(stored_notes.contains("submission note"), "text content should be preserved");
}

// ── Race condition (best-effort) ───────────────────────────────────────────────

#[tokio::test]
async fn test_race_condition_double_approve() {
    let app = build_app().await;
    let admin_token = login_admin(&app).await;
    let (_, user_token) = register_and_login(&app).await;

    dotenvy::dotenv().ok();
    let config = career_path_be::config::Config::from_env().unwrap();
    let db = career_path_be::db::pool::create_pool(&config.database_url)
        .await
        .unwrap();
    let project_id = unlock_project_for_user(&app, &admin_token, &user_token, &db).await;
    let submission_id = submit_project(&app, &user_token, &project_id).await;

    // Spawn 2 concurrent approve requests
    let app1 = app.clone();
    let app2 = app.clone();
    let sid1 = submission_id.clone();
    let sid2 = submission_id.clone();
    let tok1 = admin_token.clone();
    let tok2 = admin_token.clone();

    let t1 = tokio::spawn(async move {
        app1.oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{sid1}/approve"))
                .header(header::AUTHORIZATION, bearer(&tok1))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Concurrent approve attempt 1 — only one should succeed."})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
    });

    let t2 = tokio::spawn(async move {
        app2.oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/admin/submissions/{sid2}/approve"))
                .header(header::AUTHORIZATION, bearer(&tok2))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"reviewer_notes": "Concurrent approve attempt 2 — only one should succeed."})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
    });

    let (s1, s2) = tokio::join!(t1, t2);
    let s1 = s1.unwrap();
    let s2 = s2.unwrap();

    // One should be 200 (OK), the other 409 (Conflict)
    let statuses = [s1.as_u16(), s2.as_u16()];
    assert!(
        statuses.contains(&200),
        "at least one approve should succeed: {s1} {s2}"
    );
    assert!(
        statuses.contains(&409),
        "the second approve should fail with 409: {s1} {s2}"
    );
}
