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
