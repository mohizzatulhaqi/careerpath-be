use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Request ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FinalQuizAnswerInput {
    pub question_id: Uuid,
    pub option_id:   Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SubmitFinalQuizRequest {
    pub answers: Vec<FinalQuizAnswerInput>,
}

// ── GET response (no correct answers revealed) ────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FinalQuizOptionResponse {
    pub id:          Uuid,
    pub text:        String,
    pub order_index: i32,
}

#[derive(Debug, Serialize)]
pub struct FinalQuizQuestionResponse {
    pub id:          Uuid,
    pub question:    String,
    pub order_index: i32,
    pub options:     Vec<FinalQuizOptionResponse>,
}

#[derive(Debug, Serialize)]
pub struct FinalQuizResponse {
    pub module_id: Uuid,
    pub questions: Vec<FinalQuizQuestionResponse>,
}

// ── Submit response (correct answers revealed) ────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FinalQuizResultOption {
    pub id:          Uuid,
    pub text:        String,
    pub order_index: i32,
    pub is_correct:  bool,
}

#[derive(Debug, Serialize)]
pub struct FinalQuizResultQuestion {
    pub id:                Uuid,
    pub question:          String,
    pub order_index:       i32,
    pub your_answer_id:    Uuid,
    pub is_correct:        bool,
    pub correct_option_id: Uuid,
    pub options:           Vec<FinalQuizResultOption>,
}

#[derive(Debug, Serialize)]
pub struct FinalQuizSubmitResponse {
    pub attempt_id: Uuid,
    pub score:      f64,
    pub passed:     bool,
    pub questions:  Vec<FinalQuizResultQuestion>,
}

// ── History response ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FinalQuizAttemptItem {
    pub attempt_id:  Uuid,
    pub score:       f64,
    pub passed:      bool,
    pub created_at:  DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FinalQuizHistoryResponse {
    pub module_id:  Uuid,
    pub best_score: Option<f64>,
    pub attempts:   Vec<FinalQuizAttemptItem>,
}
