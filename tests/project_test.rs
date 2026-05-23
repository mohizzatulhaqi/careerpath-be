/// Integration test: full project flow (submit, gating, resubmit, download).
/// Requires a running PostgreSQL instance with migrations + seed data applied.
///
///   cargo test --test project_test -- --nocapture
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use career_path_be::{
    app, config::Config, db::pool,
    features::project::storage::local::LocalStorage,
    state::AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn build_app() -> axum::Router {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let db = pool::create_pool(&config.database_url).await.expect("db pool");
    let storage = Arc::new(
        LocalStorage::new(config.storage_root.clone()).expect("storage"),
    );
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

async fn register_and_get_token(app: &axum::Router) -> String {
    let email = format!("project_test_{}@example.com", uuid::Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "pass1234", "name": "Project Flow"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    body_json(resp).await["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Helper to fully complete learning progress for frontend role by direct DB manipulation
/// This speeds up the test significantly instead of running through all submaterial quizzes manually.
async fn cheat_complete_frontend_learning(db: &sqlx::PgPool, user_id: uuid::Uuid) {
    let frontend_role_id = uuid::Uuid::parse_str("00000001-0000-0000-0000-000000000001").unwrap();

    // 1. Set user role via quiz_attempts
    sqlx::query!(
        "INSERT INTO quiz_attempts (user_id, status, result_role_id) VALUES ($1, 'submitted', $2)",
        user_id,
        frontend_role_id
    )
    .execute(db)
    .await
    .unwrap();

    // 2. Mark all submaterials as completed
    sqlx::query!(
        r#"
        INSERT INTO user_submaterial_progress (user_id, submaterial_id)
        SELECT $1, s.id
        FROM submaterials s
        JOIN learning_modules lm ON lm.id = s.module_id
        WHERE lm.role_id = $2
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        frontend_role_id
    )
    .execute(db)
    .await
    .unwrap();

    // 3. Mark all module quizzes as passed (score 100)
    sqlx::query!(
        r#"
        INSERT INTO module_quiz_attempts (module_id, user_id, score)
        SELECT lm.id, $1, 100
        FROM learning_modules lm
        WHERE lm.role_id = $2
        "#,
        user_id,
        frontend_role_id
    )
    .execute(db)
    .await
    .unwrap();
}

async fn create_multipart_request(
    uri: &str,
    token: &str,
    file_name: &str,
    file_content: &[u8],
    notes: &str,
) -> Request<Body> {
    let boundary = "------------------------Boundaryxyz123";
    let mut body = Vec::new();

    if !notes.is_empty() {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"submission_notes\"\r\n\r\n");
        body.extend_from_slice(notes.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    if !file_content.is_empty() {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
                file_name
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: application/zip\r\n\r\n");
        body.extend_from_slice(file_content);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::AUTHORIZATION, bearer(token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn test_full_project_flow() {
    let app = build_app().await;

    // We need the DB to cheat progress
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let db = pool::create_pool(&config.database_url).await.expect("db pool");

    // ── 1. Create a user who hasn't completed modules ────────────────────────
    let token_locked = register_and_get_token(&app).await;

    // Simulate taking role quiz to be assigned frontend role
    let jwt_secret = &config.jwt_secret;
    let claims = career_path_be::shared::jwt::verify_token(&token_locked, jwt_secret).unwrap();
    let user_locked_id = uuid::Uuid::parse_str(&claims.sub).unwrap();
    let frontend_role_id = uuid::Uuid::parse_str("00000001-0000-0000-0000-000000000001").unwrap();

    sqlx::query!(
        "INSERT INTO quiz_attempts (user_id, status, result_role_id) VALUES ($1, 'submitted', $2)",
        user_locked_id,
        frontend_role_id
    )
    .execute(&db)
    .await
    .unwrap();

    let my_proj_resp_locked = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/projects/me")
                .header(header::AUTHORIZATION, bearer(&token_locked))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(my_proj_resp_locked.status(), StatusCode::OK);
    let my_proj_locked = body_json(my_proj_resp_locked).await;
    let project_id = my_proj_locked["data"]["project"]["id"].as_str().unwrap().to_string();

    assert_eq!(
        my_proj_locked["data"]["project"]["access_status"].as_str().unwrap(),
        "locked"
    );

    // Trying to submit while locked should fail 403
    let submit_locked_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token_locked,
        "dummy.zip",
        b"PK\x03\x04test",
        "This is my minimum 20 chars notes",
    ).await;
    let submit_locked_resp = app.clone().oneshot(submit_locked_req).await.unwrap();
    assert_eq!(submit_locked_resp.status(), StatusCode::FORBIDDEN);

    // ── 2. Create a user who HAS completed modules ───────────────────────────
    let token = register_and_get_token(&app).await;
    let claims = career_path_be::shared::jwt::verify_token(&token, jwt_secret).unwrap();
    let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap();

    cheat_complete_frontend_learning(&db, user_id).await;

    let my_proj_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/projects/me")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(my_proj_resp.status(), StatusCode::OK);
    let my_proj = body_json(my_proj_resp).await;

    assert_eq!(
        my_proj["data"]["project"]["access_status"].as_str().unwrap(),
        "available"
    );

    // ── 3. Submit Validation Failures ────────────────────────────────────────

    // a. No file
    let submit_nofile_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token,
        "",
        b"",
        "This is my minimum 20 chars notes",
    ).await;
    let submit_nofile_resp = app.clone().oneshot(submit_nofile_req).await.unwrap();
    assert_eq!(submit_nofile_resp.status(), StatusCode::BAD_REQUEST);

    // b. Invalid File Type
    let submit_pdffile_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token,
        "dummy.pdf",
        b"%PDF-dummy",
        "This is my minimum 20 chars notes",
    ).await;
    let submit_pdffile_resp = app.clone().oneshot(submit_pdffile_req).await.unwrap();
    assert_eq!(submit_pdffile_resp.status(), StatusCode::BAD_REQUEST);

    // c. Notes too short
    let submit_shortnotes_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token,
        "dummy.zip",
        b"PK\x03\x04dummy",
        "short",
    ).await;
    let submit_shortnotes_resp = app.clone().oneshot(submit_shortnotes_req).await.unwrap();
    assert_eq!(submit_shortnotes_resp.status(), StatusCode::BAD_REQUEST);

    // ── 4. Successful Submit ─────────────────────────────────────────────────
    let submit_ok_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token,
        "my_project.zip",
        b"PK\x03\x04test_content_zip",
        "This is my final project submission. Please review.",
    ).await;
    let submit_ok_resp = app.clone().oneshot(submit_ok_req).await.unwrap();
    assert_eq!(submit_ok_resp.status(), StatusCode::OK);
    let submit_ok = body_json(submit_ok_resp).await;

    assert_eq!(submit_ok["data"]["status"].as_str().unwrap(), "pending_review");
    assert_eq!(submit_ok["data"]["file_original_name"].as_str().unwrap(), "my_project.zip");

    // ── 5. Check Project Access again ────────────────────────────────────────
    let my_proj_after = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/projects/me")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let my_proj_after_body = body_json(my_proj_after).await;
    assert_eq!(
        my_proj_after_body["data"]["project"]["access_status"].as_str().unwrap(),
        "pending_review"
    );

    // ── 6. Submitting again while pending review fails ───────────────────────
    let submit_again_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token,
        "my_project2.zip",
        b"PK\x03\x04test_content_zip",
        "This is my final project submission. Please review.",
    ).await;
    let submit_again_resp = app.clone().oneshot(submit_again_req).await.unwrap();
    assert_eq!(submit_again_resp.status(), StatusCode::CONFLICT);

    // ── 7. Get submission history ────────────────────────────────────────────
    let history_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/api/projects/{}/submissions", project_id))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(history_resp.status(), StatusCode::OK);
    let history_body = body_json(history_resp).await;
    let submissions = history_body["data"].as_array().unwrap();
    assert_eq!(submissions.len(), 1);

    let submission_id = submissions[0]["id"].as_str().unwrap();

    // ── 8. Download Submission ───────────────────────────────────────────────
    let dl_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/api/projects/submissions/{}/download", submission_id))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dl_resp.status(), StatusCode::OK);
    let content_type = dl_resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
    assert_eq!(content_type, "application/zip");
    
    let content_disp = dl_resp.headers().get(header::CONTENT_DISPOSITION).unwrap().to_str().unwrap();
    assert!(content_disp.contains("my_project.zip"));

    let dl_bytes = axum::body::to_bytes(dl_resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(dl_bytes, b"PK\x03\x04test_content_zip".as_ref());

    // ── 9. Test cross-user download fails ────────────────────────────────────
    let dl_fail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/api/projects/submissions/{}/download", submission_id))
                .header(header::AUTHORIZATION, bearer(&token_locked)) // wrong user
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dl_fail_resp.status(), StatusCode::FORBIDDEN);

    // ── 10. Resubmit flow: Set to rejected via DB, then resubmit ─────────────
    sqlx::query!(
        "UPDATE project_submissions SET status = 'rejected' WHERE id = $1",
        uuid::Uuid::parse_str(submission_id).unwrap()
    )
    .execute(&db)
    .await
    .unwrap();

    // Now access_status should be rejected
    let my_proj_rej = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/projects/me")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let my_proj_rej_body = body_json(my_proj_rej).await;
    assert_eq!(
        my_proj_rej_body["data"]["project"]["access_status"].as_str().unwrap(),
        "rejected"
    );

    // Resubmit should succeed
    let resubmit_req = create_multipart_request(
        &format!("/api/projects/{}/submit", project_id),
        &token,
        "resubmission.zip",
        b"PK\x03\x04test2",
        "Resubmission with fixes. Min 20 chars.",
    ).await;
    let resubmit_resp = app.clone().oneshot(resubmit_req).await.unwrap();
    assert_eq!(resubmit_resp.status(), StatusCode::OK);

    // History should now have 2 items
    let history2_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(&format!("/api/projects/{}/submissions", project_id))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let history2_body = body_json(history2_resp).await;
    assert_eq!(history2_body["data"].as_array().unwrap().len(), 2);
}
