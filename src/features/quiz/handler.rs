use crate::{
    error::AppError,
    features::quiz::{dto::SubmitAnswerRequest, service},
    middleware::auth::AuthUser,
    shared::{pagination::PaginationQuery, response::ApiResponse},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_questions(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_questions(&state).await.map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn create_attempt(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::create_or_resume_attempt(&state, auth.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::created(data))
}

pub async fn save_answer(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
    Json(body): Json<SubmitAnswerRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::save_answer(
        &state,
        auth.user_id,
        attempt_id,
        body.question_id,
        body.option_id,
    )
    .await
    .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn submit_attempt(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::submit_attempt(&state, auth.user_id, attempt_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn get_result(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(attempt_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_result(&state, auth.user_id, attempt_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

pub async fn get_history(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_history(&state, auth.user_id, &pagination)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}
