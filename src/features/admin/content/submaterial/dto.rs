use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::admin::audit::dto::PaginationDto;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminSubmaterialDto {
    pub id: Uuid,
    pub module_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub order_index: i32,
    pub estimated_minutes: i32,
    pub is_published: bool,
    /// True if this submaterial has at least one mini quiz question.
    pub has_mini_quiz: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminSubmaterialListResponse {
    pub submaterials: Vec<AdminSubmaterialDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct AdminSubmaterialFilter {
    pub module_id: Option<Uuid>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateSubmaterialRequest {
    pub module_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub content: Option<String>,
    pub order_index: Option<i32>,
    pub estimated_minutes: Option<i32>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateSubmaterialRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub content: Option<String>,
    pub order_index: Option<i32>,
    pub estimated_minutes: Option<i32>,
    pub is_published: Option<bool>,
}
