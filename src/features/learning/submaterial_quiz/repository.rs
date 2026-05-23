use crate::features::learning::submaterial_quiz::entity::{
    SubMaterialQuizAttempt, SubMaterialQuizOption, SubMaterialQuizQuestion,
};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Fetch **published** questions for a submaterial (user-facing).
pub async fn get_questions(
    pool: &PgPool,
    submaterial_id: Uuid,
) -> Result<Vec<SubMaterialQuizQuestion>, sqlx::Error> {
    sqlx::query_as::<_, SubMaterialQuizQuestion>(
        r#"
        SELECT id, submaterial_id, question, order_index, is_published, created_at
        FROM   submaterial_quizzes
        WHERE  submaterial_id = $1 AND is_published = true
        ORDER  BY order_index
        "#,
    )
    .bind(submaterial_id)
    .fetch_all(pool)
    .await
}

pub async fn get_options_for_questions(
    pool: &PgPool,
    question_ids: &[Uuid],
) -> Result<Vec<SubMaterialQuizOption>, sqlx::Error> {
    sqlx::query_as::<_, SubMaterialQuizOption>(
        r#"
        SELECT id, question_id, text, is_correct, order_index
        FROM   submaterial_quiz_options
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
    submaterial_id: Uuid,
    score: f64,
    passed: bool,
) -> Result<SubMaterialQuizAttempt, sqlx::Error> {
    sqlx::query_as::<_, SubMaterialQuizAttempt>(
        r#"
        INSERT INTO submaterial_quiz_attempts (user_id, submaterial_id, score, passed)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, submaterial_id, score, passed, created_at
        "#,
    )
    .bind(user_id)
    .bind(submaterial_id)
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
        INSERT INTO submaterial_quiz_attempt_answers
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
