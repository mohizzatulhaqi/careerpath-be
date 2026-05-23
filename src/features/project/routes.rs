use crate::{features::project::handler, state::AppState};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

const UPLOAD_LIMIT_BYTES: usize = 26_214_400; // 25 MB

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(handler::get_my_project))
        .route("/:id", get(handler::get_project))
        .route(
            "/:id/submit",
            post(handler::submit_project).layer(DefaultBodyLimit::max(UPLOAD_LIMIT_BYTES)),
        )
        .route("/:id/submissions", get(handler::get_submissions))
        .route(
            "/submissions/:submission_id/download",
            get(handler::download_submission),
        )
}
