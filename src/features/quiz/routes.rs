use crate::{features::quiz::handler, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/questions",                  get(handler::get_questions))
        .route("/attempts",                   post(handler::create_attempt))
        .route("/attempts/:id/answers",       post(handler::save_answer))
        .route("/attempts/:id/submit",        post(handler::submit_attempt))
        .route("/attempts/:id/result",        get(handler::get_result))
        .route("/history",                    get(handler::get_history))
}
