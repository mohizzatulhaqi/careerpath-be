use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Stats snapshot attached to each user in the list ─────────────────────────

#[derive(Debug, Serialize)]
pub struct UserStatsDto {
    pub quiz_attempts: i64,
    pub modules_completed: i64,
    pub projects_submitted: i64,
}

// ── Summary row (list endpoint) ───────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserSummaryDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub deactivation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub stats: UserStatsDto,
}

// ── History items for the detail endpoint ────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ModuleProgressDto {
    pub module_id: Uuid,
    pub module_title: String,
    pub status: String,     // "completed" | "in_progress" | "not_started"
    pub score: Option<f64>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct QuizAttemptDto {
    pub quiz_id: Uuid,
    pub quiz_title: String,
    pub score: f64,
    pub passed: bool,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProjectSubmissionDto {
    pub project_id: Uuid,
    pub project_title: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

// ── Full detail (single user endpoint) ───────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserDetailDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub deactivated_by: Option<Uuid>,
    pub deactivation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub stats: UserStatsDto,
    pub module_progress: Vec<ModuleProgressDto>,
    pub recent_quiz_attempts: Vec<QuizAttemptDto>,
    pub project_submissions: Vec<ProjectSubmissionDto>,
}

// ── Paginated list response ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummaryDto>,
    pub pagination: crate::features::admin::audit::dto::PaginationDto,
}

// ── Query parameters ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
    /// Column to sort by: name | email | created_at | role. Default: created_at.
    pub sort: Option<String>,
    /// "asc" or "desc". Default: "desc".
    pub order: Option<String>,
}

// ── Mutation requests ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1–100 characters"))]
    pub name: Option<String>,

    /// Must be "user" or "admin"
    pub role: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeactivateRequest {
    #[validate(length(min = 5, max = 500, message = "Reason must be 5–500 characters"))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ActivateRequest {
    pub reason: Option<String>,
}
