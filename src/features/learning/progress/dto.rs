use utoipa::ToSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::features::learning::module::dto::RoleInfo;

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentModuleInfo {
    pub id: Uuid,
    pub title: String,
    pub completion_percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProgressModuleItem {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
    pub is_unlocked: bool,
    pub is_completed: bool,
    pub completion_percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OverallProgressResponse {
    pub role: RoleInfo,
    pub overall_percentage: f64,
    pub total_modules: i64,
    pub completed_modules: i64,
    pub total_submaterials: i64,
    pub completed_submaterials: i64,
    pub current_module: Option<CurrentModuleInfo>,
    pub modules: Vec<ProgressModuleItem>,
}
