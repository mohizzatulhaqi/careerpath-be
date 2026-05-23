use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct LearningModule {
    pub id: Uuid,
    pub role_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub order_index: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
