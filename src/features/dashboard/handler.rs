use crate::{
    error::AppError,
    features::{
        dashboard::service,
        learning::{error::LearningError, progress::service as progress_service},
    },
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/api/dashboard",
    tag = "Dashboard",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Full dashboard data", body = crate::features::dashboard::dto::DashboardResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
/// GET /api/dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_full_dashboard(&state.db, auth.user_id).await?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    get,
    path = "/api/dashboard/learning-summary",
    tag = "Dashboard",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Learning summary", body = crate::features::dashboard::dto::LearningSummaryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Quiz not completed"),
    )
)]
/// GET /api/dashboard/learning-summary
pub async fn get_learning_summary(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let (role_id, role_info) = progress_service::resolve_user_role(&state, auth.user_id)
        .await
        .map_err(|e| match e {
            LearningError::QuizNotCompleted => {
                AppError::Forbidden("Kamu belum menyelesaikan kuis role".into())
            }
            LearningError::Database(db_err) => AppError::Internal(db_err.into()),
            other => AppError::Internal(anyhow::anyhow!("{other}")),
        })?;

    let data = service::get_learning_summary(
        &state.db,
        auth.user_id,
        role_id,
        role_info.code,
        role_info.name,
        None, // description not in RoleInfo; pass None
    )
    .await?;

    Ok(ApiResponse::ok(data))
}

#[derive(Deserialize)]
pub struct ActivityQuery {
    pub limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/dashboard/activity",
    tag = "Dashboard",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Activity log", body = crate::features::dashboard::dto::ActivityLogResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
/// GET /api/dashboard/activity?limit=20
pub async fn get_activity_log(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<ActivityQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let data = service::get_activity_log(&state.db, auth.user_id, limit).await?;
    Ok(ApiResponse::ok(data))
}
