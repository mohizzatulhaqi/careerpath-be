use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Submaterial {
    pub id: Uuid,
    pub module_id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub order_index: i32,
    pub estimated_minutes: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
