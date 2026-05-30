use crate::{features, middleware, openapi, state::AppState};
use axum::middleware as axum_middleware;
use axum::{extract::State, http::{HeaderValue, Method, StatusCode}, response::IntoResponse, Json, Router, routing::get};
use serde_json::json;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::Span;
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

pub fn create_app(state: Arc<AppState>) -> Router {
    let allowed_origins = [
        "https://skill-up-kel-2.vercel.app".parse::<HeaderValue>().unwrap(),
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    let openapi_spec = openapi::build();

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &axum::http::Request<_>| {
            tracing::info_span!(
                "request",
                method = %req.method(),
                uri = %req.uri().path(),
                request_id = tracing::field::Empty,
            )
        })
        .on_request(|req: &axum::http::Request<_>, _span: &Span| {
            tracing::info!("--> {} {}", req.method(), req.uri().path());
        })
        .on_response(
            |res: &axum::http::Response<_>,
             latency: std::time::Duration,
             _span: &Span| {
                tracing::info!(
                    "<-- {} {}ms",
                    res.status().as_u16(),
                    latency.as_millis()
                );
            },
        );

    Router::new()
        .route("/health", get(health))
        .nest("/api", api_router())
        .merge(Scalar::with_url("/docs", openapi_spec.clone()))
        .merge(
            SwaggerUi::new("/swagger")
                .url("/api-docs/openapi.json", openapi_spec),
        )
        .layer(axum_middleware::from_fn(middleware::request_id::layer))
        .layer(trace_layer)
        .layer(cors)
        .with_state(state)
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let status = if db_ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    let body = json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "db":     if db_ok { "ok" } else { "unreachable" },
    });
    (status, Json(body))
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
