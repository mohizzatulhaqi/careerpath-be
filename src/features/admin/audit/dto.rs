use utoipa::ToSchema;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditAdminDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditLogDto {
    pub id: Uuid,
    pub admin: AuditAdminDto,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationDto {
    pub page: i64,
    pub per_page: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditLogPageDto {
    pub logs: Vec<AuditLogDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct AuditLogFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub action: Option<String>,
    pub admin_id: Option<Uuid>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    /// Inclusive start date, format YYYY-MM-DD (e.g. 2025-01-01)
    pub from_date: Option<NaiveDate>,
    /// Inclusive end date, format YYYY-MM-DD (e.g. 2025-12-31)
    pub to_date: Option<NaiveDate>,
}
