use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Nested DTOs ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct UserBriefDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleBriefDto {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectBriefDto {
    pub id: Uuid,
    pub title: String,
    pub role: RoleBriefDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FileBriefDto {
    pub original_name: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewerBriefDto {
    pub id: Uuid,
    pub name: String,
}

// ── List response ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmissionSummaryDto {
    pub id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub user: UserBriefDto,
    pub project: ProjectBriefDto,
    pub file: FileBriefDto,
    /// First 200 chars of (already-sanitized) submission_notes.
    pub submission_notes_preview: String,
    pub reviewer: Option<ReviewerBriefDto>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub previous_attempts_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QueueSummaryDto {
    pub pending_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmissionListResponse {
    pub submissions: Vec<SubmissionSummaryDto>,
    pub pagination: crate::features::admin::audit::dto::PaginationDto,
    pub summary: QueueSummaryDto,
}

// ── Filter ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct SubmissionFilter {
    /// "pending_review" | "approved" | "rejected" | "all" (default: "pending_review")
    pub status: Option<String>,
    pub project_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    /// "submitted_at_asc" (default, FIFO) | "submitted_at_desc"
    pub sort: Option<String>,
}

// ── Detail response ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDetailForReviewDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: Option<RoleBriefDto>,
    pub modules_completed: i64,
    pub modules_total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectDetailForReviewDto {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: Option<String>,
    pub estimated_hours: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentReviewDto {
    pub reviewer: Option<ReviewerBriefDto>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewer_notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewHistoryEntryDto {
    pub id: Uuid,
    pub old_status: Option<String>,
    pub new_status: String,
    pub reviewer: ReviewerBriefDto,
    pub reviewer_notes: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmissionHistoryItemDto {
    pub id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmissionDetailDto {
    pub id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub user: UserDetailForReviewDto,
    pub project: ProjectDetailForReviewDto,
    pub file: FileBriefDto,
    pub submission_notes: String,
    pub current_review: CurrentReviewDto,
    pub review_history: Vec<ReviewHistoryEntryDto>,
    pub submission_history: Vec<SubmissionHistoryItemDto>,
}

// ── Review result (approve/reject response) ───────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewResultDto {
    pub submission_id: Uuid,
    pub status: String,
    pub reviewed_at: DateTime<Utc>,
    pub reviewer_notes: String,
    pub user: UserBriefDto,
    pub project: ProjectBriefDto,
    pub certificate_issued: bool,
    pub certificate_id: Option<Uuid>,
    pub certificate_code: Option<String>,
}

// ── Queue stats ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct RolePendingCountDto {
    pub role: RoleBriefDto,
    pub pending_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QueueStatsDto {
    pub pending_count: i64,
    pub oldest_pending_submitted_at: Option<DateTime<Utc>>,
    pub approved_today: i64,
    pub rejected_today: i64,
    pub approved_this_week: i64,
    pub rejected_this_week: i64,
    pub by_role: Vec<RolePendingCountDto>,
}

// ── Request bodies ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReviewRequest {
    pub reviewer_notes: String,
}
