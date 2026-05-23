use crate::{
    error::AppError,
    features::learning::progress::service,
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

/// GET /api/learning/progress
pub async fn get_progress(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_overall_progress(&state, auth.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}
