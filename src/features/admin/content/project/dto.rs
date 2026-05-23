use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::admin::audit::dto::PaginationDto;

#[derive(Debug, Serialize)]
pub struct AdminProjectDto {
    pub id: Uuid,
    pub role_id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: Option<String>,
    pub estimated_hours: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub submission_count: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminProjectListResponse {
    pub projects: Vec<AdminProjectDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Deserialize, Default)]
pub struct AdminProjectFilter {
    pub role_id: Option<Uuid>,
    pub is_published: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    pub role_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub description: String,
    pub requirements: Option<String>,
    pub estimated_hours: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1))]
    pub description: Option<String>,
    pub requirements: Option<String>,
    pub estimated_hours: Option<i32>,
    pub is_published: Option<bool>,
}
