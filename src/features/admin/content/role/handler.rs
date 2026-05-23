use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    features::admin::content::role::{
        dto::{CreateRoleRequest, RoleListQuery, UpdateRoleRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/admin/roles",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of roles", body = crate::features::admin::content::role::dto::RoleListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_roles(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<RoleListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_roles(&state.db, &q).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    get,
    path = "/api/admin/roles/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role detail", body = crate::features::admin::content::role::dto::RoleDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_role(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_role(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    post,
    path = "/api/admin/roles",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_role(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_role(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

#[utoipa::path(
    patch,
    path = "/api/admin/roles/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Role ID"),
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_role(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::update_role(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Role updated" })))
}

#[utoipa::path(
    post,
    path = "/api/admin/roles/{id}/deactivate",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role deactivated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn deactivate_role(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::deactivate_role(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Role deactivated" })))
}

#[utoipa::path(
    post,
    path = "/api/admin/roles/{id}/restore",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role restored"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn restore_role(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_role(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Role restored" })))
}
