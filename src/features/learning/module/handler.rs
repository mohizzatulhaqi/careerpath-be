use crate::{
    error::AppError,
    features::learning::module::service,
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/learning/modules",
    tag = "Learning - Modules",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of modules", body = crate::features::learning::module::dto::ModuleListResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
/// GET /api/learning/modules
pub async fn list_modules(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_modules_for_user(&state, auth.user_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    get,
    path = "/api/learning/modules/{id}",
    tag = "Learning - Modules",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Module ID"),
    ),
    responses(
        (status = 200, description = "Module detail", body = crate::features::learning::module::dto::ModuleDetailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// GET /api/learning/modules/:id
pub async fn get_module(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(module_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_module_detail(&state, auth.user_id, module_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}
