use crate::features::learning::module_quiz::entity::{
    ModuleQuizAttempt, ModuleQuizOption, ModuleQuizQuestion,
};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Fetch **published** questions for a module (user-facing).
pub async fn get_questions(
    pool: &PgPool,
    module_id: Uuid,
) -> Result<Vec<ModuleQuizQuestion>, sqlx::Error> {
    sqlx::query_as::<_, ModuleQuizQuestion>(
        r#"
        SELECT id, module_id, question, order_index, is_published, created_at
        FROM   module_quizzes
        WHERE  module_id = $1 AND is_published = true
        ORDER  BY order_index
        "#,
    )
    .bind(module_id)
    .fetch_all(pool)
    .await
}

pub async fn get_options_for_questions(
    pool: &PgPool,
    question_ids: &[Uuid],
) -> Result<Vec<ModuleQuizOption>, sqlx::Error> {
    sqlx::query_as::<_, ModuleQuizOption>(
        r#"
        SELECT id, question_id, text, is_correct, order_index
        FROM   module_quiz_options
        WHERE  question_id = ANY($1)
        ORDER  BY question_id, order_index
        "#,
    )
    .bind(question_ids)
    .fetch_all(pool)
    .await
}

pub async fn insert_attempt(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    module_id: Uuid,
    score: f64,
    passed: bool,
) -> Result<ModuleQuizAttempt, sqlx::Error> {
    sqlx::query_as::<_, ModuleQuizAttempt>(
        r#"
        INSERT INTO module_quiz_attempts (user_id, module_id, score, passed)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, module_id, score, passed, created_at
        "#,
    )
    .bind(user_id)
    .bind(module_id)
    .bind(score)
    .bind(passed)
    .fetch_one(&mut **tx)
    .await
}

pub async fn insert_answer(
    tx: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
    question_id: Uuid,
    option_id: Uuid,
    is_correct: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO module_quiz_attempt_answers
            (attempt_id, question_id, option_id, is_correct)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(attempt_id)
    .bind(question_id)
    .bind(option_id)
    .bind(is_correct)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn get_attempts_for_user(
    pool: &PgPool,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<Vec<ModuleQuizAttempt>, sqlx::Error> {
    sqlx::query_as::<_, ModuleQuizAttempt>(
        r#"
        SELECT id, user_id, module_id, score, passed, created_at
        FROM   module_quiz_attempts
        WHERE  user_id = $1 AND module_id = $2
        ORDER  BY created_at DESC
        "#,
    )
    .bind(user_id)
    .bind(module_id)
    .fetch_all(pool)
    .await
}
