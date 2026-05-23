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
    features::admin::content::module_quiz::{
        dto::{CreateModuleQuizQuestionRequest, ReplaceModuleQuizOptionsRequest, UpdateModuleQuizQuestionRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[derive(Debug, Deserialize, Default)]
pub struct ForceQuery { pub force: Option<bool> }

/// GET /api/admin/modules/:module_id/final-quiz
pub async fn list_questions(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(module_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_questions(&state.db, module_id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

/// POST /api/admin/module-quiz-questions
pub async fn create_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateModuleQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_question(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

/// PATCH /api/admin/module-quiz-questions/:id
pub async fn update_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateModuleQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::update_question(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question updated" })))
}

/// PATCH /api/admin/module-quiz-questions/:id/options
pub async fn replace_options(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
    Json(req): Json<ReplaceModuleQuizOptionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::replace_options(&state.db, admin.0.user_id, id, &req, q.force.unwrap_or(false))
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Options replaced" })))
}

/// DELETE /api/admin/module-quiz-questions/:id
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
