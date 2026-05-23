use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        module_quiz::{
            dto::{
                AdminModuleQuizQuestionDto, CreateModuleQuizQuestionRequest,
                ReplaceModuleQuizOptionsRequest, UpdateModuleQuizQuestionRequest,
            },
            repository,
        },
    },
};

fn validate_options(options: &[crate::features::admin::content::module_quiz::dto::ModuleQuizOptionInput]) -> Result<(), ContentError> {
    let correct = options.iter().filter(|o| o.is_correct).count();
    if correct == 0 {
        return Err(ContentError::InvalidStructure("harus ada minimal 1 jawaban benar".into()));
    }
    if options.len() < 2 {
        return Err(ContentError::InvalidStructure("minimal 2 pilihan jawaban".into()));
    }
    Ok(())
}

pub async fn list_questions(
    pool: &sqlx::PgPool,
    module_id: Uuid,
) -> Result<Vec<AdminModuleQuizQuestionDto>, ContentError> {
    repository::list_questions_for_module(pool, module_id)
        .await
        .map_err(ContentError::Internal)
}

pub async fn create_question(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    req: &CreateModuleQuizQuestionRequest,
) -> Result<Uuid, ContentError> {
    validate_options(&req.options)?;

    if !repository::module_exists(pool, req.module_id).await.map_err(ContentError::Internal)? {
        return Err(ContentError::ParentNotFound);
    }

    let order_index = match req.order_index {
        Some(i) => i,
        None => repository::next_order_index(pool, req.module_id)
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
        "module_quiz_question.created",
        "module",
        Some(req.module_id),
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
    req: &UpdateModuleQuizQuestionRequest,
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
        "module_quiz_question.updated",
        "module",
        Some(q.module_id),
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
    req: &ReplaceModuleQuizOptionsRequest,
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
                "pertanyaan ini ada dalam {attempts} jawaban user. Set ?force=true untuk mengganti."
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
        "module_quiz_question.options_updated",
        "module",
        Some(q.module_id),
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
        "module_quiz_question.deleted",
        "module",
        Some(q.module_id),
        Some(json!({ "question_id": question_id, "forced": force, "attempts_count": attempts })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::admin::content::module_quiz::dto::ModuleQuizOptionInput;

    fn opt(is_correct: bool) -> ModuleQuizOptionInput {
        ModuleQuizOptionInput { text: "opt".into(), is_correct, order_index: 1 }
    }

    #[test]
    fn no_correct_fails() { assert!(validate_options(&[opt(false), opt(false)]).is_err()); }
    #[test]
    fn at_least_one_correct_ok() { assert!(validate_options(&[opt(true), opt(false)]).is_ok()); }
    #[test]
    fn multiple_correct_ok_for_final_quiz() {
        // Final quiz allows multiple correct answers
        assert!(validate_options(&[opt(true), opt(true), opt(false)]).is_ok());
    }
}
