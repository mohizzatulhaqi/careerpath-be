use crate::{features::dashboard::handler, state::AppState};
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handler::get_dashboard))
        .route("/learning-summary", get(handler::get_learning_summary))
        .route("/activity", get(handler::get_activity_log))
}
