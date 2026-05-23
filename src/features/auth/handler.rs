use crate::{
    error::AppError,
    features::auth::{
        dto::{LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest},
        service,
    },
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;
use validator::Validate;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let data = service::register(&state, body).await.map_err(AppError::from)?;
    Ok(ApiResponse::created(data))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let data = service::login(&state, body).await.map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::me(&state, auth_user.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let data = service::refresh(&state, &body.refresh_token)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    // AuthUser ensures the request carries a valid access token
    _auth_user: AuthUser,
    Json(body): Json<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    service::logout(&state, &body.refresh_token)
        .await
        .map_err(AppError::from)?;

    Ok(ApiResponse::ok(serde_json::json!({ "message": "Logged out successfully" })))
}
