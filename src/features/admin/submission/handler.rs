use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use super::{dto::ReviewRequest, dto::SubmissionFilter, service::AdminSubmissionService};
use crate::{error::AppError, middleware::role_guard::AdminUser, shared::body::chunked_body, state::AppState};

// ── GET /api/admin/submissions ────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/submissions",
    tag = "Admin - Submissions",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of submissions", body = crate::features::admin::submission::dto::SubmissionListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_submissions(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<SubmissionFilter>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage), state.config.resend_api_key.clone());
    let result = svc.list(filter).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── GET /api/admin/submissions/:id ────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/submissions/{id}",
    tag = "Admin - Submissions",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submission ID"),
    ),
    responses(
        (status = 200, description = "Submission detail", body = crate::features::admin::submission::dto::SubmissionDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_submission(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage), state.config.resend_api_key.clone());
    let result = svc.get_detail(submission_id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── GET /api/admin/submissions/:id/download ───────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/submissions/{id}/download",
    tag = "Admin - Submissions",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submission ID"),
    ),
    responses(
        (status = 200, description = "Download submission file"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn download_submission(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage), state.config.resend_api_key.clone());
    let payload = svc
        .download_file(admin.0.user_id, submission_id)
        .await
        .map_err(AppError::from)?;

    // Safe ASCII fallback filename (strip non-ASCII + dangerous chars)
    let ascii_name: String = payload
        .original_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();

    // RFC 5987 UTF-8 encoded filename for clients that support it
    let encoded_name =
        utf8_percent_encode(&payload.original_name, NON_ALPHANUMERIC).to_string();

    let content_disposition = format!(
        "attachment; filename=\"{ascii_name}\"; filename*=UTF-8''{encoded_name}"
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &payload.mime_type)
        .header(header::CONTENT_LENGTH, payload.size.to_string())
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(chunked_body(payload.bytes))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("response build: {e}")))
}

// ── POST /api/admin/submissions/:id/approve ───────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/submissions/{id}/approve",
    tag = "Admin - Submissions",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submission ID"),
    ),
    request_body = ReviewRequest,
    responses(
        (status = 200, description = "Submission approved", body = crate::features::admin::submission::dto::ReviewResultDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn approve_submission(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage), state.config.resend_api_key.clone());
    let result = svc
        .approve(admin.0.user_id, submission_id, req.reviewer_notes)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── POST /api/admin/submissions/:id/reject ────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/submissions/{id}/reject",
    tag = "Admin - Submissions",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Submission ID"),
    ),
    request_body = ReviewRequest,
    responses(
        (status = 200, description = "Submission rejected", body = crate::features::admin::submission::dto::ReviewResultDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn reject_submission(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage), state.config.resend_api_key.clone());
    let result = svc
        .reject(admin.0.user_id, submission_id, req.reviewer_notes)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── GET /api/admin/submissions/queue/stats ─────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/submissions/queue/stats",
    tag = "Admin - Submissions",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Queue statistics", body = crate::features::admin::submission::dto::QueueStatsDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn queue_stats(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage), state.config.resend_api_key.clone());
    let result = svc.queue_stats().await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}
