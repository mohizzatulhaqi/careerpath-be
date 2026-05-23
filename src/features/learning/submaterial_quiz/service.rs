use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::learning::{
        error::LearningError,
        module::repository as module_repo,
        progress::{
            gating::{self, ModuleInfo, SubmaterialInfo},
            repository as progress_repo,
            service as progress_service,
        },
        submaterial::repository as sub_repo,
        submaterial_quiz::{
            dto::{
                MiniQuizOptionResponse, MiniQuizQuestionResponse, MiniQuizResponse,
                MiniQuizResultOption, MiniQuizResultQuestion, MiniQuizSubmitResponse,
                SubmitMiniQuizRequest,
            },
            repository,
            scoring::score_mini_quiz,
        },
    },
    state::AppState,
};

/// GET /api/learning/submaterials/:id/quiz
pub async fn get_mini_quiz(
    state: &Arc<AppState>,
    user_id: Uuid,
    submaterial_id: Uuid,
) -> Result<MiniQuizResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let sub = sub_repo::find_by_id(&state.db, submaterial_id)
        .await?
        .ok_or(LearningError::SubmaterialNotFound)?;

    let module = module_repo::find_by_id(&state.db, sub.module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    if module.role_id != role_id {
        return Err(LearningError::SubmaterialNotForUserRole);
    }

    assert_module_unlocked(state, user_id, role_id, module.order_index).await?;
    assert_submaterial_unlocked(state, user_id, sub.module_id, sub.order_index).await?;

    let questions = repository::get_questions(&state.db, submaterial_id).await?;
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
                        .map(|o| MiniQuizOptionResponse {
                            id:          o.id,
                            text:        o.text.clone(),
                            order_index: o.order_index,
                        })
                        .collect()
                })
                .unwrap_or_default();
            MiniQuizQuestionResponse {
                id:          q.id,
                question:    q.question.clone(),
                order_index: q.order_index,
                options:     opts,
            }
        })
        .collect();

    Ok(MiniQuizResponse {
        submaterial_id,
        questions: questions_resp,
    })
}

/// POST /api/learning/submaterials/:id/quiz/submit
pub async fn submit_mini_quiz(
    state: &Arc<AppState>,
    user_id: Uuid,
    submaterial_id: Uuid,
    req: SubmitMiniQuizRequest,
) -> Result<MiniQuizSubmitResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let sub = sub_repo::find_by_id(&state.db, submaterial_id)
        .await?
        .ok_or(LearningError::SubmaterialNotFound)?;

    let module = module_repo::find_by_id(&state.db, sub.module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    if module.role_id != role_id {
        return Err(LearningError::SubmaterialNotForUserRole);
    }

    assert_module_unlocked(state, user_id, role_id, module.order_index).await?;
    assert_submaterial_unlocked(state, user_id, sub.module_id, sub.order_index).await?;

    let questions = repository::get_questions(&state.db, submaterial_id).await?;
    if questions.is_empty() {
        return Err(LearningError::QuizNotFound);
    }

    // Validate answer count
    if req.answers.len() != questions.len() {
        return Err(LearningError::IncompleteAnswers);
    }

    let q_ids: Vec<Uuid> = questions.iter().map(|q| q.id).collect();
    let all_options = repository::get_options_for_questions(&state.db, &q_ids).await?;

    // Build lookup: question_id → options
    let mut opts_by_q: HashMap<Uuid, Vec<&_>> = HashMap::new();
    for opt in &all_options {
        opts_by_q.entry(opt.question_id).or_default().push(opt);
    }

    // Build lookup: answer question_id → option_id
    let answer_map: HashMap<Uuid, Uuid> = req
        .answers
        .iter()
        .map(|a| (a.question_id, a.option_id))
        .collect();

    // Validate all question_ids are for this submaterial
    for q in &questions {
        if !answer_map.contains_key(&q.id) {
            return Err(LearningError::IncompleteAnswers);
        }
    }

    // Score and build result
    let mut correct_count = 0usize;
    let mut result_questions = Vec::with_capacity(questions.len());

    for q in &questions {
        let chosen_option_id = answer_map[&q.id];
        let opts = opts_by_q.get(&q.id).map(|v| v.as_slice()).unwrap_or(&[]);

        // Validate chosen option belongs to this question
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
            .map(|o| MiniQuizResultOption {
                id:          o.id,
                text:        o.text.clone(),
                order_index: o.order_index,
                is_correct:  o.is_correct,
            })
            .collect();

        result_questions.push(MiniQuizResultQuestion {
            id:                q.id,
            question:          q.question.clone(),
            order_index:       q.order_index,
            your_answer_id:    chosen_option_id,
            is_correct:        chosen.is_correct,
            correct_option_id,
            options:           result_opts,
        });
    }

    let (score, passed) = score_mini_quiz(correct_count, questions.len());

    // Persist attempt + answers in transaction
    let mut tx = state.db.begin().await?;
    let attempt =
        repository::insert_attempt(&mut tx, user_id, submaterial_id, score, passed).await?;

    for q in &questions {
        let chosen_option_id = answer_map[&q.id];
        let opts = opts_by_q.get(&q.id).map(|v| v.as_slice()).unwrap_or(&[]);
        let is_correct = opts.iter().any(|o| o.id == chosen_option_id && o.is_correct);
        repository::insert_answer(&mut tx, attempt.id, q.id, chosen_option_id, is_correct).await?;
    }

    tx.commit().await?;

    // If passed → mark submaterial complete (idempotent)
    let submaterial_completed = if passed {
        progress_repo::upsert_submaterial_progress(&state.db, user_id, submaterial_id).await?;
        true
    } else {
        // Still check if previously completed
        let completed_ids =
            progress_repo::get_completed_submaterial_ids_for_module(&state.db, user_id, sub.module_id)
                .await?;
        completed_ids.contains(&submaterial_id)
    };

    Ok(MiniQuizSubmitResponse {
        attempt_id:            attempt.id,
        score,
        passed,
        submaterial_completed,
        questions:             result_questions,
    })
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

async fn assert_submaterial_unlocked(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
    target_order_index: i32,
) -> Result<(), LearningError> {
    let subs = sub_repo::get_submaterials_for_module(&state.db, module_id).await?;
    let completed_ids =
        progress_repo::get_completed_submaterial_ids_for_module(&state.db, user_id, module_id)
            .await?;

    let sub_infos: Vec<SubmaterialInfo> = subs
        .iter()
        .map(|s| SubmaterialInfo { submaterial_id: s.id, order_index: s.order_index })
        .collect();

    let gates = gating::compute_submaterial_gates(true, &sub_infos, &completed_ids);

    let is_unlocked = gates
        .iter()
        .zip(sub_infos.iter())
        .find(|(_, si)| si.order_index == target_order_index)
        .map(|(g, _)| g.is_unlocked)
        .unwrap_or(false);

    if !is_unlocked {
        return Err(LearningError::SubmaterialLocked);
    }
    Ok(())
}

use crate::features::learning::{
    module::repository::ModuleWithCountsRow, progress::repository::ModuleQuizStatusRow,
};
use std::collections::HashSet;

pub fn compute_completed_ids(
    rows: &[ModuleWithCountsRow],
    quiz_status: &HashMap<Uuid, ModuleQuizStatusRow>,
) -> HashSet<Uuid> {
    use crate::features::learning::module_quiz::scoring::FINAL_QUIZ_PASSING_SCORE;
    rows.iter()
        .filter(|r| {
            let all_subs_done =
                r.submaterials_total > 0 && r.submaterials_completed == r.submaterials_total;
            let quiz_passed = match quiz_status.get(&r.id) {
                None => true,
                Some(qs) => qs.total_questions == 0 || qs.best_score >= FINAL_QUIZ_PASSING_SCORE,
            };
            all_subs_done && quiz_passed
        })
        .map(|r| r.id)
        .collect()
}
