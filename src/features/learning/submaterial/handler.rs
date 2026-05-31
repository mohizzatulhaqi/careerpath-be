use crate::{
    error::AppError,
    features::learning::submaterial::service,
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
    operation_id = "user_get_submaterial",
    path = "/api/learning/submaterials/{id}",
    tag = "Learning - Submaterials",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    responses(
        (status = 200, description = "Submaterial detail", body = crate::features::learning::submaterial::dto::SubmaterialDetailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// GET /api/learning/submaterials/:id
pub async fn get_submaterial(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(submaterial_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_submaterial(&state, auth.user_id, submaterial_id)
        .await
        .map_err(AppError::from)?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    post,
    operation_id = "user_complete_submaterial",
    path = "/api/learning/submaterials/{id}/complete",
    tag = "Learning - Submaterials",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submaterial ID"),
    ),
    responses(
        (status = 410, description = "Endpoint deprecated - use quiz submit instead"),
    )
)]
/// POST /api/learning/submaterials/:id/complete — DEPRECATED (410 Gone)
/// Use POST /api/learning/submaterials/:id/quiz/submit instead.
pub async fn complete_submaterial(
    _state: State<Arc<AppState>>,
    _auth: AuthUser,
    _path: Path<Uuid>,
) -> impl IntoResponse {
    AppError::Gone(
        "Endpoint ini sudah dihapus — gunakan POST /api/learning/submaterials/:id/quiz/submit untuk menyelesaikan sub materi".to_string(),
    )
}
