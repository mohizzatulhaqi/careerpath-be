use crate::{features::auth::handler, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(handler::register))
        .route("/login",    post(handler::login))
        .route("/refresh",  post(handler::refresh))
        .route("/logout",   post(handler::logout))
        .route("/me",       get(handler::me))
}
