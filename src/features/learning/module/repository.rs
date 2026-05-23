use crate::features::learning::module::entity::LearningModule;
use sqlx::PgPool;
use uuid::Uuid;

/// Row returned by the aggregate query for modules with progress counts.
#[derive(Debug, sqlx::FromRow)]
pub struct ModuleWithCountsRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub order_index: i32,
    pub submaterials_total: i64,
    pub submaterials_completed: i64,
}

/// Fetch all published modules for a role with submaterial counts and user progress
/// in a single query (no N+1).
pub async fn get_modules_with_counts(
    pool: &PgPool,
    role_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<ModuleWithCountsRow>, sqlx::Error> {
    sqlx::query_as::<_, ModuleWithCountsRow>(
        r#"
        SELECT m.id,
               m.title,
               m.description,
               m.order_index,
               COUNT(s.id)                                        AS submaterials_total,
               COUNT(usp.id)                                      AS submaterials_completed
        FROM   learning_modules m
        LEFT JOIN submaterials s ON s.module_id = m.id
        LEFT JOIN user_submaterial_progress usp
               ON usp.submaterial_id = s.id AND usp.user_id = $2
        WHERE  m.role_id = $1 AND m.is_published = true
        GROUP  BY m.id, m.title, m.description, m.order_index
        ORDER  BY m.order_index
        "#,
    )
    .bind(role_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Fetch a single module by ID.
pub async fn find_by_id(
    pool: &PgPool,
    module_id: Uuid,
) -> Result<Option<LearningModule>, sqlx::Error> {
    sqlx::query_as::<_, LearningModule>(
        r#"
        SELECT id, role_id, title, description, order_index, is_published, created_at, updated_at
        FROM   learning_modules
        WHERE  id = $1
        "#,
    )
    .bind(module_id)
    .fetch_optional(pool)
    .await
}

/// Fetch all published modules for a role, ordered by order_index.
pub async fn get_modules_for_role(
    pool: &PgPool,
    role_id: Uuid,
) -> Result<Vec<LearningModule>, sqlx::Error> {
    sqlx::query_as::<_, LearningModule>(
        r#"
        SELECT id, role_id, title, description, order_index, is_published, created_at, updated_at
        FROM   learning_modules
        WHERE  role_id = $1 AND is_published = true
        ORDER  BY order_index
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await
}
