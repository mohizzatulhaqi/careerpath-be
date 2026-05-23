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
    features::admin::content::project::{
        dto::{AdminProjectFilter, CreateProjectRequest, UpdateProjectRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/admin/projects",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of projects", body = crate::features::admin::content::project::dto::AdminProjectListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_projects(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<AdminProjectFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_projects(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    get,
    path = "/api/admin/projects/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project detail", body = crate::features::admin::content::project::dto::AdminProjectDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_project(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_project(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    post,
    path = "/api/admin/projects",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_project(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_project(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

#[utoipa::path(
    patch,
    path = "/api/admin/projects/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_project(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::update_project(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Project updated" })))
}

#[utoipa::path(
    post,
    path = "/api/admin/projects/{id}/unpublish",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project unpublished"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn unpublish_project(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::unpublish_project(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Project unpublished" })))
}

#[utoipa::path(
    post,
    path = "/api/admin/projects/{id}/restore",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project restored"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn restore_project(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_project(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Project restored" })))
}
