use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        submaterial_quiz::{
            dto::{
                AdminSubQuizQuestionDto, CreateSubQuizQuestionRequest, ReplaceOptionsRequest,
                UpdateSubQuizQuestionRequest,
            },
            repository,
        },
    },
};

fn validate_options(options: &[crate::features::admin::content::submaterial_quiz::dto::QuizOptionInput]) -> Result<(), ContentError> {
    let correct_count = options.iter().filter(|o| o.is_correct).count();
    if correct_count != 1 {
        return Err(ContentError::InvalidStructure(format!(
            "mini quiz harus memiliki tepat 1 jawaban benar, ditemukan {correct_count}"
        )));
    }
    if options.len() < 2 {
        return Err(ContentError::InvalidStructure(
            "mini quiz harus memiliki minimal 2 pilihan jawaban".into(),
        ));
    }
    Ok(())
}

pub async fn list_questions(
    pool: &sqlx::PgPool,
    submaterial_id: Uuid,
) -> Result<Vec<AdminSubQuizQuestionDto>, ContentError> {
    repository::list_questions_for_sub(pool, submaterial_id)
        .await
        .map_err(ContentError::Internal)
}

pub async fn create_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    req: &CreateSubQuizQuestionRequest,
) -> Result<Uuid, ContentError> {
    validate_options(&req.options)?;

    if !repository::submaterial_exists(pool, req.submaterial_id)
        .await
        .map_err(ContentError::Internal)?
    {
        return Err(ContentError::ParentNotFound);
    }

    let order_index = match req.order_index {
        Some(i) => i,
        None => repository::next_order_index(pool, req.submaterial_id)
            .await
            .map_err(ContentError::Internal)?,
    };

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    let id = repository::create_question(&mut tx, req, order_index)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "submaterial_quiz_question.created",
        "submaterial",
        Some(req.submaterial_id),
        Some(json!({ "question_id": id })),
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
    req: &UpdateSubQuizQuestionRequest,
) -> Result<(), ContentError> {
    let q = repository::find_question(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::update_question(&mut tx, id, req).await.map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "submaterial_quiz_question.updated",
        "submaterial",
        Some(q.submaterial_id),
        Some(json!({ "question_id": id })),
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
    req: &ReplaceOptionsRequest,
    force: bool,
) -> Result<(), ContentError> {
    validate_options(&req.options)?;

    let q = repository::find_question(pool, question_id)
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
                "pertanyaan ini sudah ada dalam {attempts} jawaban user. \
                Set ?force=true untuk mengganti pilihan dan membuat jawaban lama tidak valid."
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
        "submaterial_quiz_question.options_updated",
        "submaterial",
        Some(q.submaterial_id),
        Some(json!({ "question_id": question_id, "forced": force, "attempts_count": attempts })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn delete_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    question_id: Uuid,
    force: bool,
) -> Result<(), ContentError> {
    let q = repository::find_question(pool, question_id)
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
                "pertanyaan ini ada dalam {attempts} jawaban user. Set ?force=true untuk hapus."
            ),
        });
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::delete_question(&mut tx, question_id)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "submaterial_quiz_question.deleted",
        "submaterial",
        Some(q.submaterial_id),
        Some(json!({ "question_id": question_id, "forced": force, "attempts_count": attempts })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::admin::content::submaterial_quiz::dto::QuizOptionInput;

    fn opt(is_correct: bool) -> QuizOptionInput {
        QuizOptionInput { text: "opt".into(), is_correct, order_index: 1 }
    }

    #[test]
    fn zero_correct_fails() {
        let opts = vec![opt(false), opt(false)];
        assert!(validate_options(&opts).is_err());
    }

    #[test]
    fn two_correct_fails() {
        let opts = vec![opt(true), opt(true)];
        assert!(validate_options(&opts).is_err());
    }

    #[test]
    fn exactly_one_correct_passes() {
        let opts = vec![opt(true), opt(false), opt(false)];
        assert!(validate_options(&opts).is_ok());
    }

    #[test]
    fn single_option_fails() {
        let opts = vec![opt(true)];
        assert!(validate_options(&opts).is_err());
    }
}
