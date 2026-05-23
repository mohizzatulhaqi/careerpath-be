use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Requests ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SubmitAnswerRequest {
    pub question_id: Uuid,
    pub option_id: Uuid,
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct OptionDto {
    pub id: Uuid,
    pub option_text: String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuestionDto {
    pub id: Uuid,
    pub question_text: String,
    pub order_index: i32,
    pub options: Vec<OptionDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttemptCreatedResponse {
    pub attempt_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnswerSavedResponse {
    pub saved: bool,
    pub question_id: Uuid,
    pub option_id: Uuid,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct RoleDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopReason {
    pub question_text: String,
    pub option_text: String,
    pub contributed_weight: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleScoreDto {
    pub role_code: String,
    pub role_name: String,
    pub score: i32,
    pub max_possible: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QuizResultResponse {
    pub attempt_id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub role: RoleDto,
    pub match_percentage: f64,
    pub top_reasons: Vec<TopReason>,
    pub all_scores: Vec<RoleScoreDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryItem {
    pub attempt_id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub role_name: String,
    pub role_code: String,
    pub match_percentage: f64,
}
