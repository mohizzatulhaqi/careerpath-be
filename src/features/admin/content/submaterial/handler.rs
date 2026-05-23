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
    features::admin::content::submaterial::{
        dto::{AdminSubmaterialFilter, CreateSubmaterialRequest, UpdateSubmaterialRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[derive(Debug, Deserialize, Default)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

pub async fn list_submaterials(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<AdminSubmaterialFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_submaterials(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

pub async fn get_submaterial(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_submaterial(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

pub async fn create_submaterial(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSubmaterialRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let (id, requires_mini_quiz) =
        service::create_submaterial(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": { "id": id },
            "requires_mini_quiz": requires_mini_quiz,
            "hint": "Tambahkan mini quiz untuk sub-materi ini agar user dapat menyelesaikannya."
        })),
    ))
}

pub async fn update_submaterial(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSubmaterialRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::update_submaterial(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Submaterial updated" })))
}

pub async fn delete_submaterial(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
) -> Result<impl IntoResponse, AppError> {
    let force = q.force.unwrap_or(false);
    if force {
        service::hard_delete_submaterial(&state.db, admin.0.user_id, id, true)
            .await
            .map_err(AppError::from)?;
        Ok(Json(json!({ "success": true, "message": "Submaterial hard-deleted" })))
    } else {
        service::unpublish_submaterial(&state.db, admin.0.user_id, id)
            .await
            .map_err(AppError::from)?;
        Ok(Json(json!({ "success": true, "message": "Submaterial unpublished" })))
    }
}

pub async fn restore_submaterial(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_submaterial(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Submaterial restored" })))
}
