use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::{
    audit::dto::PaginationDto,
    content::pre_quiz::dto::{
        CreatePreQuizQuestionRequest, PreQuizFilter, PreQuizListResponse, PreQuizOptionDto,
        PreQuizOptionInput, PreQuizQuestionDto, UpdatePreQuizQuestionRequest, WeightDto,
    },
};

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn list_questions(
    pool: &PgPool,
    f: &PreQuizFilter,
) -> anyhow::Result<PreQuizListResponse> {
    let page = f.page.unwrap_or(1).max(1);
    let per_page = f.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM quiz_questions WHERE ($1::bool IS NULL OR is_active=$1)",
        f.is_active as _
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let questions = sqlx::query!(
        "SELECT id, question_text, order_index, is_active, created_at FROM quiz_questions WHERE ($1::bool IS NULL OR is_active=$1) ORDER BY order_index LIMIT $2 OFFSET $3",
        f.is_active as _, per_page, offset
    )
    .fetch_all(pool)
    .await
    .context("list pre-quiz questions")?;

    if questions.is_empty() {
        return Ok(PreQuizListResponse {
            questions: vec![],
            pagination: PaginationDto { page, per_page, total_items: 0, total_pages: 0 },
        });
    }

    let q_ids: Vec<Uuid> = questions.iter().map(|q| q.id).collect();

    let options = sqlx::query!(
        "SELECT id, question_id, option_text, order_index FROM quiz_options WHERE question_id=ANY($1) ORDER BY question_id, order_index",
        &q_ids as _
    )
    .fetch_all(pool)
    .await?;

    let opt_ids: Vec<Uuid> = options.iter().map(|o| o.id).collect();

    let weights = sqlx::query!(
        "SELECT orw.option_id, orw.role_id, r.code AS role_code, orw.weight FROM option_role_weights orw JOIN roles r ON r.id=orw.role_id WHERE orw.option_id=ANY($1)",
        &opt_ids as _
    )
    .fetch_all(pool)
    .await?;

    let total_pages = (total + per_page - 1) / per_page;

    Ok(PreQuizListResponse {
        questions: questions
            .into_iter()
            .map(|q| PreQuizQuestionDto {
                id: q.id,
                question_text: q.question_text,
                order_index: q.order_index,
                is_active: q.is_active,
                created_at: q.created_at,
                options: options
                    .iter()
                    .filter(|o| o.question_id == q.id)
                    .map(|o| PreQuizOptionDto {
                        id: o.id,
                        option_text: o.option_text.clone(),
                        order_index: o.order_index,
                        weights: weights
                            .iter()
                            .filter(|w| w.option_id == o.id)
                            .map(|w| WeightDto {
                                role_id: w.role_id,
                                role_code: w.role_code.clone(),
                                weight: w.weight,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        pagination: PaginationDto { page, per_page, total_items: total, total_pages },
    })
}

// ── Single ────────────────────────────────────────────────────────────────────

pub async fn find_question(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<PreQuizQuestionDto>> {
    let q = sqlx::query!(
        "SELECT id, question_text, order_index, is_active, created_at FROM quiz_questions WHERE id=$1",
        id
    )
    .fetch_optional(pool)
    .await?;
    let Some(q) = q else { return Ok(None) };

    let options = sqlx::query!(
        "SELECT id, question_id, option_text, order_index FROM quiz_options WHERE question_id=$1 ORDER BY order_index",
        id
    )
    .fetch_all(pool)
    .await?;

    let opt_ids: Vec<Uuid> = options.iter().map(|o| o.id).collect();
    let weights = sqlx::query!(
        "SELECT orw.option_id, orw.role_id, r.code AS role_code, orw.weight FROM option_role_weights orw JOIN roles r ON r.id=orw.role_id WHERE orw.option_id=ANY($1)",
        &opt_ids as _
    )
    .fetch_all(pool)
    .await?;

    Ok(Some(PreQuizQuestionDto {
        id: q.id,
        question_text: q.question_text,
        order_index: q.order_index,
        is_active: q.is_active,
        created_at: q.created_at,
        options: options
            .into_iter()
            .map(|o| PreQuizOptionDto {
                id: o.id,
                option_text: o.option_text,
                order_index: o.order_index,
                weights: weights
                    .iter()
                    .filter(|w| w.option_id == o.id)
                    .map(|w| WeightDto {
                        role_id: w.role_id,
                        role_code: w.role_code.clone(),
                        weight: w.weight,
                    })
                    .collect(),
            })
            .collect(),
    }))
}

// ── Create ────────────────────────────────────────────────────────────────────

pub async fn create_question(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreatePreQuizQuestionRequest,
    order_index: i32,
) -> anyhow::Result<Uuid> {
    let id = sqlx::query_scalar!(
        "INSERT INTO quiz_questions (question_text, order_index) VALUES ($1,$2) RETURNING id",
        req.question_text,
        order_index
    )
    .fetch_one(&mut **tx)
    .await?;

    insert_options_with_weights(tx, id, &req.options).await?;
    Ok(id)
}

pub async fn update_question(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    req: &UpdatePreQuizQuestionRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE quiz_questions SET question_text=COALESCE($2,question_text), order_index=COALESCE($3,order_index), is_active=COALESCE($4,is_active), updated_at=NOW() WHERE id=$1",
        id, req.question_text as _, req.order_index as _, req.is_active as _
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn replace_options(
    tx: &mut Transaction<'_, Postgres>,
    question_id: Uuid,
    options: &[PreQuizOptionInput],
) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM quiz_options WHERE question_id=$1", question_id)
        .execute(&mut **tx)
        .await?;
    insert_options_with_weights(tx, question_id, options).await
}

async fn insert_options_with_weights(
    tx: &mut Transaction<'_, Postgres>,
    question_id: Uuid,
    options: &[PreQuizOptionInput],
) -> anyhow::Result<()> {
    for opt in options {
        let opt_id = sqlx::query_scalar!(
            "INSERT INTO quiz_options (question_id, option_text, order_index) VALUES ($1,$2,$3) RETURNING id",
            question_id, opt.option_text, opt.order_index
        )
        .fetch_one(&mut **tx)
        .await?;

        for w in &opt.weights {
            sqlx::query!(
                "INSERT INTO option_role_weights (option_id, role_id, weight) VALUES ($1,$2,$3)",
                opt_id, w.role_id, w.weight
            )
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

pub async fn set_question_active(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    active: bool,
) -> anyhow::Result<()> {
    sqlx::query!("UPDATE quiz_questions SET is_active=$2, updated_at=NOW() WHERE id=$1", id, active)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn count_attempts_for_question(pool: &PgPool, question_id: Uuid) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!(
        "SELECT COUNT(*) FROM quiz_attempt_answers WHERE question_id=$1",
        question_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}

pub async fn next_order_index(pool: &PgPool) -> anyhow::Result<i32> {
    let max = sqlx::query_scalar!("SELECT COALESCE(MAX(order_index),0) FROM quiz_questions")
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
    Ok(max + 1)
}

pub async fn get_active_role_ids(pool: &PgPool) -> anyhow::Result<Vec<Uuid>> {
    Ok(sqlx::query_scalar!("SELECT id FROM roles WHERE is_active=true ORDER BY code")
        .fetch_all(pool)
        .await?)
}
