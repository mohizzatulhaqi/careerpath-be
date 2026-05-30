pub mod dto;
pub mod handler;
pub mod routes;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    routes::router()
}
