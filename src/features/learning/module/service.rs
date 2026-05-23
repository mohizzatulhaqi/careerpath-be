use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::learning::{
        error::LearningError,
        module::{
            dto::{ModuleDetailResponse, ModuleListItem, ModuleListResponse, SubmaterialInModule},
            repository,
        },
        module_quiz::scoring::FINAL_QUIZ_PASSING_SCORE,
        progress::{
            gating::{self, ModuleInfo, SubmaterialInfo},
            repository as progress_repo,
            service as progress_service,
        },
        submaterial::repository as sub_repo,
    },
    state::AppState,
};

/// GET /api/learning/modules
pub async fn get_modules_for_user(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<ModuleListResponse, LearningError> {
    let (role_id, role_info) = progress_service::resolve_user_role(state, user_id).await?;

    // Batch queries: modules + counts + final quiz status (no N+1)
    let rows = repository::get_modules_with_counts(&state.db, role_id, user_id).await?;
    let quiz_status =
        progress_repo::get_final_quiz_status_per_module(&state.db, role_id, user_id).await?;

    let module_infos: Vec<ModuleInfo> = rows
        .iter()
        .map(|r| ModuleInfo { module_id: r.id, order_index: r.order_index })
        .collect();

    // Module completed = all subs done AND final quiz passed
    let completed_module_ids: HashSet<Uuid> = rows
        .iter()
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
        .collect();

    let gates = gating::compute_module_gates(&module_infos, &completed_module_ids);

    let modules = rows
        .into_iter()
        .zip(gates.iter())
        .map(|(row, gate)| {
            let pct = if row.submaterials_total == 0 {
                0.0
            } else {
                (row.submaterials_completed as f64 / row.submaterials_total as f64 * 100.0).round()
            };
            let final_quiz_unlocked =
                row.submaterials_total > 0 && row.submaterials_completed == row.submaterials_total;
            let final_quiz_passed = quiz_status
                .get(&row.id)
                .map(|qs| qs.total_questions == 0 || qs.best_score >= FINAL_QUIZ_PASSING_SCORE)
                .unwrap_or(false);
            ModuleListItem {
                id: row.id,
                title: row.title,
                description: row.description,
                order_index: row.order_index,
                is_unlocked: gate.is_unlocked,
                is_completed: gate.is_completed,
                completion_percentage: pct,
                submaterials_total: row.submaterials_total,
                submaterials_completed: row.submaterials_completed,
                final_quiz_unlocked,
                final_quiz_passed,
            }
        })
        .collect();

    Ok(ModuleListResponse {
        role: role_info,
        modules,
    })
}

/// GET /api/learning/modules/:id
pub async fn get_module_detail(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
) -> Result<ModuleDetailResponse, LearningError> {
    let (role_id, _role_info) = progress_service::resolve_user_role(state, user_id).await?;

    let module = repository::find_by_id(&state.db, module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    // Validate role
    if module.role_id != role_id {
        return Err(LearningError::ModuleNotForUserRole);
    }

    // Check module unlock
    assert_module_unlocked(state, user_id, role_id, module.order_index).await?;

    // Fetch submaterials
    let subs = sub_repo::get_submaterials_for_module(&state.db, module_id).await?;
    let completed_ids = progress_repo::get_completed_submaterial_ids_for_module(
        &state.db, user_id, module_id,
    )
    .await?;

    let sub_infos: Vec<SubmaterialInfo> = subs
        .iter()
        .map(|s| SubmaterialInfo {
            submaterial_id: s.id,
            order_index: s.order_index,
        })
        .collect();

    let sub_gates = gating::compute_submaterial_gates(true, &sub_infos, &completed_ids);

    // Fetch completion timestamps
    let progress_rows =
        progress_repo::get_progress_for_module(&state.db, user_id, module_id).await?;

    let submaterials: Vec<SubmaterialInModule> = subs
        .into_iter()
        .zip(sub_gates.iter())
        .map(|(s, gate)| {
            let completed_at = progress_rows
                .iter()
                .find(|p| p.submaterial_id == s.id)
                .map(|p| p.completed_at);
            SubmaterialInModule {
                id: s.id,
                title: s.title,
                order_index: s.order_index,
                estimated_minutes: s.estimated_minutes,
                is_unlocked: gate.is_unlocked,
                is_completed: gate.is_completed,
                completed_at,
            }
        })
        .collect();

    let total = submaterials.len() as f64;
    let done = submaterials.iter().filter(|s| s.is_completed).count() as f64;
    let pct = if total == 0.0 { 0.0 } else { (done / total * 100.0).round() };
    let final_quiz_unlocked = done == total && total > 0.0;

    let quiz_status =
        progress_repo::get_final_quiz_status_per_module(&state.db, role_id, user_id).await?;
    let final_quiz_passed = quiz_status
        .get(&module.id)
        .map(|qs| qs.total_questions == 0 || qs.best_score >= FINAL_QUIZ_PASSING_SCORE)
        .unwrap_or(false);

    Ok(ModuleDetailResponse {
        id: module.id,
        title: module.title,
        description: module.description,
        order_index: module.order_index,
        is_unlocked: true,
        is_completed: final_quiz_unlocked && final_quiz_passed,
        completion_percentage: pct,
        final_quiz_unlocked,
        final_quiz_passed,
        submaterials,
    })
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Asserts the module at the given order_index is unlocked for the user.
async fn assert_module_unlocked(
    state: &Arc<AppState>,
    user_id: Uuid,
    role_id: Uuid,
    target_order_index: i32,
) -> Result<(), LearningError> {
    let rows = repository::get_modules_with_counts(&state.db, role_id, user_id).await?;
    let quiz_status =
        progress_repo::get_final_quiz_status_per_module(&state.db, role_id, user_id).await?;

    let module_infos: Vec<ModuleInfo> = rows
        .iter()
        .map(|r| ModuleInfo { module_id: r.id, order_index: r.order_index })
        .collect();

    let completed_ids: HashSet<Uuid> = rows
        .iter()
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
        .collect();

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
