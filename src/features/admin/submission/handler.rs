use axum::{
    body::Body,
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
use crate::{error::AppError, middleware::role_guard::AdminUser, state::AppState};

// ── GET /api/admin/submissions ────────────────────────────────────────────────

pub async fn list_submissions(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<SubmissionFilter>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage));
    let result = svc.list(filter).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── GET /api/admin/submissions/:id ────────────────────────────────────────────

pub async fn get_submission(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage));
    let result = svc.get_detail(submission_id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── GET /api/admin/submissions/:id/download ───────────────────────────────────

pub async fn download_submission(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage));
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
        .body(Body::from(payload.bytes))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("response build: {e}")))
}

// ── POST /api/admin/submissions/:id/approve ───────────────────────────────────

pub async fn approve_submission(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage));
    let result = svc
        .approve(admin.0.user_id, submission_id, req.reviewer_notes)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── POST /api/admin/submissions/:id/reject ────────────────────────────────────

pub async fn reject_submission(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage));
    let result = svc
        .reject(admin.0.user_id, submission_id, req.reviewer_notes)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

// ── GET /api/admin/submissions/queue/stats ─────────────────────────────────────

pub async fn queue_stats(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let svc = AdminSubmissionService::new(&state.db, Arc::clone(&state.storage));
    let result = svc.queue_stats().await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}
