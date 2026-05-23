use crate::features::learning::progress::entity::UserSubmaterialProgress;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Row type for progress entries within a module.
#[derive(Debug, sqlx::FromRow)]
pub struct SubmaterialProgressRow {
    pub submaterial_id: Uuid,
    pub completed_at: DateTime<Utc>,
}

/// Get the set of completed submaterial IDs for a user within a specific module.
pub async fn get_completed_submaterial_ids_for_module(
    pool: &PgPool,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<HashSet<Uuid>, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT usp.submaterial_id
        FROM   user_submaterial_progress usp
        JOIN   submaterials s ON s.id = usp.submaterial_id
        WHERE  usp.user_id = $1 AND s.module_id = $2
        "#,
    )
    .bind(user_id)
    .bind(module_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().collect())
}

/// Get progress rows for a user within a specific module (for completed_at timestamps).
pub async fn get_progress_for_module(
    pool: &PgPool,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<Vec<SubmaterialProgressRow>, sqlx::Error> {
    sqlx::query_as::<_, SubmaterialProgressRow>(
        r#"
        SELECT usp.submaterial_id, usp.completed_at
        FROM   user_submaterial_progress usp
        JOIN   submaterials s ON s.id = usp.submaterial_id
        WHERE  usp.user_id = $1 AND s.module_id = $2
        "#,
    )
    .bind(user_id)
    .bind(module_id)
    .fetch_all(pool)
    .await
}

/// Upsert submaterial progress. Idempotent: ON CONFLICT DO NOTHING, then fetch.
pub async fn upsert_submaterial_progress(
    pool: &PgPool,
    user_id: Uuid,
    submaterial_id: Uuid,
) -> Result<UserSubmaterialProgress, sqlx::Error> {
    // Try insert, ignore if already exists
    sqlx::query(
        r#"
        INSERT INTO user_submaterial_progress (id, user_id, submaterial_id, completed_at)
        VALUES (gen_random_uuid(), $1, $2, NOW())
        ON CONFLICT (user_id, submaterial_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(submaterial_id)
    .execute(pool)
    .await?;

    // Always fetch the existing row
    sqlx::query_as::<_, UserSubmaterialProgress>(
        r#"
        SELECT id, user_id, submaterial_id, completed_at
        FROM   user_submaterial_progress
        WHERE  user_id = $1 AND submaterial_id = $2
        "#,
    )
    .bind(user_id)
    .bind(submaterial_id)
    .fetch_one(pool)
    .await
}

/// Count total completed submaterials for a user across a role's modules.
pub async fn count_completed_for_role(
    pool: &PgPool,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM   user_submaterial_progress usp
        JOIN   submaterials s ON s.id = usp.submaterial_id
        JOIN   learning_modules m ON m.id = s.module_id
        WHERE  usp.user_id = $1 AND m.role_id = $2 AND m.is_published = true
        "#,
    )
    .bind(user_id)
    .bind(role_id)
    .fetch_one(pool)
    .await
}

/// Per-module final quiz status: total questions + best attempt score (-1.0 if no attempts).
#[derive(Debug, sqlx::FromRow)]
pub struct ModuleQuizStatusRow {
    pub module_id:       Uuid,
    pub total_questions: i64,
    pub best_score:      f64,
}

/// Returns a map of module_id → (total_questions, best_score) for all published
/// modules in a role. Used to determine whether the final quiz condition is met.
pub async fn get_final_quiz_status_per_module(
    pool: &PgPool,
    role_id: Uuid,
    user_id: Uuid,
) -> Result<HashMap<Uuid, ModuleQuizStatusRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ModuleQuizStatusRow>(
        r#"
        SELECT
            lm.id                                      AS module_id,
            COUNT(DISTINCT mq.id)                      AS total_questions,
            COALESCE(MAX(mqa.score), -1.0::float8)     AS best_score
        FROM   learning_modules lm
        LEFT JOIN module_quizzes mq
               ON mq.module_id = lm.id AND mq.is_published = true
        LEFT JOIN module_quiz_attempts mqa
               ON mqa.module_id = lm.id AND mqa.user_id = $2
        WHERE  lm.role_id = $1 AND lm.is_published = true
        GROUP  BY lm.id
        "#,
    )
    .bind(role_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| (r.module_id, r)).collect())
}

/// Count total submaterials for a role.
pub async fn count_total_submaterials_for_role(
    pool: &PgPool,
    role_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM   submaterials s
        JOIN   learning_modules m ON m.id = s.module_id
        WHERE  m.role_id = $1 AND m.is_published = true
        "#,
    )
    .bind(role_id)
    .fetch_one(pool)
    .await
}
