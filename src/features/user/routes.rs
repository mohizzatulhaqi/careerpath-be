use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

// TODO: implement user profile endpoints (GET /users/me, PATCH /users/me, etc.)
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
}
