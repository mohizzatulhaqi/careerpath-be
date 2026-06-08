use crate::{features::user::handler, state::AppState};
use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(handler::get_profile).patch(handler::update_profile))
        .route("/me/summary", get(handler::get_profile_summary))
}
