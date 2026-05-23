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
    features::admin::{
        audit::dto::AuditLogFilter,
        user::{
            dto::{ActivateRequest, DeactivateRequest, UpdateUserRequest, UserListQuery},
            service,
        },
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

// ── GET /api/admin/users ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "Admin - Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of users", body = crate::features::admin::user::dto::UserListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_users(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<UserListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_users(&state.db, &q)
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "success": true,
        "data": result
    })))
}

// ── GET /api/admin/users/:id ──────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    tag = "Admin - Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User detail", body = crate::features::admin::user::dto::UserDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_user(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_user_detail(&state.db, user_id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "success": true,
        "data": result
    })))
}

// ── PATCH /api/admin/users/:id ────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/admin/users/{id}",
    tag = "Admin - Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID"),
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_user(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    service::update_user(&state.db, admin.0.user_id, user_id, &req)
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "success": true,
        "message": "User updated successfully"
    })))
}

// ── POST /api/admin/users/:id/deactivate ─────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/deactivate",
    tag = "Admin - Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID"),
    ),
    request_body = DeactivateRequest,
    responses(
        (status = 200, description = "User deactivated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn deactivate_user(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<DeactivateRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    service::deactivate_user(&state.db, admin.0.user_id, user_id, &req)
        .await
        .map_err(AppError::from)?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "User deactivated"
        })),
    ))
}

// ── POST /api/admin/users/:id/activate ───────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/activate",
    tag = "Admin - Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID"),
    ),
    request_body = ActivateRequest,
    responses(
        (status = 200, description = "User activated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn activate_user(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<ActivateRequest>,
) -> Result<impl IntoResponse, AppError> {
    service::activate_user(&state.db, admin.0.user_id, user_id, &req)
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "success": true,
        "message": "User activated"
    })))
}

// ── GET /api/admin/users/:id/audit-logs ──────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/users/{id}/audit-logs",
    tag = "Admin - Audit",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User audit logs", body = crate::features::admin::audit::dto::AuditLogPageDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_user_audit_logs(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<crate::features::admin::audit::dto::AuditLogFilter>,
) -> Result<impl IntoResponse, AppError> {
    let page = q.page.unwrap_or(1);
    let per_page = q.per_page.unwrap_or(20);

    let result = service::get_user_audit_logs(&state.db, user_id, page, per_page)
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "success": true,
        "data": result
    })))
}

// ── GET /api/admin/audit-logs ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/audit-logs",
    tag = "Admin - Audit",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All audit logs", body = crate::features::admin::audit::dto::AuditLogPageDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_audit_logs(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<AuditLogFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_audit_logs(&state.db, &filter)
        .await
        .map_err(AppError::from)?;

    Ok(Json(json!({
        "success": true,
        "data": result
    })))
}
