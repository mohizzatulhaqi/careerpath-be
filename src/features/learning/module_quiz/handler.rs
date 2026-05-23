use crate::{
    error::AppError,
    features::learning::module_quiz::{dto::SubmitFinalQuizRequest, service},
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// GET /api/learning/modules/:id/quiz
pub async fn get_final_quiz(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(module_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_final_quiz(&state, auth.user_id, module_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

/// POST /api/learning/modules/:id/quiz/submit
pub async fn submit_final_quiz(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(module_id): Path<Uuid>,
    Json(req): Json<SubmitFinalQuizRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::submit_final_quiz(&state, auth.user_id, module_id, req)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

/// GET /api/learning/modules/:id/quiz/history
pub async fn get_final_quiz_history(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(module_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_final_quiz_history(&state, auth.user_id, module_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}
