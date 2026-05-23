/// Integration test: Learning Part 2 — Mini Quiz + Final Quiz flow.
/// Requires a running PostgreSQL instance with migrations + seed data applied.
///
///   cargo test --test learning_quiz_test -- --nocapture
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
    let email = format!("quiz_flow_{}@example.com", uuid::Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "pass1234", "name": "Quiz Flow"}).to_string(),
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

/// Complete career quiz targeting frontend role.
async fn complete_career_quiz_frontend(app: &axum::Router, token: &str) {
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
    let attempt_id = att_body["data"]["attempt_id"].as_str().unwrap().to_string();

    for question in questions {
        let q_id = question["id"].as_str().unwrap();
        let order = question["order_index"].as_i64().unwrap();
        // Bias toward frontend role
        let opt_idx = if matches!(order, 4 | 6 | 8 | 9) { 3 } else { 0 };
        let o_id = question["options"][opt_idx]["id"].as_str().unwrap();
        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/quiz/attempts/{attempt_id}/answers"))
                    .header(header::AUTHORIZATION, bearer(token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({"question_id": q_id, "option_id": o_id}).to_string(),
                    ))
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
}

/// Submit mini quiz for a submaterial. First submits first options (may fail),
/// then reads correct answers from response and submits again with all correct.
/// Returns the final submit response body.
async fn complete_mini_quiz(app: &axum::Router, token: &str, sub_id: &str) -> Value {
    // GET quiz
    let quiz_resp = app
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
    assert_eq!(quiz_resp.status(), StatusCode::OK, "GET mini quiz failed for {sub_id}");
    let quiz_body = body_json(quiz_resp).await;
    let questions = quiz_body["data"]["questions"].as_array().unwrap();

    // Submit all first options (order_index=1, always wrong per seed design)
    let wrong_answers: Vec<Value> = questions
        .iter()
        .map(|q| {
            json!({
                "question_id": q["id"],
                "option_id": q["options"][0]["id"]
            })
        })
        .collect();

    let wrong_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/submaterials/{sub_id}/quiz/submit"))
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"answers": wrong_answers}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_resp.status(), StatusCode::OK, "wrong submit failed for {sub_id}");
    let wrong_body = body_json(wrong_resp).await;

    // Extract correct option IDs from the response
    let result_questions = wrong_body["data"]["questions"].as_array().unwrap();
    let correct_answers: Vec<Value> = result_questions
        .iter()
        .map(|q| {
            let correct_opt_id = q["correct_option_id"].as_str().unwrap();
            json!({
                "question_id": q["id"],
                "option_id": correct_opt_id
            })
        })
        .collect();

    // Submit with correct answers
    let correct_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/submaterials/{sub_id}/quiz/submit"))
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"answers": correct_answers}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(correct_resp.status(), StatusCode::OK, "correct submit failed for {sub_id}");
    let correct_body = body_json(correct_resp).await;

    assert_eq!(correct_body["data"]["passed"], true, "mini quiz should pass for {sub_id}");
    assert_eq!(
        correct_body["data"]["submaterial_completed"],
        true,
        "submaterial_completed should be true for {sub_id}"
    );

    correct_body
}

/// Submit final quiz. First submits wrong (first options), then correct. Returns final body.
async fn complete_final_quiz(app: &axum::Router, token: &str, module_id: &str) -> Value {
    // GET final quiz
    let quiz_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module_id}/quiz"))
                .header(header::AUTHORIZATION, bearer(token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(quiz_resp.status(), StatusCode::OK, "GET final quiz failed");
    let quiz_body = body_json(quiz_resp).await;
    let questions = quiz_body["data"]["questions"].as_array().unwrap();
    assert!(!questions.is_empty(), "final quiz should have questions");

    // Submit all first options → should score 0% (seed: correct is always option 2)
    let wrong_answers: Vec<Value> = questions
        .iter()
        .map(|q| json!({"question_id": q["id"], "option_id": q["options"][0]["id"]}))
        .collect();

    let wrong_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/modules/{module_id}/quiz/submit"))
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"answers": wrong_answers}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_resp.status(), StatusCode::OK);
    let wrong_body = body_json(wrong_resp).await;
    assert_eq!(
        wrong_body["data"]["passed"],
        false,
        "wrong answers should fail final quiz"
    );
    assert!(
        wrong_body["data"]["score"].as_f64().unwrap() < 70.0,
        "wrong answer score should be < 70%"
    );

    // Extract correct answers
    let result_questions = wrong_body["data"]["questions"].as_array().unwrap();
    let correct_answers: Vec<Value> = result_questions
        .iter()
        .map(|q| {
            let correct_opt_id = q["correct_option_id"].as_str().unwrap();
            json!({"question_id": q["id"], "option_id": correct_opt_id})
        })
        .collect();

    // Submit with correct answers
    let pass_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/modules/{module_id}/quiz/submit"))
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"answers": correct_answers}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(pass_resp.status(), StatusCode::OK);
    let pass_body = body_json(pass_resp).await;
    assert_eq!(pass_body["data"]["passed"], true, "correct answers should pass final quiz");

    pass_body
}

#[tokio::test]
async fn test_full_learning_quiz_flow() {
    let app = build_app().await;

    // ── Setup ─────────────────────────────────────────────────────────────────
    let token = register_and_get_token(&app).await;
    complete_career_quiz_frontend(&app, &token).await;

    // ── 1. GET modules → module1 unlocked, module2 locked ────────────────────
    let mods_resp = app
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
    assert_eq!(mods_resp.status(), StatusCode::OK);
    let mods_body = body_json(mods_resp).await;
    let modules = mods_body["data"]["modules"].as_array().unwrap();
    assert!(modules.len() >= 2);

    let m1 = &modules[0];
    let m2 = &modules[1];
    assert_eq!(m1["is_unlocked"], true);
    assert_eq!(m1["is_completed"], false);
    assert_eq!(m2["is_unlocked"], false);
    assert_eq!(m1["final_quiz_unlocked"], false, "final quiz locked before subs done");
    assert_eq!(m1["final_quiz_passed"], false);

    let module1_id = m1["id"].as_str().unwrap().to_string();
    let module2_id = m2["id"].as_str().unwrap().to_string();

    // ── 2. GET module1 detail → get submaterial IDs ───────────────────────────
    let m1_detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module1_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(m1_detail_resp.status(), StatusCode::OK);
    let m1_detail = body_json(m1_detail_resp).await;
    let subs = m1_detail["data"]["submaterials"].as_array().unwrap();
    assert!(subs.len() >= 3);

    let sub1_id = subs[0]["id"].as_str().unwrap().to_string();
    let sub2_id = subs[1]["id"].as_str().unwrap().to_string();
    let sub3_id = subs[2]["id"].as_str().unwrap().to_string();

    // ── 3. Attempt GET mini quiz for sub2 → 403 locked ───────────────────────
    let locked_quiz_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/submaterials/{sub2_id}/quiz"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        locked_quiz_resp.status(),
        StatusCode::FORBIDDEN,
        "mini quiz for sub2 should be locked"
    );

    // ── 4. Complete sub1 via mini quiz (wrong then correct) ───────────────────
    let sub1_result = complete_mini_quiz(&app, &token, &sub1_id).await;
    assert_eq!(sub1_result["data"]["score"].as_f64().unwrap(), 100.0);

    // ── 5. sub2 now unlocked → complete sub2 ─────────────────────────────────
    let sub2_result = complete_mini_quiz(&app, &token, &sub2_id).await;
    assert_eq!(sub2_result["data"]["passed"], true);

    // ── 6. Complete sub3 → all subs done but final quiz not yet ──────────────
    let _sub3_result = complete_mini_quiz(&app, &token, &sub3_id).await;

    // ── 7. GET modules → final_quiz_unlocked=true, module still not completed ─
    let mods_after_subs_resp = app
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
    let mods_after_subs = body_json(mods_after_subs_resp).await;
    let modules2 = mods_after_subs["data"]["modules"].as_array().unwrap();
    assert_eq!(modules2[0]["final_quiz_unlocked"], true, "final quiz should be unlocked");
    assert_eq!(modules2[0]["is_completed"], false, "module not complete until final quiz passed");
    assert_eq!(modules2[1]["is_unlocked"], false, "module2 still locked");

    // ── 8. GET final quiz for module2 → 403 (module2 locked) ─────────────────
    let locked_final_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module2_id}/quiz"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        locked_final_resp.status(),
        StatusCode::FORBIDDEN,
        "final quiz for locked module2 should be 403"
    );

    // ── 9. Submit final quiz for module1: wrong → fail, correct → pass ────────
    let _final_result = complete_final_quiz(&app, &token, &module1_id).await;

    // ── 10. GET modules → module1 completed, module2 unlocked ────────────────
    let mods_final_resp = app
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
    assert_eq!(mods_final_resp.status(), StatusCode::OK);
    let mods_final = body_json(mods_final_resp).await;
    let modules3 = mods_final["data"]["modules"].as_array().unwrap();
    assert_eq!(modules3[0]["is_completed"], true, "module1 should be completed");
    assert_eq!(modules3[0]["final_quiz_passed"], true, "final quiz passed");
    assert_eq!(modules3[1]["is_unlocked"], true, "module2 now unlocked");

    // ── 11. GET final quiz history → 2 attempts ──────────────────────────────
    let hist_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module1_id}/quiz/history"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hist_resp.status(), StatusCode::OK);
    let hist_body = body_json(hist_resp).await;
    assert_eq!(
        hist_body["data"]["attempts"].as_array().unwrap().len(),
        2,
        "should have 2 attempts (1 fail + 1 pass)"
    );
    assert!(
        hist_body["data"]["best_score"].as_f64().unwrap() >= 70.0,
        "best score should be >= 70"
    );

    // ── 12. GET progress → overall_percentage > 0 ────────────────────────────
    let prog_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/learning/progress")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(prog_resp.status(), StatusCode::OK);
    let prog_body = body_json(prog_resp).await;
    assert_eq!(prog_body["data"]["completed_modules"].as_i64().unwrap(), 1);
    assert!(prog_body["data"]["overall_percentage"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_final_quiz_locked_before_all_subs_done() {
    let app = build_app().await;

    let token = register_and_get_token(&app).await;
    complete_career_quiz_frontend(&app, &token).await;

    let mods_resp = app
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
    let mods_body = body_json(mods_resp).await;
    let module1_id = mods_body["data"]["modules"][0]["id"].as_str().unwrap().to_string();

    // Try to GET final quiz with 0 submaterials done → should fail
    let quiz_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module1_id}/quiz"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        quiz_resp.status(),
        StatusCode::FORBIDDEN,
        "final quiz should be locked when submaterials not complete"
    );
}

#[tokio::test]
async fn test_deprecated_complete_endpoint_returns_410() {
    let app = build_app().await;

    let token = register_and_get_token(&app).await;
    complete_career_quiz_frontend(&app, &token).await;

    let mods_resp = app
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
    let mods_body = body_json(mods_resp).await;
    let module1_id = mods_body["data"]["modules"][0]["id"].as_str().unwrap().to_string();

    let m1_detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module1_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let m1_detail = body_json(m1_detail_resp).await;
    let sub1_id = m1_detail["data"]["submaterials"][0]["id"].as_str().unwrap().to_string();

    let gone_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/submaterials/{sub1_id}/complete"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(gone_resp.status(), StatusCode::GONE, "deprecated endpoint should return 410");
}

#[tokio::test]
async fn test_incomplete_answers_rejected() {
    let app = build_app().await;

    let token = register_and_get_token(&app).await;
    complete_career_quiz_frontend(&app, &token).await;

    let mods_resp = app
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
    let mods_body = body_json(mods_resp).await;
    let module1_id = mods_body["data"]["modules"][0]["id"].as_str().unwrap().to_string();
    let m1_detail_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module1_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let m1_detail = body_json(m1_detail_resp).await;
    let sub1_id = m1_detail["data"]["submaterials"][0]["id"].as_str().unwrap().to_string();

    // Submit empty answers → should be 400 Bad Request
    let bad_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/submaterials/{sub1_id}/quiz/submit"))
                .header(header::AUTHORIZATION, bearer(&token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"answers": []}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        bad_resp.status(),
        StatusCode::BAD_REQUEST,
        "empty answers should return 400"
    );
}
