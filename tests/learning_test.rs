/// Integration test: learning flow with strict gating (regression suite for Part 1 + Part 2).
/// Requires a running PostgreSQL instance with migrations + seed data applied.
///
///   cargo test --test learning_test -- --nocapture
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

async fn register_user(app: &axum::Router, suffix: &str) -> String {
    let email = format!("learn_test_{suffix}_{}@example.com", uuid::Uuid::new_v4());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "password123", "name": "Learn Tester"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = body_json(resp).await;
    body["data"]["access_token"].as_str().unwrap().to_string()
}

async fn complete_quiz_for_frontend(app: &axum::Router, token: &str) {
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
    assert_eq!(q_resp.status(), StatusCode::OK);
    let q_body = body_json(q_resp).await;
    let questions = q_body["data"].as_array().expect("questions array");

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
        let option_idx = match order {
            4 | 6 | 8 | 9 => 3,
            _ => 0,
        };
        let o_id = question["options"][option_idx]["id"].as_str().unwrap();
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

    let sub_resp = app
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
    assert_eq!(sub_resp.status(), StatusCode::OK, "quiz submit failed");
}

/// Complete a submaterial by submitting the mini quiz with correct answers.
/// Strategy: submit first options (likely wrong per seed), read correct answers, re-submit.
async fn complete_sub_via_mini_quiz(app: &axum::Router, token: &str, sub_id: &str) {
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
    assert_eq!(quiz_resp.status(), StatusCode::OK);
    let quiz_body = body_json(quiz_resp).await;
    let questions = quiz_body["data"]["questions"].as_array().unwrap();

    // First submission (wrong) to get correct answers
    let first_answers: Vec<Value> = questions
        .iter()
        .map(|q| json!({"question_id": q["id"], "option_id": q["options"][0]["id"]}))
        .collect();

    let first_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/learning/submaterials/{sub_id}/quiz/submit"))
                .header(header::AUTHORIZATION, bearer(token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"answers": first_answers}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first_resp.status(), StatusCode::OK);
    let first_body = body_json(first_resp).await;

    // If already passed (first options happened to be correct), done
    if first_body["data"]["passed"] == true {
        return;
    }

    // Re-submit with correct answers from response
    let result_qs = first_body["data"]["questions"].as_array().unwrap();
    let correct_answers: Vec<Value> = result_qs
        .iter()
        .map(|q| json!({"question_id": q["id"], "option_id": q["correct_option_id"]}))
        .collect();

    let pass_resp = app
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
    assert_eq!(pass_resp.status(), StatusCode::OK);
    let pass_body = body_json(pass_resp).await;
    assert_eq!(pass_body["data"]["passed"], true, "should pass mini quiz for {sub_id}");
}

/// Complete a module's final quiz with correct answers (read from wrong submission response).
async fn complete_final_quiz(app: &axum::Router, token: &str, module_id: &str) {
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
    assert_eq!(quiz_resp.status(), StatusCode::OK, "GET final quiz should be 200");
    let quiz_body = body_json(quiz_resp).await;
    let questions = quiz_body["data"]["questions"].as_array().unwrap();

    let first_answers: Vec<Value> = questions
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
                .body(Body::from(json!({"answers": first_answers}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(wrong_resp.status(), StatusCode::OK);
    let wrong_body = body_json(wrong_resp).await;

    if wrong_body["data"]["passed"] == true {
        return;
    }

    let result_qs = wrong_body["data"]["questions"].as_array().unwrap();
    let correct_answers: Vec<Value> = result_qs
        .iter()
        .map(|q| json!({"question_id": q["id"], "option_id": q["correct_option_id"]}))
        .collect();

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
    assert_eq!(pass_body["data"]["passed"], true, "final quiz should pass");
}

#[tokio::test]
async fn test_full_learning_flow() {
    let app = build_app().await;

    let token = register_user(&app, "flow").await;
    complete_quiz_for_frontend(&app, &token).await;

    // ── 1. GET /api/learning/modules → module 1 unlocked, module 2 locked ─────
    let mod_resp = app
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
    assert_eq!(mod_resp.status(), StatusCode::OK);
    let mod_body = body_json(mod_resp).await;

    let role = &mod_body["data"]["role"];
    assert!(role["code"].as_str().is_some(), "role.code should exist");

    let modules = mod_body["data"]["modules"].as_array().unwrap();
    assert!(modules.len() >= 2, "should have at least 2 modules");

    let m1 = &modules[0];
    let m2 = &modules[1];
    assert_eq!(m1["is_unlocked"], true, "module 1 should be unlocked");
    assert_eq!(m1["is_completed"], false, "module 1 should not be completed");
    assert_eq!(m2["is_unlocked"], false, "module 2 should be locked");

    let module1_id = m1["id"].as_str().unwrap().to_string();
    let module2_id = m2["id"].as_str().unwrap().to_string();

    // ── 2. GET /api/learning/modules/:id for module 1 → 200 ──────────────────
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
    let submaterials = m1_detail["data"]["submaterials"].as_array().unwrap();
    assert!(submaterials.len() >= 3, "module 1 should have 3 submaterials");

    let sub1_id = submaterials[0]["id"].as_str().unwrap().to_string();
    let sub2_id = submaterials[1]["id"].as_str().unwrap().to_string();
    let sub3_id = submaterials[2]["id"].as_str().unwrap().to_string();

    assert_eq!(submaterials[0]["is_unlocked"], true, "sub 1 should be unlocked");
    assert_eq!(submaterials[1]["is_unlocked"], false, "sub 2 should be locked");

    // ── 3. GET /api/learning/submaterials/:id for sub 1 → 200 ────────────────
    let sub1_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/submaterials/{sub1_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub1_resp.status(), StatusCode::OK);
    let sub1_body = body_json(sub1_resp).await;
    assert!(sub1_body["data"]["content"].as_str().unwrap().len() > 100, "should have content");

    // ── 4. GET /api/learning/submaterials/:id for sub 2 → 403 locked ─────────
    let sub2_locked_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/submaterials/{sub2_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub2_locked_resp.status(), StatusCode::FORBIDDEN, "sub 2 should be locked");

    // ── 5. GET /api/learning/modules/:id for module 2 → 403 locked ───────────
    let m2_locked_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{module2_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(m2_locked_resp.status(), StatusCode::FORBIDDEN, "module 2 should be locked");

    // ── 6. POST /complete → 410 Gone (deprecated) ────────────────────────────
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
    assert_eq!(gone_resp.status(), StatusCode::GONE, "complete endpoint should be 410");

    // ── 7. Complete sub1, sub2, sub3 via mini quiz ────────────────────────────
    complete_sub_via_mini_quiz(&app, &token, &sub1_id).await;
    complete_sub_via_mini_quiz(&app, &token, &sub2_id).await;
    complete_sub_via_mini_quiz(&app, &token, &sub3_id).await;

    // ── 8. All subs done → final quiz unlocked but module not yet complete ─────
    let mod_after_subs = app
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
    let mod_after_subs_body = body_json(mod_after_subs).await;
    let mods_after = mod_after_subs_body["data"]["modules"].as_array().unwrap();
    assert_eq!(mods_after[0]["final_quiz_unlocked"], true, "final quiz should be unlocked");
    assert_eq!(mods_after[0]["is_completed"], false, "module not complete before final quiz");
    assert_eq!(mods_after[1]["is_unlocked"], false, "module 2 still locked");

    // ── 9. Submit final quiz for module 1 → module 1 completed ───────────────
    complete_final_quiz(&app, &token, &module1_id).await;

    // ── 10. GET /api/learning/modules → module 2 now unlocked ────────────────
    let mod_resp2 = app
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
    assert_eq!(mod_resp2.status(), StatusCode::OK);
    let mod_body2 = body_json(mod_resp2).await;
    let modules2 = mod_body2["data"]["modules"].as_array().unwrap();
    assert_eq!(modules2[0]["is_completed"], true, "module 1 should be completed");
    assert_eq!(modules2[1]["is_unlocked"], true, "module 2 should now be unlocked");

    // ── 11. GET /api/learning/progress → overall progress ────────────────────
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
    assert!(prog_body["data"]["overall_percentage"].as_f64().unwrap() > 0.0);
    assert_eq!(prog_body["data"]["completed_modules"].as_i64().unwrap(), 1);
    assert!(prog_body["data"]["current_module"].is_object(), "should have current module");
}

#[tokio::test]
async fn test_cross_role_access_denied() {
    let app = build_app().await;

    let token = register_user(&app, "cross").await;
    complete_quiz_for_frontend(&app, &token).await;

    let mod_resp = app
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
    assert_eq!(mod_resp.status(), StatusCode::OK);
    let mod_body = body_json(mod_resp).await;
    let role_code = mod_body["data"]["role"]["code"].as_str().unwrap();
    assert!(
        role_code == "frontend" || role_code == "uiux",
        "expected frontend-like role, got {role_code}"
    );

    let fake_id = uuid::Uuid::new_v4();
    let cross_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/learning/modules/{fake_id}"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_quiz_not_completed() {
    let app = build_app().await;

    let token = register_user(&app, "noquiz").await;

    let resp = app
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
    assert_eq!(
        resp.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "should be 422 when quiz not completed"
    );
}
