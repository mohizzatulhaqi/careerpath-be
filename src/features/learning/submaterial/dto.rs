use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmaterialModuleInfo {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FinalQuizStatusDto {
    /// Apakah ini submaterial terakhir dalam modul.
    pub is_last_submaterial: bool,
    /// Apakah semua submaterials dalam modul sudah selesai.
    pub all_subs_completed: bool,
    /// Apakah modul punya soal final quiz.
    pub has_questions: bool,
    /// Apakah user sudah lulus final quiz (score >= 70).
    pub is_passed: bool,
    /// Skor terbaik user. None jika belum pernah attempt.
    pub best_score: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubmaterialDetailResponse {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub order_index: i32,
    pub estimated_minutes: i32,
    pub is_completed: bool,
    pub module: SubmaterialModuleInfo,
    pub final_quiz: FinalQuizStatusDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompleteModuleStatus {
    pub id: Uuid,
    pub completion_percentage: f64,
    pub is_completed: bool,
    pub next_submaterial_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NextModuleUnlocked {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CompleteSubmaterialResponse {
    pub submaterial_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub module: CompleteModuleStatus,
    pub next_module_unlocked: Option<NextModuleUnlocked>,
}
