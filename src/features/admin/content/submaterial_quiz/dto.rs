use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Option DTOs ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdminQuizOptionDto {
    pub id: Uuid,
    pub text: String,
    pub is_correct: bool,
    pub order_index: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct QuizOptionInput {
    #[validate(length(min = 1))]
    pub text: String,
    pub is_correct: bool,
    pub order_index: i32,
}

// ── Question DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdminSubQuizQuestionDto {
    pub id: Uuid,
    pub submaterial_id: Uuid,
    pub question: String,
    pub order_index: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub options: Vec<AdminQuizOptionDto>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubQuizQuestionRequest {
    pub submaterial_id: Uuid,
    #[validate(length(min = 1))]
    pub question_text: String,
    pub order_index: Option<i32>,
    #[validate(length(min = 2))]
    pub options: Vec<QuizOptionInput>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSubQuizQuestionRequest {
    #[validate(length(min = 1))]
    pub question_text: Option<String>,
    pub order_index: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReplaceOptionsRequest {
    #[validate(length(min = 2))]
    pub options: Vec<QuizOptionInput>,
}
