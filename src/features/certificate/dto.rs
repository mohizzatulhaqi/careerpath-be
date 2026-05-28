use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::features::admin::audit::dto::PaginationDto;

// ── User-facing summary (list endpoint) ──────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CertificateSummaryDto {
    pub id: Uuid,
    pub certificate_code: String,
    pub recipient_name: String,
    pub role_name: String,
    pub role_code: String,
    pub issued_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub preview_url: String,
    pub download_url: String,
    pub verification_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CertificateListResponse {
    pub certificates: Vec<CertificateSummaryDto>,
}

// ── Achievement details (for detail endpoint) ─────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CompletedModuleDto {
    pub id: Uuid,
    pub title: String,
    pub completion_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FinalProjectAchievementDto {
    pub title: String,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AchievementDto {
    pub modules_completed_count: i32,
    pub modules: Vec<CompletedModuleDto>,
    pub final_project: FinalProjectAchievementDto,
}

// ── User-facing detail ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CertificateDetailDto {
    pub id: Uuid,
    pub certificate_code: String,
    pub recipient_name: String,
    pub role_name: String,
    pub role_code: String,
    pub issued_at: DateTime<Utc>,
    pub achievement: AchievementDto,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
    pub verification_url: String,
    pub qr_code_data_url: String,
}

// ── Public verification ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicCertificateDto {
    pub certificate_code: String,
    pub recipient_name: String,
    pub role_name: String,
    pub issued_at: DateTime<Utc>,
    pub modules_completed_count: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VerificationResponse {
    pub is_valid: bool,
    pub certificate: Option<PublicCertificateDto>,
    /// "valid" | "revoked" | "not_found"
    pub status: String,
    pub message: String,
}

// ── Admin list ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminCertificateSummaryDto {
    pub id: Uuid,
    pub certificate_code: String,
    pub recipient_name: String,
    pub user_id: Uuid,
    pub user_email: String,
    pub role_name: String,
    pub role_code: String,
    pub issued_at: DateTime<Utc>,
    pub modules_completed_count: i32,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminCertificateListResponse {
    pub certificates: Vec<AdminCertificateSummaryDto>,
    pub pagination: PaginationDto,
}

// ── Admin filter / query ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct AdminCertificateFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub is_revoked: Option<bool>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

// ── Admin revoke request ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RevokeRequest {
    #[validate(length(min = 20, max = 2000, message = "reason must be 20–2000 chars"))]
    pub reason: String,
}

// ── Internal: data needed for try_issue ───────────────────────────────────────

/// Returned by try_issue when a certificate is newly created.
#[derive(Debug, Clone)]
pub struct IssuedCertificate {
    pub id: Uuid,
    pub certificate_code: String,
}
