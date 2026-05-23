use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::learning::{
        error::LearningError,
        module::{dto::RoleInfo, repository as module_repo},
        module_quiz::scoring::FINAL_QUIZ_PASSING_SCORE,
        progress::{
            dto::{CurrentModuleInfo, OverallProgressResponse, ProgressModuleItem},
            gating::{self, ModuleInfo},
            repository as progress_repo,
        },
    },
    state::AppState,
};

/// Resolve the user's role from their latest submitted quiz attempt.
/// Returns (role_id, RoleInfo) or LearningError::QuizNotCompleted.
pub async fn resolve_user_role(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<(Uuid, RoleInfo), LearningError> {
    #[derive(Debug, sqlx::FromRow)]
    struct RoleRow {
        role_id: Uuid,
        code: String,
        name: String,
    }

    let row = sqlx::query_as::<_, RoleRow>(
        r#"
        SELECT r.id AS role_id, r.code, r.name
        FROM   quiz_attempts qa
        JOIN   roles r ON r.id = qa.result_role_id
        WHERE  qa.user_id = $1 AND qa.status = 'submitted'
        ORDER  BY qa.submitted_at DESC
        LIMIT  1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(LearningError::QuizNotCompleted)?;

    Ok((
        row.role_id,
        RoleInfo {
            id: row.role_id,
            code: row.code,
            name: row.name,
        },
    ))
}

/// GET /api/learning/progress
pub async fn get_overall_progress(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<OverallProgressResponse, LearningError> {
    let (role_id, role_info) = resolve_user_role(state, user_id).await?;

    // Batch query: modules + counts + final quiz status
    let rows = module_repo::get_modules_with_counts(&state.db, role_id, user_id).await?;
    let quiz_status =
        progress_repo::get_final_quiz_status_per_module(&state.db, role_id, user_id).await?;

    let module_infos: Vec<ModuleInfo> = rows
        .iter()
        .map(|r| ModuleInfo { module_id: r.id, order_index: r.order_index })
        .collect();

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

    let total_submaterials: i64 = rows.iter().map(|r| r.submaterials_total).sum();
    let completed_submaterials: i64 = rows.iter().map(|r| r.submaterials_completed).sum();
    let total_modules = rows.len() as i64;
    let completed_modules = completed_module_ids.len() as i64;

    let overall_percentage = if total_submaterials == 0 {
        0.0
    } else {
        (completed_submaterials as f64 / total_submaterials as f64 * 100.0).round()
    };

    // Find current module: first unlocked + not completed
    let current_module = rows
        .iter()
        .zip(gates.iter())
        .find(|(_, g)| g.is_unlocked && !g.is_completed)
        .map(|(row, _)| {
            let pct = if row.submaterials_total == 0 {
                0.0
            } else {
                (row.submaterials_completed as f64 / row.submaterials_total as f64 * 100.0).round()
            };
            CurrentModuleInfo {
                id: row.id,
                title: row.title.clone(),
                completion_percentage: pct,
            }
        });

    let modules: Vec<ProgressModuleItem> = rows
        .into_iter()
        .zip(gates.iter())
        .map(|(row, gate)| {
            let pct = if row.submaterials_total == 0 {
                0.0
            } else {
                (row.submaterials_completed as f64 / row.submaterials_total as f64 * 100.0).round()
            };
            ProgressModuleItem {
                id: row.id,
                title: row.title,
                order_index: row.order_index,
                is_unlocked: gate.is_unlocked,
                is_completed: gate.is_completed,
                completion_percentage: pct,
            }
        })
        .collect();

    Ok(OverallProgressResponse {
        role: role_info,
        overall_percentage,
        total_modules,
        completed_modules,
        total_submaterials,
        completed_submaterials,
        current_module,
        modules,
    })
}
