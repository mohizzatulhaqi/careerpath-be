use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::learning::{
        error::LearningError,
        module::repository as module_repo,
        module_quiz::scoring::FINAL_QUIZ_PASSING_SCORE,
        progress::{
            gating::{self, ModuleInfo, SubmaterialInfo},
            repository as progress_repo,
            service as progress_service,
        },
        submaterial::{
            dto::{
                CompleteModuleStatus, CompleteSubmaterialResponse, NextModuleUnlocked,
                SubmaterialDetailResponse, SubmaterialModuleInfo,
            },
            repository,
        },
    },
    state::AppState,
};

/// GET /api/learning/submaterials/:id
pub async fn get_submaterial(
    state: &Arc<AppState>,
    user_id: Uuid,
    submaterial_id: Uuid,
) -> Result<SubmaterialDetailResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let sub = repository::find_by_id(&state.db, submaterial_id)
        .await?
        .ok_or(LearningError::SubmaterialNotFound)?;

    let module = module_repo::find_by_id(&state.db, sub.module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    // Validate role
    if module.role_id != role_id {
        return Err(LearningError::SubmaterialNotForUserRole);
    }

    // Assert module unlocked
    assert_module_unlocked_for_sub(state, user_id, role_id, module.order_index).await?;

    // Assert submaterial unlocked
    assert_submaterial_unlocked(state, user_id, sub.module_id, sub.order_index).await?;

    // Check completion
    let completed_ids =
        progress_repo::get_completed_submaterial_ids_for_module(&state.db, user_id, sub.module_id)
            .await?;

    Ok(SubmaterialDetailResponse {
        id: sub.id,
        title: sub.title,
        content: sub.content,
        order_index: sub.order_index,
        estimated_minutes: sub.estimated_minutes,
        is_completed: completed_ids.contains(&sub.id),
        module: SubmaterialModuleInfo {
            id: module.id,
            title: module.title,
            order_index: module.order_index,
        },
    })
}

/// POST /api/learning/submaterials/:id/complete
pub async fn mark_complete(
    state: &Arc<AppState>,
    user_id: Uuid,
    submaterial_id: Uuid,
) -> Result<CompleteSubmaterialResponse, LearningError> {
    let (role_id, _) = progress_service::resolve_user_role(state, user_id).await?;

    let sub = repository::find_by_id(&state.db, submaterial_id)
        .await?
        .ok_or(LearningError::SubmaterialNotFound)?;

    let module = module_repo::find_by_id(&state.db, sub.module_id)
        .await?
        .ok_or(LearningError::ModuleNotFound)?;

    // Validate role
    if module.role_id != role_id {
        return Err(LearningError::SubmaterialNotForUserRole);
    }

    // Assert module + submaterial unlocked
    assert_module_unlocked_for_sub(state, user_id, role_id, module.order_index).await?;
    assert_submaterial_unlocked(state, user_id, sub.module_id, sub.order_index).await?;

    // Upsert progress (idempotent)
    let progress =
        progress_repo::upsert_submaterial_progress(&state.db, user_id, submaterial_id).await?;

    // Calculate module completion
    let all_subs = repository::get_submaterials_for_module(&state.db, sub.module_id).await?;
    let completed_ids =
        progress_repo::get_completed_submaterial_ids_for_module(&state.db, user_id, sub.module_id)
            .await?;

    let total = all_subs.len() as f64;
    let done = completed_ids.len() as f64;
    let pct = if total == 0.0 {
        0.0
    } else {
        (done / total * 100.0).round()
    };
    let module_completed = done == total && total > 0.0;

    // Find next submaterial
    let next_sub = repository::find_next_submaterial(&state.db, sub.module_id, sub.order_index)
        .await?;
    let next_submaterial_id = next_sub.map(|ns| ns.id);

    // Check if module just completed → unlock next module
    let next_module_unlocked = if module_completed {
        let modules = module_repo::get_modules_for_role(&state.db, role_id).await?;
        modules
            .iter()
            .find(|m| m.order_index == module.order_index + 1)
            .map(|m| NextModuleUnlocked {
                id: m.id,
                title: m.title.clone(),
            })
    } else {
        None
    };

    Ok(CompleteSubmaterialResponse {
        submaterial_id: sub.id,
        completed_at: progress.completed_at,
        module: CompleteModuleStatus {
            id: module.id,
            completion_percentage: pct,
            is_completed: module_completed,
            next_submaterial_id,
        },
        next_module_unlocked,
    })
}

// ── Private helpers ───────────────────────────────────────────────────────────

async fn assert_module_unlocked_for_sub(
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

async fn assert_submaterial_unlocked(
    state: &Arc<AppState>,
    user_id: Uuid,
    module_id: Uuid,
    target_order_index: i32,
) -> Result<(), LearningError> {
    let subs = repository::get_submaterials_for_module(&state.db, module_id).await?;
    let completed_ids =
        progress_repo::get_completed_submaterial_ids_for_module(&state.db, user_id, module_id)
            .await?;

    let sub_infos: Vec<SubmaterialInfo> = subs
        .iter()
        .map(|s| SubmaterialInfo {
            submaterial_id: s.id,
            order_index: s.order_index,
        })
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
