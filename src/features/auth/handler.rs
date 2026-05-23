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

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = crate::features::auth::dto::AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists"),
    )
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let data = service::register(&state, body).await.map_err(AppError::from)?;
    Ok(ApiResponse::created(data))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = crate::features::auth::dto::AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let data = service::login(&state, body).await.map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user info", body = crate::features::auth::dto::UserResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn me(
    State(state): State<Arc<AppState>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::me(&state, auth_user.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = crate::features::auth::dto::TokenResponse),
        (status = 401, description = "Invalid or expired refresh token"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth",
    security(("bearer_auth" = [])),
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 401, description = "Unauthorized"),
    )
)]
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
