use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserSubmaterialProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub submaterial_id: Uuid,
    pub completed_at: DateTime<Utc>,
}
