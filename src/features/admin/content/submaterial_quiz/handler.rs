use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    features::admin::content::submaterial_quiz::{
        dto::{CreateSubQuizQuestionRequest, ReplaceOptionsRequest, UpdateSubQuizQuestionRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[derive(Debug, Deserialize, Default)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

/// GET /api/admin/submaterials/:submaterial_id/quiz
pub async fn list_questions(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submaterial_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_questions(&state.db, submaterial_id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

/// POST /api/admin/submaterial-quiz-questions
pub async fn create_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSubQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_question(&state.db, admin.0.user_id, &req)
        .await
        .map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

/// PATCH /api/admin/submaterial-quiz-questions/:id
pub async fn update_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSubQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::update_question(&state.db, admin.0.user_id, id, &req)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question updated" })))
}

/// PATCH /api/admin/submaterial-quiz-questions/:id/options  (force flag)
pub async fn replace_options(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
    Json(req): Json<ReplaceOptionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::replace_options(&state.db, admin.0.user_id, id, &req, q.force.unwrap_or(false))
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Options replaced" })))
}

/// DELETE /api/admin/submaterial-quiz-questions/:id  (force flag)
pub async fn delete_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
) -> Result<impl IntoResponse, AppError> {
    service::delete_question(&state.db, admin.0.user_id, id, q.force.unwrap_or(false))
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question deleted" })))
}
