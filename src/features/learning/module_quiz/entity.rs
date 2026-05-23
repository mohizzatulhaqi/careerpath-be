use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct ModuleQuizQuestion {
    pub id:          Uuid,
    pub module_id:   Uuid,
    pub question:    String,
    pub order_index: i32,
    pub is_published: bool,
    pub created_at:  DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ModuleQuizOption {
    pub id:          Uuid,
    pub question_id: Uuid,
    pub text:        String,
    pub is_correct:  bool,
    pub order_index: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct ModuleQuizAttempt {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub module_id:  Uuid,
    pub score:      f64,
    pub passed:     bool,
    pub created_at: DateTime<Utc>,
}
