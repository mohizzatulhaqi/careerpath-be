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

#[utoipa::path(
    get,
    path = "/api/quiz",
    tag = "Quiz",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of quiz questions", body = Vec<crate::features::quiz::dto::QuestionDto>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_questions(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_questions(&state).await.map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    post,
    path = "/api/quiz/attempts",
    tag = "Quiz",
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Attempt created or resumed", body = crate::features::quiz::dto::AttemptCreatedResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn create_attempt(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::create_or_resume_attempt(&state, auth.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::created(data))
}

#[utoipa::path(
    post,
    path = "/api/quiz/attempts/{attempt_id}/answers",
    tag = "Quiz",
    security(("bearer_auth" = [])),
    params(
        ("attempt_id" = Uuid, Path, description = "Quiz attempt ID"),
    ),
    request_body = SubmitAnswerRequest,
    responses(
        (status = 200, description = "Answer saved", body = crate::features::quiz::dto::AnswerSavedResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attempt not found"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/quiz/attempts/{attempt_id}/submit",
    tag = "Quiz",
    security(("bearer_auth" = [])),
    params(
        ("attempt_id" = Uuid, Path, description = "Quiz attempt ID"),
    ),
    responses(
        (status = 200, description = "Attempt submitted", body = crate::features::quiz::dto::QuizResultResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attempt not found"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/quiz/attempts/{attempt_id}/result",
    tag = "Quiz",
    security(("bearer_auth" = [])),
    params(
        ("attempt_id" = Uuid, Path, description = "Quiz attempt ID"),
    ),
    responses(
        (status = 200, description = "Quiz result", body = crate::features::quiz::dto::QuizResultResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attempt not found"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/quiz/history",
    tag = "Quiz",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Quiz attempt history", body = Vec<crate::features::quiz::dto::HistoryItem>),
        (status = 401, description = "Unauthorized"),
    )
)]
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
