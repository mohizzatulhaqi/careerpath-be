use crate::{
    error::AppError,
    features::learning::module::service,
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

/// GET /api/learning/modules
pub async fn list_modules(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_modules_for_user(&state, auth.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

/// GET /api/learning/modules/:id
pub async fn get_module(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(module_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_module_detail(&state, auth.user_id, module_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}
