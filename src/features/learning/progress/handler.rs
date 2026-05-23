use crate::{
    error::AppError,
    features::learning::progress::service,
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/learning/progress",
    tag = "Learning - Modules",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Overall learning progress", body = crate::features::learning::progress::dto::OverallProgressResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
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
