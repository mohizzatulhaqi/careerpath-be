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
    features::admin::content::module::{
        dto::{AdminModuleFilter, CreateModuleRequest, UpdateModuleRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[derive(Debug, Deserialize, Default)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

pub async fn list_modules(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<AdminModuleFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_modules(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

pub async fn get_module(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_module(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

pub async fn create_module(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateModuleRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_module(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

pub async fn update_module(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateModuleRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::update_module(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Module updated" })))
}

pub async fn delete_module(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
) -> Result<impl IntoResponse, AppError> {
    let force = q.force.unwrap_or(false);
    if force {
        service::hard_delete_module(&state.db, admin.0.user_id, id, true)
            .await
            .map_err(AppError::from)?;
        Ok(Json(json!({ "success": true, "message": "Module hard-deleted" })))
    } else {
        let affected = service::unpublish_module(&state.db, admin.0.user_id, id)
            .await
            .map_err(AppError::from)?;
        Ok(Json(json!({
            "success": true,
            "message": "Module unpublished (soft delete)",
            "affected_users": affected
        })))
    }
}

pub async fn restore_module(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_module(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Module restored" })))
}
