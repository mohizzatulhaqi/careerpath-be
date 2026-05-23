use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct RoleDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RoleDetailDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub stats: RoleStatsDto,
}

#[derive(Debug, Serialize)]
pub struct RoleStatsDto {
    pub total_modules: i64,
    pub has_project: bool,
    pub total_users_with_role: i64,
}

#[derive(Debug, Serialize)]
pub struct RoleListResponse {
    pub roles: Vec<RoleDto>,
    pub total: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct RoleListQuery {
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    /// Lowercase alphanumeric + underscore, e.g. "frontend_dev". Validated in service.
    #[validate(length(min = 1, max = 50))]
    pub code: String,

    #[validate(length(min = 1, max = 100))]
    pub name: String,

    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub description: Option<String>,
}

// Role code validation is done in the service layer (is_valid_role_code helper).
