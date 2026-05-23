use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct QuizQuestion {
    pub id: Uuid,
    pub question_text: String,
    pub order_index: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct QuizOption {
    pub id: Uuid,
    pub question_id: Uuid,
    pub option_text: String,
    pub order_index: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct QuizAttempt {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub result_role_id: Option<Uuid>,
    pub match_percentage: Option<f64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct QuizAttemptAnswer {
    pub id: Uuid,
    pub attempt_id: Uuid,
    pub question_id: Uuid,
    pub option_id: Uuid,
    pub answered_at: DateTime<Utc>,
}
