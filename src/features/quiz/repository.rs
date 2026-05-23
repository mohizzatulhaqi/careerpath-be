use crate::features::quiz::entity::{QuizAttempt, Role};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

// ── Query row types (internal to this module) ─────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct QuestionOptionRow {
    pub question_id: Uuid,
    pub question_text: String,
    pub question_order: i32,
    pub option_id: Uuid,
    pub option_text: String,
    pub option_order: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AnswerWeightRow {
    pub question_id: Uuid,
    pub question_text: String,
    pub option_id: Uuid,
    pub option_text: String,
    pub role_id: Uuid,
    pub role_code: String,
    pub role_name: String,
    pub weight: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MaxPossibleRow {
    pub role_id: Uuid,
    pub role_code: String,
    pub role_name: String,
    pub max_total: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct HistoryRow {
    pub attempt_id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub role_name: String,
    pub role_code: String,
    pub match_percentage: f64,
}

// ── Questions & options ───────────────────────────────────────────────────────

pub async fn get_active_question_option_rows(
    pool: &PgPool,
) -> Result<Vec<QuestionOptionRow>, sqlx::Error> {
    sqlx::query_as::<_, QuestionOptionRow>(
        r#"
        SELECT q.id    AS question_id,
               q.question_text,
               q.order_index AS question_order,
               o.id    AS option_id,
               o.option_text,
               o.order_index AS option_order
        FROM   quiz_questions q
        JOIN   quiz_options   o ON o.question_id = q.id
        WHERE  q.is_active = true
        ORDER  BY q.order_index, o.order_index
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_active_questions(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM quiz_questions WHERE is_active = true",
    )
    .fetch_one(pool)
    .await
}

// ── Attempts ──────────────────────────────────────────────────────────────────

pub async fn find_in_progress_attempt(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<QuizAttempt>, sqlx::Error> {
    sqlx::query_as::<_, QuizAttempt>(
        r#"
        SELECT id, user_id, status, started_at, submitted_at, result_role_id, match_percentage
        FROM   quiz_attempts
        WHERE  user_id = $1 AND status = 'in_progress'
        LIMIT  1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn create_attempt(pool: &PgPool, user_id: Uuid) -> Result<QuizAttempt, sqlx::Error> {
    sqlx::query_as::<_, QuizAttempt>(
        r#"
        INSERT INTO quiz_attempts (id, user_id, status, started_at)
        VALUES (gen_random_uuid(), $1, 'in_progress', now())
        RETURNING id, user_id, status, started_at, submitted_at, result_role_id, match_percentage
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn find_attempt_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<QuizAttempt>, sqlx::Error> {
    sqlx::query_as::<_, QuizAttempt>(
        r#"
        SELECT id, user_id, status, started_at, submitted_at, result_role_id, match_percentage
        FROM   quiz_attempts
        WHERE  id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

// ── Answers ───────────────────────────────────────────────────────────────────

/// Returns true when option_id belongs to question_id.
pub async fn option_belongs_to_question(
    pool: &PgPool,
    option_id: Uuid,
    question_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM quiz_options WHERE id = $1 AND question_id = $2)",
    )
    .bind(option_id)
    .bind(question_id)
    .fetch_one(pool)
    .await
}

pub async fn upsert_answer(
    pool: &PgPool,
    attempt_id: Uuid,
    question_id: Uuid,
    option_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO quiz_attempt_answers (id, attempt_id, question_id, option_id, answered_at)
        VALUES (gen_random_uuid(), $1, $2, $3, now())
        ON CONFLICT (attempt_id, question_id)
        DO UPDATE SET option_id = EXCLUDED.option_id, answered_at = now()
        "#,
    )
    .bind(attempt_id)
    .bind(question_id)
    .bind(option_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_attempt_answers(
    pool: &PgPool,
    attempt_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM quiz_attempt_answers WHERE attempt_id = $1",
    )
    .bind(attempt_id)
    .fetch_one(pool)
    .await
}

// ── Scoring data ──────────────────────────────────────────────────────────────

pub async fn get_answers_with_weights(
    pool: &PgPool,
    attempt_id: Uuid,
) -> Result<Vec<AnswerWeightRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerWeightRow>(
        r#"
        SELECT aa.question_id,
               q.question_text,
               aa.option_id,
               o.option_text,
               orw.role_id,
               r.code  AS role_code,
               r.name  AS role_name,
               orw.weight
        FROM   quiz_attempt_answers  aa
        JOIN   quiz_questions         q   ON q.id  = aa.question_id
        JOIN   quiz_options           o   ON o.id  = aa.option_id
        JOIN   option_role_weights    orw ON orw.option_id = aa.option_id
        JOIN   roles                  r   ON r.id  = orw.role_id
        WHERE  aa.attempt_id = $1
        ORDER  BY aa.question_id, r.code
        "#,
    )
    .bind(attempt_id)
    .fetch_all(pool)
    .await
}

pub async fn get_max_possible_per_role(
    pool: &PgPool,
) -> Result<Vec<MaxPossibleRow>, sqlx::Error> {
    sqlx::query_as::<_, MaxPossibleRow>(
        r#"
        SELECT r.id   AS role_id,
               r.code AS role_code,
               r.name AS role_name,
               COALESCE(SUM(mx.max_weight), 0)::int AS max_total
        FROM   roles r
        LEFT JOIN (
            SELECT orw.role_id,
                   MAX(orw.weight) AS max_weight
            FROM   quiz_questions      q
            JOIN   quiz_options        o   ON o.question_id = q.id
            JOIN   option_role_weights orw ON orw.option_id = o.id
            WHERE  q.is_active = true
            GROUP  BY q.id, orw.role_id
        ) mx ON mx.role_id = r.id
        GROUP  BY r.id, r.code, r.name
        "#,
    )
    .fetch_all(pool)
    .await
}

// ── Submit (inside a caller-managed transaction) ──────────────────────────────

pub async fn update_attempt_submitted(
    tx: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
    role_id: Uuid,
    match_percentage: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE quiz_attempts
        SET    status = 'submitted',
               submitted_at     = now(),
               result_role_id   = $2,
               match_percentage = $3
        WHERE  id = $1
        "#,
    )
    .bind(attempt_id)
    .bind(role_id)
    .bind(match_percentage)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

// ── Result & history ──────────────────────────────────────────────────────────

pub async fn find_role_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Role>, sqlx::Error> {
    sqlx::query_as::<_, Role>(
        "SELECT id, code, name, description, is_active, created_at FROM roles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<HistoryRow>, sqlx::Error> {
    sqlx::query_as::<_, HistoryRow>(
        r#"
        SELECT a.id              AS attempt_id,
               a.submitted_at,
               r.name            AS role_name,
               r.code            AS role_code,
               a.match_percentage
        FROM   quiz_attempts a
        JOIN   roles          r ON r.id = a.result_role_id
        WHERE  a.user_id = $1
          AND  a.status  = 'submitted'
        ORDER  BY a.submitted_at DESC
        LIMIT  $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_history(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM quiz_attempts WHERE user_id = $1 AND status = 'submitted'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}
