use crate::{
    error::AppError,
    features::learning::submaterial_quiz::{dto::SubmitMiniQuizRequest, service},
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
    path = "/api/learning/submaterials/{id}/quiz",
    tag = "Learning - Mini Quiz",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    responses(
        (status = 200, description = "Mini quiz questions", body = crate::features::learning::submaterial_quiz::dto::MiniQuizResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// GET /api/learning/submaterials/:id/quiz
pub async fn get_mini_quiz(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(submaterial_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_mini_quiz(&state, auth.user_id, submaterial_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    post,
    path = "/api/learning/submaterials/{id}/quiz/submit",
    tag = "Learning - Mini Quiz",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    request_body = SubmitMiniQuizRequest,
    responses(
        (status = 200, description = "Mini quiz submitted", body = crate::features::learning::submaterial_quiz::dto::MiniQuizSubmitResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// POST /api/learning/submaterials/:id/quiz/submit
pub async fn submit_mini_quiz(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(submaterial_id): Path<Uuid>,
    Json(req): Json<SubmitMiniQuizRequest>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::submit_mini_quiz(&state, auth.user_id, submaterial_id, req)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}
