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

#[utoipa::path(
    get,
    path = "/api/admin/modules",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of modules", body = crate::features::admin::content::module::dto::AdminModuleListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_modules(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<AdminModuleFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_modules(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    get,
    path = "/api/admin/modules/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Module detail", body = crate::features::admin::content::module::dto::AdminModuleDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_module(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_module(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    post,
    path = "/api/admin/modules",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    request_body = CreateModuleRequest,
    responses(
        (status = 201, description = "Module created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_module(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateModuleRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_module(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

#[utoipa::path(
    patch,
    path = "/api/admin/modules/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    request_body = UpdateModuleRequest,
    responses(
        (status = 200, description = "Module updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    delete,
    path = "/api/admin/modules/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Module deleted or unpublished"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/admin/modules/{id}/restore",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Module restored"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn restore_module(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_module(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Module restored" })))
}
