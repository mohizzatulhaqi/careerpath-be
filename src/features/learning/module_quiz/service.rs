use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::learning::{
        error::LearningError,
        module::repository as module_repo,
        module_quiz::{
            dto::{
                FinalQuizAttemptItem, FinalQuizHistoryResponse, FinalQuizOptionResponse,
                FinalQuizQuestionResponse, FinalQuizResponse, FinalQuizResultOption,
                FinalQuizResultQuestion, FinalQuizSubmitResponse, SubmitFinalQuizRequest,
            },
            repository,
            scoring::score_final_quiz,
        },
        progress::{
            gating::{self, ModuleInfo},
            repository as progress_repo,
            service as progress_service,
        },
        submaterial::repository as sub_repo,
        submaterial_quiz::service::compute_completed_ids,
    },
    state::AppState,
};

/// GET /api/learning/modules/:id/quiz
pub async fn get_final_quiz(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<FinalQuizResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let module = module_repo::find_by_id(&state.db, module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    if module.role_id != role_id {
        return Err(LearningError::ModuleNotForUserRole);
    }

    assert_module_unlocked(state, user_id, role_id, module.order_index).await?;
    assert_all_submaterials_complete(state, user_id, module_id).await?;

    let questions = repository::get_questions(&state.db, module_id).await?;
    if questions.is_empty() {
        return Err(LearningError::QuizNotFound);
    }

    let q_ids: Vec<Uuid> = questions.iter().map(|q| q.id).collect();
    let options = repository::get_options_for_questions(&state.db, &q_ids).await?;

    let mut opts_by_q: HashMap<Uuid, Vec<&_>> = HashMap::new();
    for opt in &options {
        opts_by_q.entry(opt.question_id).or_default().push(opt);
    }

    let questions_resp = questions
        .iter()
        .map(|q| {
            let opts = opts_by_q
                .get(&q.id)
                .map(|v| {
                    v.iter()
                        .map(|o| FinalQuizOptionResponse {
                            id:          o.id,
                            text:        o.text.clone(),
                            order_index: o.order_index,
                        })
                        .collect()
                })
                .unwrap_or_default();
            FinalQuizQuestionResponse {
                id:          q.id,
                question:    q.question.clone(),
                order_index: q.order_index,
                options:     opts,
            }
        })
        .collect();

    Ok(FinalQuizResponse { module_id, questions: questions_resp })
}

/// POST /api/learning/modules/:id/quiz/submit
pub async fn submit_final_quiz(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
    req: SubmitFinalQuizRequest,
) -> Result<FinalQuizSubmitResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let module = module_repo::find_by_id(&state.db, module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    if module.role_id != role_id {
        return Err(LearningError::ModuleNotForUserRole);
    }

    assert_module_unlocked(state, user_id, role_id, module.order_index).await?;
    assert_all_submaterials_complete(state, user_id, module_id).await?;

    let questions = repository::get_questions(&state.db, module_id).await?;
    if questions.is_empty() {
        return Err(LearningError::QuizNotFound);
    }

    if req.answers.len() != questions.len() {
        return Err(LearningError::IncompleteAnswers);
    }

    let q_ids: Vec<Uuid> = questions.iter().map(|q| q.id).collect();
    let all_options = repository::get_options_for_questions(&state.db, &q_ids).await?;

    let mut opts_by_q: HashMap<Uuid, Vec<&_>> = HashMap::new();
    for opt in &all_options {
        opts_by_q.entry(opt.question_id).or_default().push(opt);
    }

    let answer_map: HashMap<Uuid, Uuid> = req
        .answers
        .iter()
        .map(|a| (a.question_id, a.option_id))
        .collect();

    for q in &questions {
        if !answer_map.contains_key(&q.id) {
            return Err(LearningError::IncompleteAnswers);
        }
    }

    let mut correct_count = 0usize;
    let mut result_questions = Vec::with_capacity(questions.len());

    for q in &questions {
        let chosen_option_id = answer_map[&q.id];
        let opts = opts_by_q.get(&q.id).map(|v| v.as_slice()).unwrap_or(&[]);

        let chosen = opts
            .iter()
            .find(|o| o.id == chosen_option_id)
            .ok_or(LearningError::InvalidOption)?;

        let correct_opt = opts.iter().find(|o| o.is_correct);
        let correct_option_id = correct_opt.map(|o| o.id).unwrap_or(chosen_option_id);

        if chosen.is_correct {
            correct_count += 1;
        }

        let result_opts = opts
            .iter()
            .map(|o| FinalQuizResultOption {
                id:          o.id,
                text:        o.text.clone(),
                order_index: o.order_index,
                is_correct:  o.is_correct,
            })
            .collect();

        result_questions.push(FinalQuizResultQuestion {
            id:                q.id,
            question:          q.question.clone(),
            order_index:       q.order_index,
            your_answer_id:    chosen_option_id,
            is_correct:        chosen.is_correct,
            correct_option_id,
            options:           result_opts,
        });
    }

    let (score, passed) = score_final_quiz(correct_count, questions.len());

    let mut tx = state.db.begin().await?;
    let attempt = repository::insert_attempt(&mut tx, user_id, module_id, score, passed).await?;

    for q in &questions {
        let chosen_option_id = answer_map[&q.id];
        let opts = opts_by_q.get(&q.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let is_correct = opts.iter().any(|o| o.id == chosen_option_id && o.is_correct);
        repository::insert_answer(&mut tx, attempt.id, q.id, chosen_option_id, is_correct).await?;
    }

    tx.commit().await?;

    Ok(FinalQuizSubmitResponse {
        attempt_id: attempt.id,
        score,
        passed,
        questions:  result_questions,
    })
}

/// GET /api/learning/modules/:id/quiz/history
pub async fn get_final_quiz_history(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<FinalQuizHistoryResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let module = module_repo::find_by_id(&state.db, module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    if module.role_id != role_id {
        return Err(LearningError::ModuleNotForUserRole);
    }

    let attempts = repository::get_attempts_for_user(&state.db, user_id, module_id).await?;

    let best_score = attempts.iter().map(|a| a.score).reduce(f64::max);

    let items = attempts
        .into_iter()
        .map(|a| FinalQuizAttemptItem {
            attempt_id: a.id,
            score:      a.score,
            passed:     a.passed,
            created_at: a.created_at,
        })
        .collect();

    Ok(FinalQuizHistoryResponse { module_id, best_score, attempts: items })
}

// ── Private helpers ───────────────────────────────────────────────────────────

async fn assert_module_unlocked(
    state: &Arc<AppState>,
    user_id: Uuid,
    role_id: Uuid,
    target_order_index: i32,
) -> Result<(), LearningError> {
    let rows = module_repo::get_modules_with_counts(&state.db, role_id, user_id).await?;
    let quiz_status =
        progress_repo::get_final_quiz_status_per_module(&state.db, role_id, user_id).await?;

    let module_infos: Vec<ModuleInfo> = rows
        .iter()
        .map(|r| ModuleInfo { module_id: r.id, order_index: r.order_index })
        .collect();

    let completed_ids = compute_completed_ids(&rows, &quiz_status);
    let gates = gating::compute_module_gates(&module_infos, &completed_ids);

    let is_unlocked = gates
        .iter()
        .zip(module_infos.iter())
        .find(|(_, mi)| mi.order_index == target_order_index)
        .map(|(g, _)| g.is_unlocked)
        .unwrap_or(false);

    if !is_unlocked {
        return Err(LearningError::ModuleLocked);
    }
    Ok(())
}

/// Assert all submaterials in the module are completed (mini quiz passed).
async fn assert_all_submaterials_complete(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<(), LearningError> {
    let subs = sub_repo::get_submaterials_for_module(&state.db, module_id).await?;
    let completed_ids =
        progress_repo::get_completed_submaterial_ids_for_module(&state.db, user_id, module_id)
            .await?;

    let all_done = !subs.is_empty() && subs.iter().all(|s| completed_ids.contains(&s.id));
    if !all_done {
        return Err(LearningError::FinalQuizLocked);
    }
    Ok(())
}
