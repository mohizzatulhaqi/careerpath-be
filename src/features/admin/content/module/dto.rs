use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::admin::audit::dto::PaginationDto;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminModuleDto {
    pub id: Uuid,
    pub role_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub order_index: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub stats: AdminModuleStatsDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminModuleStatsDto {
    pub total_submaterials: i64,
    pub total_users_started: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminModuleListResponse {
    pub modules: Vec<AdminModuleDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct AdminModuleFilter {
    pub role_id: Option<Uuid>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateModuleRequest {
    pub role_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    /// If omitted, auto-computed as max(order_index)+1 for the role.
    pub order_index: Option<i32>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateModuleRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub order_index: Option<i32>,
    pub is_published: Option<bool>,
}
