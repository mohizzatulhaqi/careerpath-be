use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::content::module_quiz::dto::{
    AdminModuleQuizOptionDto, AdminModuleQuizQuestionDto, CreateModuleQuizQuestionRequest,
    ModuleQuizOptionInput, UpdateModuleQuizQuestionRequest,
};

pub async fn list_questions_for_module(
    pool: &PgPool,
    module_id: Uuid,
) -> anyhow::Result<Vec<AdminModuleQuizQuestionDto>> {
    let questions = sqlx::query!(
        "SELECT id, module_id, question, order_index, is_published, created_at FROM module_quizzes WHERE module_id=$1 ORDER BY order_index",
        module_id
    )
    .fetch_all(pool)
    .await
    .context("list_module_quiz_questions")?;

    if questions.is_empty() {
        return Ok(vec![]);
    }

    let ids: Vec<Uuid> = questions.iter().map(|q| q.id).collect();
    let options = sqlx::query!(
        "SELECT id, question_id, text, is_correct, order_index FROM module_quiz_options WHERE question_id=ANY($1) ORDER BY question_id, order_index",
        &ids as _
    )
    .fetch_all(pool)
    .await?;

    Ok(questions
        .into_iter()
        .map(|q| AdminModuleQuizQuestionDto {
            id: q.id,
            module_id: q.module_id,
            question: q.question,
            order_index: q.order_index,
            is_published: q.is_published,
            created_at: q.created_at,
            options: options
                .iter()
                .filter(|o| o.question_id == q.id)
                .map(|o| AdminModuleQuizOptionDto {
                    id: o.id,
                    text: o.text.clone(),
                    is_correct: o.is_correct,
                    order_index: o.order_index,
                })
                .collect(),
        })
        .collect())
}

pub async fn find_question(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AdminModuleQuizQuestionDto>> {
    let q = sqlx::query!(
        "SELECT id, module_id, question, order_index, is_published, created_at FROM module_quizzes WHERE id=$1",
        id
    )
    .fetch_optional(pool)
    .await?;
    let Some(q) = q else { return Ok(None) };

    let options = sqlx::query!(
        "SELECT id, question_id, text, is_correct, order_index FROM module_quiz_options WHERE question_id=$1 ORDER BY order_index",
        id
    )
    .fetch_all(pool)
    .await?;

    Ok(Some(AdminModuleQuizQuestionDto {
        id: q.id,
        module_id: q.module_id,
        question: q.question,
        order_index: q.order_index,
        is_published: q.is_published,
        created_at: q.created_at,
        options: options
            .into_iter()
            .map(|o| AdminModuleQuizOptionDto {
                id: o.id,
                text: o.text,
                is_correct: o.is_correct,
                order_index: o.order_index,
            })
            .collect(),
    }))
}

pub async fn create_question(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateModuleQuizQuestionRequest,
    order_index: i32,
) -> anyhow::Result<Uuid> {
    let id = sqlx::query_scalar!(
        "INSERT INTO module_quizzes (module_id, question, order_index) VALUES ($1,$2,$3) RETURNING id",
        req.module_id, req.question_text, order_index
    )
    .fetch_one(&mut **tx)
    .await?;

    insert_options(tx, id, &req.options).await?;
    Ok(id)
}

pub async fn update_question(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    req: &UpdateModuleQuizQuestionRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE module_quizzes SET question=COALESCE($2,question), order_index=COALESCE($3,order_index) WHERE id=$1",
        id, req.question_text as _, req.order_index as _
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn replace_options(
    tx: &mut Transaction<'_, Postgres>,
    question_id: Uuid,
    options: &[ModuleQuizOptionInput],
) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM module_quiz_options WHERE question_id=$1", question_id)
        .execute(&mut **tx)
        .await?;
    insert_options(tx, question_id, options).await
}

async fn insert_options(
    tx: &mut Transaction<'_, Postgres>,
    question_id: Uuid,
    options: &[ModuleQuizOptionInput],
) -> anyhow::Result<()> {
    for opt in options {
        sqlx::query!(
            "INSERT INTO module_quiz_options (question_id, text, is_correct, order_index) VALUES ($1,$2,$3,$4)",
            question_id, opt.text, opt.is_correct, opt.order_index
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn delete_question(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM module_quizzes WHERE id=$1", id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn count_attempts_for_question(pool: &PgPool, question_id: Uuid) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!(
        "SELECT COUNT(*) FROM module_quiz_attempt_answers WHERE question_id=$1",
        question_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}

pub async fn next_order_index(pool: &PgPool, module_id: Uuid) -> anyhow::Result<i32> {
    let max = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(order_index),0) FROM module_quizzes WHERE module_id=$1",
        module_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    Ok(max + 1)
}

pub async fn module_exists(pool: &PgPool, id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM learning_modules WHERE id=$1)",
        id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false))
}
