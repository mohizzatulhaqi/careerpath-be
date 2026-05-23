use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        pre_quiz::{
            dto::{
                CreatePreQuizQuestionRequest, PreQuizFilter, PreQuizListResponse,
                PreQuizQuestionDto, ReplacePreQuizOptionsRequest, UpdatePreQuizQuestionRequest, PreQuizOptionInput,
            },
            repository,
        },
    },
};

/// Every option must have weights for ALL active roles.
async fn validate_options_weights(
    pool: &sqlx::PgPool,
    options: &[PreQuizOptionInput],
) -> Result<(), ContentError> {
    if options.len() < 2 {
        return Err(ContentError::InvalidStructure("minimal 2 pilihan jawaban".into()));
    }

    let active_role_ids: HashSet<Uuid> = repository::get_active_role_ids(pool)
        .await
        .map_err(ContentError::Internal)?
        .into_iter()
        .collect();

    for (i, opt) in options.iter().enumerate() {
        let provided_ids: HashSet<Uuid> = opt.weights.iter().map(|w| w.role_id).collect();
        for rid in &active_role_ids {
            if !provided_ids.contains(rid) {
                return Err(ContentError::InvalidStructure(format!(
                    "option {i} tidak memiliki weight untuk role {rid}"
                )));
            }
        }
        for w in &opt.weights {
            if w.weight < 0 || w.weight > 10 {
                return Err(ContentError::InvalidStructure(format!(
                    "weight harus antara 0–10, ditemukan {}",
                    w.weight
                )));
            }
        }
    }
    Ok(())
}

pub async fn list_questions(
    pool: &sqlx::PgPool,
    f: &PreQuizFilter,
) -> Result<PreQuizListResponse, ContentError> {
    repository::list_questions(pool, f).await.map_err(ContentError::Internal)
}

pub async fn get_question(pool: &sqlx::PgPool, id: Uuid) -> Result<PreQuizQuestionDto, ContentError> {
    repository::find_question(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)
}

pub async fn create_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    req: &CreatePreQuizQuestionRequest,
) -> Result<Uuid, ContentError> {
    validate_options_weights(pool, &req.options).await?;

    let order_index = match req.order_index {
        Some(i) => i,
        None => repository::next_order_index(pool).await.map_err(ContentError::Internal)?,
    };

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    let id = repository::create_question(&mut tx, req, order_index)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "pre_quiz_question.created",
        "quiz_question",
        Some(id),
        Some(json!({ "question_text": req.question_text })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(id)
}

pub async fn update_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
    req: &UpdatePreQuizQuestionRequest,
) -> Result<(), ContentError> {
    let _ = repository::find_question(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::update_question(&mut tx, id, req).await.map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "pre_quiz_question.updated",
        "quiz_question",
        Some(id),
        Some(json!({ "question_text": req.question_text, "is_active": req.is_active })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn replace_options(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    question_id: Uuid,
    req: &ReplacePreQuizOptionsRequest,
    force: bool,
) -> Result<(), ContentError> {
    validate_options_weights(pool, &req.options).await?;

    let _ = repository::find_question(pool, question_id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let attempts = repository::count_attempts_for_question(pool, question_id)
        .await
        .map_err(ContentError::Internal)?;

    if attempts > 0 && !force {
        return Err(ContentError::RequiresForce {
            count: attempts,
            message: format!(
                "pertanyaan ini dijawab {attempts} kali. Mengganti pilihan TIDAK mengubah hasil yang sudah ada. \
                Set ?force=true jika sudah paham."
            ),
        });
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::replace_options(&mut tx, question_id, &req.options)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "pre_quiz_question.options_updated",
        "quiz_question",
        Some(question_id),
        Some(json!({ "forced": force, "attempts_count": attempts })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn deactivate_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _ = repository::find_question(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_question_active(&mut tx, id, false)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "pre_quiz_question.deactivated", "quiz_question", Some(id), None)
        .await
        .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn restore_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _ = repository::find_question(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_question_active(&mut tx, id, true)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "pre_quiz_question.restored", "quiz_question", Some(id), None)
        .await
        .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}
