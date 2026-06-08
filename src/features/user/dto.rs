use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ── Profile Summary ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileCareerRoleDto {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileStatsDto {
    pub modules_completed: i64,
    pub modules_total: i64,
    pub projects_approved: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileCertificateDto {
    pub id: Uuid,
    pub certificate_code: String,
    pub role_name: String,
    pub issued_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub verification_url: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileActivityDto {
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileSummaryResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub member_since: DateTime<Utc>,
    /// Role karier dari hasil prequiz (null jika belum prequiz).
    pub career_role: Option<ProfileCareerRoleDto>,
    pub stats: ProfileStatsDto,
    pub certificates: Vec<ProfileCertificateDto>,
    pub recent_activities: Vec<ProfileActivityDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    /// Nama baru (opsional, 1–100 karakter)
    #[validate(length(min = 1, max = 100, message = "name must be 1–100 characters"))]
    pub name: Option<String>,

    /// Email baru (opsional)
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}
