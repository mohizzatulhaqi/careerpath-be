use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SubmaterialModuleInfo {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize)]
pub struct SubmaterialDetailResponse {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub order_index: i32,
    pub estimated_minutes: i32,
    pub is_completed: bool,
    pub module: SubmaterialModuleInfo,
}

#[derive(Debug, Serialize)]
pub struct CompleteModuleStatus {
    pub id: Uuid,
    pub completion_percentage: f64,
    pub is_completed: bool,
    pub next_submaterial_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct NextModuleUnlocked {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct CompleteSubmaterialResponse {
    pub submaterial_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub module: CompleteModuleStatus,
    pub next_module_unlocked: Option<NextModuleUnlocked>,
}
