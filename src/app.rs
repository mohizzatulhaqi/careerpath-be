use crate::{features, state::AppState};
use axum::{http::Method, Router};
use std::sync::Arc;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

pub fn create_app(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    Router::new()
        .nest("/api", api_router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth",      features::auth::routes::router())
        .nest("/users",     features::user::routes::router())
        .nest("/quiz",      features::quiz::routes::router())
        .nest("/learning",  features::learning::routes::router())
        .nest("/projects",  features::project::routes::router())
        .nest("/dashboard", features::dashboard::routes::router())
        .nest("/admin",     features::admin::routes::router())
}
