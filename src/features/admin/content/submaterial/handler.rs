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

#[utoipa::path(
    get,
    path = "/api/admin/submaterials",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of submaterials", body = crate::features::admin::content::submaterial::dto::AdminSubmaterialListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_submaterials(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<AdminSubmaterialFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_submaterials(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    get,
    path = "/api/admin/submaterials/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    responses(
        (status = 200, description = "Submaterial detail", body = crate::features::admin::content::submaterial::dto::AdminSubmaterialDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_submaterial(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_submaterial(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    post,
    path = "/api/admin/submaterials",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    request_body = CreateSubmaterialRequest,
    responses(
        (status = 201, description = "Submaterial created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
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

#[utoipa::path(
    patch,
    path = "/api/admin/submaterials/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    request_body = UpdateSubmaterialRequest,
    responses(
        (status = 200, description = "Submaterial updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    delete,
    path = "/api/admin/submaterials/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    responses(
        (status = 200, description = "Submaterial deleted or unpublished"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/admin/submaterials/{id}/restore",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    responses(
        (status = 200, description = "Submaterial restored"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn restore_submaterial(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_submaterial(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Submaterial restored" })))
}
