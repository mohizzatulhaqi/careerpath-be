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

#[utoipa::path(
    get,
    path = "/api/learning/modules/{id}/quiz",
    tag = "Learning - Final Quiz",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Final quiz questions", body = crate::features::learning::module_quiz::dto::FinalQuizResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/learning/modules/{id}/quiz/submit",
    tag = "Learning - Final Quiz",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    request_body = SubmitFinalQuizRequest,
    responses(
        (status = 200, description = "Final quiz submitted", body = crate::features::learning::module_quiz::dto::FinalQuizSubmitResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/learning/modules/{id}/quiz/history",
    tag = "Learning - Final Quiz",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Final quiz attempt history", body = crate::features::learning::module_quiz::dto::FinalQuizHistoryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
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
