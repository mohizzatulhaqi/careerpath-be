use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Database entity for the `projects` table.
#[derive(Debug, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub role_id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: Option<String>,
    pub estimated_hours: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database entity for the `project_submissions` table.
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ProjectSubmission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub submission_notes: String,
    pub file_path: String,
    pub file_original_name: String,
    pub file_size_bytes: i64,
    pub file_mime_type: String,
    pub status: String,
    pub reviewer_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub submitted_at: DateTime<Utc>,
}
