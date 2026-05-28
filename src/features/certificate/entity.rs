use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Certificate {
    pub id: Uuid,
    pub certificate_code: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub recipient_name: String,
    pub role_name: String,
    pub role_code: String,
    pub issued_at: DateTime<Utc>,
    pub modules_completed_count: i32,
    pub final_project_submission_id: Uuid,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub revocation_reason: Option<String>,
}
