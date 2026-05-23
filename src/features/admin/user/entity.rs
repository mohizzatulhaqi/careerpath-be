use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Lightweight row returned by the paginated list query (includes aggregate stats).
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
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
    // Aggregate stats (computed in the list query to avoid N+1)
    pub quiz_attempts: Option<i64>,
    pub modules_completed: Option<i64>,
    pub projects_submitted: Option<i64>,
}

/// Full user row without stats, used for detail / mutation queries.
#[derive(Debug, sqlx::FromRow)]
pub struct UserDetailRow {
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
}

/// Per-module progress row used in user detail.
#[derive(Debug, sqlx::FromRow)]
pub struct UserModuleProgress {
    pub module_id: Uuid,
    pub module_title: String,
    pub status: String,
    pub score: Option<f64>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Quiz attempt row used in user detail.
#[derive(Debug, sqlx::FromRow)]
pub struct UserQuizAttempt {
    pub quiz_id: Uuid,
    pub quiz_title: String,
    pub score: f64,
    pub passed: bool,
    pub attempted_at: DateTime<Utc>,
}

/// Project submission row used in user detail.
#[derive(Debug, sqlx::FromRow)]
pub struct UserProjectSubmission {
    pub project_id: Uuid,
    pub project_title: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}
