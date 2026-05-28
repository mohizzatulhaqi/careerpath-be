use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use super::{dto::{AdminCertificateFilter, RevokeRequest}, service::CertificateService};
use crate::{error::AppError, middleware::{auth::AuthUser, role_guard::AdminUser}, state::AppState};

// ── GET /api/certificates/me ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/certificates/me",
    tag = "Certificates",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "My certificates", body = crate::features::certificate::dto::CertificateListResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_my_certificates(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let data = svc.list_for_user(auth.user_id).await?;
    Ok(Json(json!({ "success": true, "data": data })))
}

// ── GET /api/certificates/me/:id ─────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/certificates/me/{id}",
    tag = "Certificates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Certificate ID")),
    responses(
        (status = 200, description = "Certificate detail", body = crate::features::certificate::dto::CertificateDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_my_certificate(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(cert_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let data = svc.get_for_user(auth.user_id, cert_id).await?;
    Ok(Json(json!({ "success": true, "data": data })))
}

// ── GET /api/certificates/me/:id/download.pdf ────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/certificates/me/{id}/download.pdf",
    tag = "Certificates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Certificate ID")),
    responses(
        (status = 200, description = "Certificate PDF"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn download_certificate_pdf(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(cert_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let (pdf_bytes, code) = svc.generate_pdf_bytes(auth.user_id, cert_id).await?;

    let filename = format!("certificate-{}.pdf", code);
    let disposition = format!("attachment; filename=\"{}\"", filename);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CONTENT_LENGTH, pdf_bytes.len())
        .body(Body::from(pdf_bytes))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{e}")))?;

    Ok(response)
}

// ── GET /api/verify/:code ────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/verify/{code}",
    tag = "Certificates",
    params(("code" = String, Path, description = "Certificate code")),
    responses(
        (status = 200, description = "Verification result", body = crate::features::certificate::dto::VerificationResponse),
    )
)]
pub async fn verify_certificate(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let data = svc.verify_by_code(&code).await?;
    Ok(Json(json!({ "success": true, "data": data })))
}

// ── GET /api/admin/certificates ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/admin/certificates",
    tag = "Admin - Certificates",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Certificate list", body = crate::features::certificate::dto::AdminCertificateListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn admin_list_certificates(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<AdminCertificateFilter>,
) -> Result<impl IntoResponse, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let data = svc.admin_list(filter).await?;
    Ok(Json(json!({ "success": true, "data": data })))
}

// ── POST /api/admin/certificates/:id/revoke ──────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/certificates/{id}/revoke",
    tag = "Admin - Certificates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Certificate ID")),
    request_body = crate::features::certificate::dto::RevokeRequest,
    responses(
        (status = 200, description = "Certificate revoked", body = crate::features::certificate::dto::AdminCertificateSummaryDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Already revoked"),
    )
)]
pub async fn admin_revoke_certificate(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(cert_id): Path<Uuid>,
    Json(req): Json<RevokeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let data = svc.revoke(admin.0.user_id, cert_id, req).await?;
    Ok(Json(json!({ "success": true, "data": data })))
}

// ── POST /api/admin/certificates/:id/restore ─────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/admin/certificates/{id}/restore",
    tag = "Admin - Certificates",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Certificate ID")),
    responses(
        (status = 200, description = "Certificate restored", body = crate::features::certificate::dto::AdminCertificateSummaryDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Not revoked"),
    )
)]
pub async fn admin_restore_certificate(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(cert_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let svc = CertificateService::new(&state.db, Arc::clone(&state.config));
    let data = svc.restore(admin.0.user_id, cert_id).await?;
    Ok(Json(json!({ "success": true, "data": data })))
}
