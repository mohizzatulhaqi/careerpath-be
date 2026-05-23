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
    features::admin::content::pre_quiz::{
        dto::{CreatePreQuizQuestionRequest, PreQuizFilter, ReplacePreQuizOptionsRequest, UpdatePreQuizQuestionRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[derive(Debug, Deserialize, Default)]
pub struct ForceQuery { pub force: Option<bool> }

pub async fn list_questions(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<PreQuizFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_questions(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

pub async fn get_question(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_question(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

pub async fn create_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePreQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_question(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

pub async fn update_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePreQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::update_question(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question updated" })))
}

pub async fn replace_options(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
    Json(req): Json<ReplacePreQuizOptionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::replace_options(&state.db, admin.0.user_id, id, &req, q.force.unwrap_or(false))
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Options replaced" })))
}

pub async fn deactivate_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::deactivate_question(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question deactivated" })))
}

pub async fn restore_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_question(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question restored" })))
}
