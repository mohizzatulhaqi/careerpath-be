use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Request ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct MiniQuizAnswerInput {
    pub question_id: Uuid,
    pub option_id:   Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitMiniQuizRequest {
    pub answers: Vec<MiniQuizAnswerInput>,
}

// ── GET response (no correct answers revealed) ────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct MiniQuizOptionResponse {
    pub id:          Uuid,
    pub text:        String,
    pub order_index: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MiniQuizQuestionResponse {
    pub id:          Uuid,
    pub question:    String,
    pub order_index: i32,
    pub options:     Vec<MiniQuizOptionResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MiniQuizResponse {
    pub submaterial_id: Uuid,
    pub questions:      Vec<MiniQuizQuestionResponse>,
}

// ── Submit response (correct answers revealed) ────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct MiniQuizResultOption {
    pub id:          Uuid,
    pub text:        String,
    pub order_index: i32,
    pub is_correct:  bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MiniQuizResultQuestion {
    pub id:               Uuid,
    pub question:         String,
    pub order_index:      i32,
    pub your_answer_id:   Uuid,
    pub is_correct:       bool,
    pub correct_option_id: Uuid,
    pub options:          Vec<MiniQuizResultOption>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MiniQuizSubmitResponse {
    pub attempt_id:            Uuid,
    pub score:                 f64,
    pub passed:                bool,
    pub submaterial_completed: bool,
    pub questions:             Vec<MiniQuizResultQuestion>,
}
