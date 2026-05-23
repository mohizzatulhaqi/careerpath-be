use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub admin_id: Uuid,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

/// Joined version with admin user info.
#[derive(Debug, sqlx::FromRow)]
pub struct AuditLogWithAdmin {
    pub id: Uuid,
    pub admin_id: Uuid,
    pub admin_name: String,
    pub admin_email: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}
