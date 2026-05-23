use crate::features::learning::submaterial::entity::Submaterial;
use sqlx::PgPool;
use uuid::Uuid;

/// Fetch all **published** submaterials for a module, ordered by order_index.
/// User-facing: only published items.
pub async fn get_submaterials_for_module(
    pool: &PgPool,
    module_id: Uuid,
) -> Result<Vec<Submaterial>, sqlx::Error> {
    sqlx::query_as::<_, Submaterial>(
        r#"
        SELECT id, module_id, title, content, order_index, estimated_minutes,
               is_published, created_at, updated_at
        FROM   submaterials
        WHERE  module_id = $1 AND is_published = true
        ORDER  BY order_index
        "#,
    )
    .bind(module_id)
    .fetch_all(pool)
    .await
}

/// Fetch a single submaterial by ID (no is_published filter — detail always accessible).
pub async fn find_by_id(
    pool: &PgPool,
    submaterial_id: Uuid,
) -> Result<Option<Submaterial>, sqlx::Error> {
    sqlx::query_as::<_, Submaterial>(
        r#"
        SELECT id, module_id, title, content, order_index, estimated_minutes,
               is_published, created_at, updated_at
        FROM   submaterials
        WHERE  id = $1
        "#,
    )
    .bind(submaterial_id)
    .fetch_optional(pool)
    .await
}

/// Find the next **published** submaterial in the same module (by order_index).
pub async fn find_next_submaterial(
    pool: &PgPool,
    module_id: Uuid,
    current_order_index: i32,
) -> Result<Option<Submaterial>, sqlx::Error> {
    sqlx::query_as::<_, Submaterial>(
        r#"
        SELECT id, module_id, title, content, order_index, estimated_minutes,
               is_published, created_at, updated_at
        FROM   submaterials
        WHERE  module_id = $1 AND order_index > $2 AND is_published = true
        ORDER  BY order_index
        LIMIT  1
        "#,
    )
    .bind(module_id)
    .bind(current_order_index)
    .fetch_optional(pool)
    .await
}
