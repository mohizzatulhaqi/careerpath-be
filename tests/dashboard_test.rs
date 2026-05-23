/// Integration tests: Dashboard feature.
///
/// Tests 7 user states (A-G), activity log, learning-summary,
/// and consistency with /api/learning/progress.
///
///   cargo test --test dashboard_test -- --nocapture
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use career_path_be::{app, config::Config, db::pool, features::project::storage::local::LocalStorage, state::AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// ── App setup ─────────────────────────────────────────────────────────────

async fn build_app_and_pool() -> (axum::Router, PgPool) {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("config");
    let db = pool::create_pool(&config.database_url).await.expect("db pool");
    let storage = Arc::new(LocalStorage::new(config.storage_root.clone()).expect("storage"));
    let state = Arc::new(AppState { db: db.clone(), config: Arc::new(config), storage });
    (app::create_app(state), db)
}

// ── HTTP helpers ───────────────────────────────────────────────────────────

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("body bytes");
    serde_json::from_slice(&bytes).expect("body json")
}

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

async fn register_user(app: &axum::Router, name: &str) -> (String, String) {
    let email = format!("dash_{}_{name}@example.com", Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "pass1234", "name": name}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED, "register {name} failed");
    let body = body_json(resp).await;
    let token = body["data"]["access_token"].as_str().unwrap().to_string();
    let user_id = body["data"]["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

/// Complete the career quiz biased toward frontend role. Returns the role_id string.
async fn complete_career_quiz_frontend(app: &axum::Router, token: &str) -> String {
    let q_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/quiz/questions")
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let q_body = body_json(q_resp).await;
    let questions = q_body["data"].as_array().unwrap();

    let att_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/quiz/attempts")
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let att_body = body_json(att_resp).await;
    let attempt_id = att_body["data"]["attempt_id"].as_str().unwrap();

    for q in questions {
        let q_id = q["id"].as_str().unwrap();
        let order = q["order_index"].as_i64().unwrap();
        let opt_idx = if matches!(order, 4 | 6 | 8 | 9) { 3 } else { 0 };
        let o_id = q["options"][opt_idx]["id"].as_str().unwrap();
        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/quiz/attempts/{attempt_id}/answers"))
                    .header(header::AUTHORIZATION, bearer(token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({"question_id": q_id, "option_id": o_id}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let sub = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/quiz/attempts/{attempt_id}/submit"))
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub.status(), StatusCode::OK, "career quiz submit failed");
    let sub_body = body_json(sub).await;
    sub_body["data"]["role"]["id"].as_str().unwrap_or("").to_string()
}

/// Complete one mini quiz via the API (wrong first, then correct).
async fn complete_one_mini_quiz(app: &axum::Router, token: &str, sub_id: &str) {
    // GET quiz to get questions
    let qr = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/submaterials/{sub_id}/quiz"))
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(qr.status(), StatusCode::OK);
    let qb = body_json(qr).await;
    let questions = qb["data"]["questions"].as_array().unwrap();

    // Submit wrong (all first options)
    let wrong: Vec<Value> = questions.iter().map(|q| json!({
        "question_id": q["id"],
        "option_id": q["options"][0]["id"]
    })).collect();
    let wr = app.clone().oneshot(
        Request::builder().method(Method::POST)
            .uri(format!("/api/learning/submaterials/{sub_id}/quiz/submit"))
            .header(header::AUTHORIZATION, bearer(token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"answers": wrong}).to_string()))
            .unwrap(),
    ).await.unwrap();
    let wb = body_json(wr).await;

    // Submit correct (second option per seed design)
    let correct: Vec<Value> = wb["data"]["questions"].as_array().unwrap().iter().map(|q| {
        json!({"question_id": q["id"], "option_id": q["correct_option_id"]})
    }).collect();
    let cr = app.clone().oneshot(
        Request::builder().method(Method::POST)
            .uri(format!("/api/learning/submaterials/{sub_id}/quiz/submit"))
            .header(header::AUTHORIZATION, bearer(token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"answers": correct}).to_string()))
            .unwrap(),
    ).await.unwrap();
    let cb = body_json(cr).await;
    assert_eq!(cb["data"]["passed"], true, "mini quiz should pass for {sub_id}");
}

/// Seed all user submaterial progress directly in DB for a given role.
async fn seed_all_submaterial_progress(db: &PgPool, user_id: &str, role_id: &str) {
    let user_uuid = Uuid::parse_str(user_id).unwrap();
    let role_uuid = Uuid::parse_str(role_id).unwrap();

    // Get all submaterial IDs for this role
    let sub_ids: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT s.id FROM submaterials s
        JOIN learning_modules m ON m.id = s.module_id
        WHERE m.role_id = $1 AND m.is_published = true
        ORDER BY m.order_index, s.order_index
        "#,
    )
    .bind(role_uuid)
    .fetch_all(db)
    .await
    .expect("fetch submaterials");

    for sub_id in sub_ids {
        sqlx::query(
            "INSERT INTO user_submaterial_progress (user_id, submaterial_id)
             VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_uuid)
        .bind(sub_id)
        .execute(db)
        .await
        .expect("insert progress");
    }
}

/// Seed passing final quiz attempts for all modules in a role.
async fn seed_all_final_quizzes_passed(db: &PgPool, user_id: &str, role_id: &str) {
    let user_uuid = Uuid::parse_str(user_id).unwrap();
    let role_uuid = Uuid::parse_str(role_id).unwrap();

    let module_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM learning_modules WHERE role_id = $1 AND is_published = true",
    )
    .bind(role_uuid)
    .fetch_all(db)
    .await
    .expect("fetch modules");

    for module_id in module_ids {
        // Only insert if there's a final quiz for this module
        let q_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM module_quizzes WHERE module_id = $1",
        )
        .bind(module_id)
        .fetch_one(db)
        .await
        .unwrap_or(0);

        if q_count > 0 {
            sqlx::query(
                "INSERT INTO module_quiz_attempts (user_id, module_id, score, passed)
                 VALUES ($1, $2, 80.0, true)",
            )
            .bind(user_uuid)
            .bind(module_id)
            .execute(db)
            .await
            .expect("insert final quiz attempt");
        }
    }
}

/// Seed a project submission for the user.
async fn seed_project_submission(
    db: &PgPool,
    user_id: &str,
    role_id: &str,
    status: &str,
) {
    let user_uuid = Uuid::parse_str(user_id).unwrap();
    let role_uuid = Uuid::parse_str(role_id).unwrap();

    let project_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM projects WHERE role_id = $1 AND is_published = true LIMIT 1",
    )
    .bind(role_uuid)
    .fetch_optional(db)
    .await
    .expect("fetch project");

    if let Some(pid) = project_id {
        sqlx::query(
            r#"INSERT INTO project_submissions
               (project_id, user_id, file_path, file_original_name, file_size_bytes,
                file_mime_type, submission_notes, status)
               VALUES ($1, $2, 'test/path.zip', 'test.zip', 1024,
                       'application/zip', 'Integration test seeded submission.', $3)"#,
        )
        .bind(pid)
        .bind(user_uuid)
        .bind(status)
        .execute(db)
        .await
        .expect("insert project submission");
    }
}

/// GET /api/dashboard helper
async fn get_dashboard(app: &axum::Router, token: &str) -> Value {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/dashboard")
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "GET /dashboard failed");
    body_json(resp).await
}

// ── Tests ─────────────────────────────────────────────────────────────────

/// User A: fresh register, no quiz taken → next_action = TAKE_QUIZ
#[tokio::test]
async fn test_user_a_no_quiz_take_quiz() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "UserA").await;

    let body = get_dashboard(&app, &token).await;
    assert_eq!(body["data"]["career_path"]["has_taken_quiz"], false);
    assert_eq!(body["data"]["next_action"]["code"], "TAKE_QUIZ");
    assert_eq!(body["data"]["next_action"]["target_url"], "/quiz");
    // learning_progress should be zero-state, not error
    assert_eq!(body["data"]["learning_progress"]["total_modules"], 0);
    assert_eq!(body["data"]["learning_progress"]["overall_percentage"], 0.0);
}

/// User B: quiz taken, no modules started → next_action = CONTINUE_LEARNING (first submaterial)
#[tokio::test]
async fn test_user_b_quiz_done_no_modules() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "UserB").await;

    complete_career_quiz_frontend(&app, &token).await;

    let body = get_dashboard(&app, &token).await;
    assert_eq!(body["data"]["career_path"]["has_taken_quiz"], true);
    assert_eq!(body["data"]["next_action"]["code"], "CONTINUE_LEARNING");

    let target = body["data"]["next_action"]["target_url"].as_str().unwrap();
    assert!(target.starts_with("/learning/submaterials/"), "target_url should point to a submaterial, got: {target}");
}

/// User C: mid-module (one submaterial done) → CONTINUE_LEARNING with next submaterial
#[tokio::test]
async fn test_user_c_partial_module() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "UserC").await;
    complete_career_quiz_frontend(&app, &token).await;

    // Get first module's first submaterial
    let modules_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/learning/modules")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let mb = body_json(modules_resp).await;
    let first_module_id = mb["data"]["modules"][0]["id"].as_str().unwrap();

    // Get first module detail to find submaterials
    let detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{first_module_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let db_detail = body_json(detail_resp).await;
    let first_sub_id = db_detail["data"]["submaterials"][0]["id"].as_str().unwrap();

    // Complete first submaterial
    complete_one_mini_quiz(&app, &token, first_sub_id).await;

    let body = get_dashboard(&app, &token).await;
    assert_eq!(body["data"]["next_action"]["code"], "CONTINUE_LEARNING");

    // Target should be the SECOND submaterial now
    let target = body["data"]["next_action"]["target_url"].as_str().unwrap();
    assert!(target.starts_with("/learning/submaterials/"));
    // The target should NOT be the first sub (already completed)
    assert!(!target.ends_with(first_sub_id), "should point to next sub, not completed one");

    // Verify current_module is populated in learning_progress
    assert!(body["data"]["learning_progress"]["current_module"].is_object());
    assert!(body["data"]["learning_progress"]["current_module"]["current_submaterial"].is_object());
}

/// User D: all submaterials in module 1 done but final quiz not taken → TAKE_FINAL_QUIZ
#[tokio::test]
async fn test_user_d_all_subs_done_final_quiz_pending() {
    let (app, db) = build_app_and_pool().await;
    let (token, user_id) = register_user(&app, "UserD").await;
    let role_id = complete_career_quiz_frontend(&app, &token).await;

    // Need to complete only module 1's submaterials
    // Get module 1 submaterials
    let modules_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/learning/modules")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let mb = body_json(modules_resp).await;
    let first_module_id = mb["data"]["modules"][0]["id"].as_str().unwrap().to_string();

    let detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{first_module_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let db_detail = body_json(detail_resp).await;
    let subs = db_detail["data"]["submaterials"].as_array().unwrap();

    // Complete all submaterials in module 1 via API
    for sub in subs {
        let sub_id = sub["id"].as_str().unwrap();
        complete_one_mini_quiz(&app, &token, sub_id).await;
    }

    // Don't complete final quiz → module not completed

    let body = get_dashboard(&app, &token).await;
    assert_eq!(
        body["data"]["next_action"]["code"], "TAKE_FINAL_QUIZ",
        "Expected TAKE_FINAL_QUIZ but got: {} (role: {})", body["data"]["next_action"]["code"], role_id
    );
    let target = body["data"]["next_action"]["target_url"].as_str().unwrap();
    assert!(target.contains("/final-quiz"), "target should include /final-quiz, got: {target}");

    // Suppress unused warning
    let _ = user_id;
    let _ = db;
}

/// User E: all modules completed, no project submission → SUBMIT_PROJECT
#[tokio::test]
async fn test_user_e_all_done_submit_project() {
    let (app, db) = build_app_and_pool().await;
    let (token, user_id) = register_user(&app, "UserE").await;
    let role_id = complete_career_quiz_frontend(&app, &token).await;

    if role_id.is_empty() {
        eprintln!("SKIP: career quiz didn't return role_id (seed mismatch)");
        return;
    }

    // Seed all progress directly
    seed_all_submaterial_progress(&db, &user_id, &role_id).await;
    seed_all_final_quizzes_passed(&db, &user_id, &role_id).await;

    let body = get_dashboard(&app, &token).await;
    assert_eq!(body["data"]["next_action"]["code"], "SUBMIT_PROJECT",
        "Expected SUBMIT_PROJECT, got: {}", body["data"]["next_action"]["code"]);
    let target = body["data"]["next_action"]["target_url"].as_str().unwrap();
    assert!(target.starts_with("/projects/"), "target should be /projects/<id>, got: {target}");

    // final_project should show is_available = true
    assert_eq!(body["data"]["final_project"]["is_available"], true);
    assert_eq!(body["data"]["final_project"]["access_status"], "available");
}

/// User F: project pending_review → WAIT_REVIEW
#[tokio::test]
async fn test_user_f_project_pending_review() {
    let (app, db) = build_app_and_pool().await;
    let (token, user_id) = register_user(&app, "UserF").await;
    let role_id = complete_career_quiz_frontend(&app, &token).await;

    if role_id.is_empty() {
        eprintln!("SKIP: career quiz didn't return role_id");
        return;
    }

    seed_all_submaterial_progress(&db, &user_id, &role_id).await;
    seed_all_final_quizzes_passed(&db, &user_id, &role_id).await;
    seed_project_submission(&db, &user_id, &role_id, "pending_review").await;

    let body = get_dashboard(&app, &token).await;
    assert_eq!(body["data"]["next_action"]["code"], "WAIT_REVIEW");
    assert_eq!(body["data"]["final_project"]["access_status"], "pending_review");
}

/// User G: project approved → ALL_DONE
#[tokio::test]
async fn test_user_g_project_approved() {
    let (app, db) = build_app_and_pool().await;
    let (token, user_id) = register_user(&app, "UserG").await;
    let role_id = complete_career_quiz_frontend(&app, &token).await;

    if role_id.is_empty() {
        eprintln!("SKIP: career quiz didn't return role_id");
        return;
    }

    seed_all_submaterial_progress(&db, &user_id, &role_id).await;
    seed_all_final_quizzes_passed(&db, &user_id, &role_id).await;
    seed_project_submission(&db, &user_id, &role_id, "approved").await;

    let body = get_dashboard(&app, &token).await;
    assert_eq!(body["data"]["next_action"]["code"], "ALL_DONE");
    assert_eq!(body["data"]["final_project"]["access_status"], "approved");
}

/// Test recent_activities: verify ordering (DESC) and structure.
#[tokio::test]
async fn test_recent_activities_structure() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "ActUser").await;
    complete_career_quiz_frontend(&app, &token).await;

    // Complete a couple of submaterials to generate activities
    let modules_resp = app.clone().oneshot(
        Request::builder().method(Method::GET).uri("/api/learning/modules")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    let mb = body_json(modules_resp).await;
    let mod_id = mb["data"]["modules"][0]["id"].as_str().unwrap();
    let detail_resp = app.clone().oneshot(
        Request::builder().method(Method::GET).uri(format!("/api/learning/modules/{mod_id}"))
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    let dd = body_json(detail_resp).await;
    let subs = dd["data"]["submaterials"].as_array().unwrap();

    // Complete at least 2 submaterials
    for sub in subs.iter().take(2) {
        complete_one_mini_quiz(&app, &token, sub["id"].as_str().unwrap()).await;
    }

    let body = get_dashboard(&app, &token).await;
    let activities = body["data"]["recent_activities"].as_array().unwrap();
    assert!(!activities.is_empty(), "should have activities after completing submaterials");

    // All activities should have required fields
    for act in activities {
        assert!(act["kind"].is_string());
        assert!(act["title"].is_string());
        assert!(act["detail"].is_string());
        assert!(act["occurred_at"].is_string());
    }

    // Verify ordering: first occurrence should be >= second (DESC)
    if activities.len() >= 2 {
        let t0 = activities[0]["occurred_at"].as_str().unwrap();
        let t1 = activities[1]["occurred_at"].as_str().unwrap();
        assert!(t0 >= t1, "activities should be ordered DESC by time");
    }

    // Max 5 activities on main dashboard
    assert!(activities.len() <= 5, "dashboard should return at most 5 activities");
}

/// Test GET /dashboard/activity with limit query param.
#[tokio::test]
async fn test_activity_log_endpoint() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "ActLog").await;

    // Test without limit → default 20
    let resp = app.clone().oneshot(
        Request::builder().method(Method::GET)
            .uri("/api/dashboard/activity")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert!(body["data"]["activities"].is_array());
    assert!(body["data"]["has_more"].is_boolean());

    // Test with limit=1
    let resp2 = app.clone().oneshot(
        Request::builder().method(Method::GET)
            .uri("/api/dashboard/activity?limit=1")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let body2 = body_json(resp2).await;
    let acts = body2["data"]["activities"].as_array().unwrap();
    assert!(acts.len() <= 1);
}

/// Test GET /dashboard/learning-summary (requires quiz taken).
#[tokio::test]
async fn test_learning_summary_endpoint() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "SumUser").await;
    complete_career_quiz_frontend(&app, &token).await;

    let resp = app.clone().oneshot(
        Request::builder().method(Method::GET)
            .uri("/api/dashboard/learning-summary")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;

    assert!(body["data"]["role"]["id"].is_string());
    assert!(body["data"]["overall_percentage"].is_number());
    let modules = body["data"]["modules"].as_array().unwrap();
    assert!(!modules.is_empty(), "should have modules for frontend role");

    for m in modules {
        assert!(m["id"].is_string());
        assert!(m["is_unlocked"].is_boolean());
        assert!(m["is_completed"].is_boolean());
        assert!(m["final_quiz_passed"].is_boolean());
        assert!(m["final_quiz_unlocked"].is_boolean());
    }
}

/// learning-summary should return 403 for user without quiz.
#[tokio::test]
async fn test_learning_summary_requires_quiz() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "NoQuizSum").await;

    let resp = app.clone().oneshot(
        Request::builder().method(Method::GET)
            .uri("/api/dashboard/learning-summary")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    // Should be 403 (quiz not completed)
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

/// Consistency: learning_progress in dashboard matches /api/learning/progress.
#[tokio::test]
async fn test_dashboard_learning_progress_consistency() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "Consist").await;
    complete_career_quiz_frontend(&app, &token).await;

    let dash_body = get_dashboard(&app, &token).await;
    let dash_lp = &dash_body["data"]["learning_progress"];

    let prog_resp = app.clone().oneshot(
        Request::builder().method(Method::GET)
            .uri("/api/learning/progress")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(prog_resp.status(), StatusCode::OK);
    let prog_body = body_json(prog_resp).await;

    // total_modules and completed_modules should match
    assert_eq!(
        dash_lp["total_modules"],
        prog_body["data"]["total_modules"],
        "total_modules mismatch"
    );
    assert_eq!(
        dash_lp["completed_modules"],
        prog_body["data"]["completed_modules"],
        "completed_modules mismatch"
    );
    assert_eq!(
        dash_lp["overall_percentage"],
        prog_body["data"]["overall_percentage"],
        "overall_percentage mismatch"
    );
}

/// Dashboard must return 200 (not panic) for fresh user with no data.
#[tokio::test]
async fn test_dashboard_no_panic_fresh_user() {
    let (app, _db) = build_app_and_pool().await;
    let (token, _uid) = register_user(&app, "Fresh").await;

    let resp = app.clone().oneshot(
        Request::builder().method(Method::GET)
            .uri("/api/dashboard")
            .header(header::AUTHORIZATION, bearer(&token))
            .body(Body::empty()).unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_json(resp).await;
    // All sections present
    assert!(body["data"]["user"].is_object());
    assert!(body["data"]["career_path"].is_object());
    assert!(body["data"]["learning_progress"].is_object());
    assert!(body["data"]["final_project"].is_object());
    assert!(body["data"]["next_action"].is_object());
    assert!(body["data"]["recent_activities"].is_array());
}
