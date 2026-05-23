use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AuditAdminDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub admin: AuditAdminDto,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginationDto {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize)]
pub struct AuditLogPageDto {
    pub logs: Vec<AuditLogDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuditLogFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub action: Option<String>,
    pub admin_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}
