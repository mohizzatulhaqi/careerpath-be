use crate::{features::auth::handler, middleware::rate_limit, state::AppState};
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    // 10 requests per minute per IP for login/register/refresh
    let limiter = rate_limit::new_limiter(10);

    let throttled = Router::new()
        .route("/login",   post(handler::login))
        .route("/register", post(handler::register))
        .route("/refresh", post(handler::refresh))
        .layer(axum_middleware::from_fn(move |req, next| {
            let limiter = limiter.clone();
            async move { rate_limit::check(limiter, req, next).await }
        }));

    Router::new()
        .merge(throttled)
        .route("/logout", post(handler::logout))
        .route("/me",     get(handler::me))
}
