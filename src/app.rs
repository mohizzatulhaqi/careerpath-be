use crate::{features, openapi, state::AppState};
use axum::{http::Method, Router, routing::get};
use std::sync::Arc;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

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

    let openapi_spec = openapi::build();

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api", api_router())
        // Scalar UI at /docs
        .merge(Scalar::with_url("/docs", openapi_spec.clone()))
        // Swagger UI at /swagger + raw JSON at /api-docs/openapi.json
        .merge(
            SwaggerUi::new("/swagger")
                .url("/api-docs/openapi.json", openapi_spec),
        )
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth",         features::auth::routes::router())
        .nest("/users",        features::user::routes::router())
        .nest("/quiz",         features::quiz::routes::router())
        .nest("/learning",     features::learning::routes::router())
        .nest("/projects",     features::project::routes::router())
        .nest("/dashboard",    features::dashboard::routes::router())
        .nest("/admin",        features::admin::routes::router())
        .nest("/certificates", features::certificate::routes::router())
        .nest("/verify",       features::certificate::routes::verify_router())
}
