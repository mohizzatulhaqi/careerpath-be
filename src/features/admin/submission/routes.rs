use crate::{features::admin::submission::handler, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Queue stats — MUST be registered before /:id routes to avoid ambiguity
        .route("/submissions/queue/stats", get(handler::queue_stats))
        // List & detail
        .route("/submissions", get(handler::list_submissions))
        .route("/submissions/:id", get(handler::get_submission))
        // Download
        .route("/submissions/:id/download", get(handler::download_submission))
        // Review decisions
        .route("/submissions/:id/approve", post(handler::approve_submission))
        .route("/submissions/:id/reject", post(handler::reject_submission))
}
