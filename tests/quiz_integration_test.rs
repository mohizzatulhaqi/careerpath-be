/// Integration test: full quiz flow.
/// Requires a running PostgreSQL instance with migrations + seed data applied.
/// Set DATABASE_URL and JWT_SECRET in .env before running.
///
///   cargo test --test quiz_integration_test -- --nocapture
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

#[tokio::test]
async fn test_full_quiz_flow() {
    let app = build_app().await;

    // ── 1. Register a fresh test user ─────────────────────────────────────────
    let email = format!("quiz_test_{}@example.com", uuid::Uuid::new_v4());
    let reg_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"email": email, "password": "password123", "name": "Quiz Tester"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reg_resp.status(), StatusCode::CREATED);
    let reg_body = body_json(reg_resp).await;
    let token = reg_body["data"]["access_token"].as_str().unwrap().to_string();

    // ── 2. GET /api/quiz/questions ────────────────────────────────────────────
    let q_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/quiz/questions")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(q_resp.status(), StatusCode::OK);
    let q_body = body_json(q_resp).await;
    let questions = q_body["data"].as_array().expect("questions array");
    assert!(!questions.is_empty(), "seed data should have questions");

    // ── 3. Create attempt ─────────────────────────────────────────────────────
    let att_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/quiz/attempts")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(att_resp.status(), StatusCode::CREATED);
    let att_body = body_json(att_resp).await;
    let attempt_id = att_body["data"]["attempt_id"].as_str().unwrap().to_string();

    // ── 4. Resume attempt (idempotent) ────────────────────────────────────────
    let att_resp2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/quiz/attempts")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let att_body2 = body_json(att_resp2).await;
    assert_eq!(
        att_body2["data"]["attempt_id"].as_str().unwrap(),
        attempt_id,
        "should resume same attempt"
    );

    // ── 5. Answer all questions (pick first option of each) ───────────────────
    for question in questions {
        let q_id = question["id"].as_str().unwrap();
        let o_id = question["options"][0]["id"].as_str().unwrap();

        let ans_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/quiz/attempts/{attempt_id}/answers"))
                    .header(header::AUTHORIZATION, bearer(&token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({"question_id": q_id, "option_id": o_id}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ans_resp.status(), StatusCode::OK, "answer save failed for q {q_id}");
        let ans_body = body_json(ans_resp).await;
        assert_eq!(ans_body["data"]["saved"], true);
    }

    // ── 6. Submit ─────────────────────────────────────────────────────────────
    let sub_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/quiz/attempts/{attempt_id}/submit"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub_resp.status(), StatusCode::OK, "submit failed");
    let sub_body = body_json(sub_resp).await;
    assert!(sub_body["data"]["role"]["code"].is_string(), "role.code missing");
    let pct = sub_body["data"]["match_percentage"].as_f64().unwrap();
    assert!(pct > 0.0, "match_percentage should be > 0");
    assert!(!sub_body["data"]["all_scores"].as_array().unwrap().is_empty());

    // ── 7. Submit again → 409 ─────────────────────────────────────────────────
    let sub_again = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/quiz/attempts/{attempt_id}/submit"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(sub_again.status(), StatusCode::CONFLICT);

    // ── 8. GET result ─────────────────────────────────────────────────────────
    let res_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/quiz/attempts/{attempt_id}/result"))
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_resp.status(), StatusCode::OK);
    let res_body = body_json(res_resp).await;
    assert_eq!(
        res_body["data"]["attempt_id"].as_str().unwrap(),
        attempt_id
    );
    assert!(!res_body["data"]["top_reasons"].as_array().unwrap().is_empty());

    // ── 9. GET history ────────────────────────────────────────────────────────
    let hist_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/quiz/history?page=1&per_page=10")
                .header(header::AUTHORIZATION, bearer(&token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(hist_resp.status(), StatusCode::OK);
    let hist_body = body_json(hist_resp).await;
    assert_eq!(hist_body["data"]["total"].as_i64().unwrap(), 1);
    assert_eq!(hist_body["data"]["items"].as_array().unwrap().len(), 1);
}
