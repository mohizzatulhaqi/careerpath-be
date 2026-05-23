use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct RoleInfo {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModuleListItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub order_index: i32,
    pub is_unlocked: bool,
    pub is_completed: bool,
    pub completion_percentage: f64,
    pub submaterials_total: i64,
    pub submaterials_completed: i64,
    pub final_quiz_unlocked: bool,
    pub final_quiz_passed: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModuleListResponse {
    pub role: RoleInfo,
    pub modules: Vec<ModuleListItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmaterialInModule {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
    pub estimated_minutes: i32,
    pub is_unlocked: bool,
    pub is_completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ModuleDetailResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub order_index: i32,
    pub is_unlocked: bool,
    pub is_completed: bool,
    pub completion_percentage: f64,
    pub final_quiz_unlocked: bool,
    pub final_quiz_passed: bool,
    pub submaterials: Vec<SubmaterialInModule>,
}
