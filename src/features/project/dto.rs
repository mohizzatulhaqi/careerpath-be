use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// ── Response DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModuleProgressDto {
    pub total: i64,
    pub completed: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LatestSubmissionDto {
    pub id: Uuid,
    pub submission_notes: String,
    pub file_original_name: String,
    pub file_size_bytes: i64,
    pub status: String,
    pub reviewer_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectDetailDto {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: Option<String>,
    pub estimated_hours: i32,
    pub module_progress: ModuleProgressDto,
    pub access_status: String,
    pub latest_submission: Option<LatestSubmissionDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MyProjectResponse {
    pub role: RoleDto,
    pub project: ProjectDetailDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmitResponse {
    pub submission_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub file_original_name: String,
    pub file_size_bytes: i64,
    pub submission_notes: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmissionHistoryItem {
    pub id: Uuid,
    pub file_original_name: String,
    pub file_size_bytes: i64,
    pub submission_notes: String,
    pub status: String,
    pub reviewer_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub submitted_at: DateTime<Utc>,
}

// ── Input DTOs (parsed from multipart) ─────────────────────────────────

/// Parsed from multipart/form-data
#[derive(Debug)]
pub struct SubmitProjectInput {
    pub file_name: String,
    pub file_bytes: bytes::Bytes,
    pub submission_notes: String,
}
