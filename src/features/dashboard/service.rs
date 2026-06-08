use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

use crate::features::{
    certificate::repository as cert_repo,
    dashboard::dto::{
        ActivityDto, ActivityLogResponse, CareerPathSummaryDto, CertificateBriefDto,
        CurrentModuleDto, CurrentModuleInfo, CurrentSubmaterialDto, DashboardResponse,
        FinalProjectInfo, FinalProjectSummaryDto, LearningSummaryModuleDto, LearningSummaryResponse,
        LearningProgressSummaryDto, NextActionDto, ProjectBriefDto, RoleDto, UserSummaryDto,
    },
    learning::{
        module::repository as module_repo,
        module_quiz::scoring::FINAL_QUIZ_PASSING_SCORE,
        progress::{
            gating::{self, ModuleInfo},
            repository as progress_repo,
        },
    },
    project::{
        gating::{compute_project_access, ModuleCompletionSummary},
    },
};

// ── Pure function: determine_next_action ──────────────────────────────────

/// Pure function — no DB access. Fully unit-testable.
pub fn determine_next_action(
    has_taken_quiz: bool,
    current_module: Option<&CurrentModuleInfo>,
    final_project: Option<&FinalProjectInfo>,
) -> NextActionDto {
    // 1. No quiz yet → ask user to take it
    if !has_taken_quiz {
        return NextActionDto {
            code: "TAKE_QUIZ".into(),
            title: "Temukan Career Path kamu".into(),
            description: "Jawab kuis singkat untuk mengetahui role yang paling cocok untukmu".into(),
            target_url: "/quiz".into(),
        };
    }

    // 2. There is a current (in-progress) module
    if let Some(module) = current_module {
        // 2a. Final quiz pending (all subs done, quiz not passed yet)
        if module.all_subs_done_quiz_pending {
            return NextActionDto {
                code: "TAKE_FINAL_QUIZ".into(),
                title: format!("Kerjakan Final Quiz: {}", module.title),
                description: "Semua sub materi selesai — sekarang saatnya final quiz!".into(),
                target_url: format!("/learning/modules/{}/final-quiz", module.id),
            };
        }

        // 2b. Sub materi belum selesai
        if let Some(sub) = &module.current_submaterial {
            return NextActionDto {
                code: "CONTINUE_LEARNING".into(),
                title: format!("Lanjutkan belajar {}", module.title),
                description: format!("Sub materi: {}", sub.title),
                target_url: format!("/learning/submaterials/{}", sub.id),
            };
        }

        // Defensive fallback if current_submaterial is None but module not all_subs_done
        return NextActionDto {
            code: "CONTINUE_LEARNING".into(),
            title: format!("Lanjutkan belajar {}", module.title),
            description: "Buka modul untuk melanjutkan".into(),
            target_url: format!("/learning/modules/{}", module.id),
        };
    }

    // 3. All modules completed → check project
    match final_project {
        None => NextActionDto {
            code: "ALL_DONE".into(),
            title: "Semua modul selesai!".into(),
            description: "Selamat! Kamu telah menyelesaikan semua materi pembelajaran.".into(),
            target_url: "/dashboard".into(),
        },
        Some(project) => match project.access_status.as_str() {
            "available" => NextActionDto {
                code: "SUBMIT_PROJECT".into(),
                title: format!("Submit Final Project: {}", project.title),
                description: "Semua modul selesai — upload file ZIP project kamu sekarang!".into(),
                target_url: format!("/projects/{}", project.id),
            },
            "rejected" => NextActionDto {
                code: "SUBMIT_PROJECT".into(),
                title: format!("Resubmit Final Project: {}", project.title),
                description: "Submission sebelumnya ditolak — perbaiki dan upload ulang project kamu.".into(),
                target_url: format!("/projects/{}", project.id),
            },
            "pending_review" => NextActionDto {
                code: "WAIT_REVIEW".into(),
                title: "Menunggu Review Final Project".into(),
                description: "Submission kamu sedang ditinjau oleh admin. Mohon bersabar.".into(),
                target_url: format!("/projects/{}", project.id),
            },
            "approved" => NextActionDto {
                code: "ALL_DONE".into(),
                title: "Selamat, kamu telah lulus!".into(),
                description: format!("Final project \"{}\" kamu telah disetujui. Luar biasa!", project.title),
                target_url: "/dashboard".into(),
            },
            _ => NextActionDto {
                code: "ALL_DONE".into(),
                title: "Semua modul selesai!".into(),
                description: "Selamat telah menyelesaikan semua materi!".into(),
                target_url: "/dashboard".into(),
            },
        },
    }
}

// ── Queries private to dashboard ───────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct CareerPathRow {
    attempt_id: Uuid,
    role_id: Uuid,
    role_code: String,
    role_name: String,
    role_description: Option<String>,
    match_percentage: Option<f64>,
    submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct ActivityRow {
    kind: String,
    title: String,
    detail: String,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct CurrentSubRow {
    id: Uuid,
    title: String,
    order_index: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct ProjectSubmissionRow {
    project_id: Uuid,
    project_title: String,
    access_status: String,
    latest_submission_at: Option<DateTime<Utc>>,
}

async fn fetch_user(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<UserRow>> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, name, email, created_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

async fn fetch_career_path(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Option<CareerPathRow>> {
    sqlx::query_as::<_, CareerPathRow>(
        r#"
        SELECT qa.id AS attempt_id,
               r.id AS role_id,
               r.code AS role_code,
               r.name AS role_name,
               r.description AS role_description,
               qa.match_percentage,
               qa.submitted_at
        FROM quiz_attempts qa
        JOIN roles r ON r.id = qa.result_role_id
        WHERE qa.user_id = $1 AND qa.status = 'submitted'
        ORDER BY qa.submitted_at DESC NULLS LAST
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Fetch the first uncompleted submaterial (by order_index) in a module.
async fn fetch_current_submaterial(
    pool: &PgPool,
    user_id: Uuid,
    module_id: Uuid,
) -> sqlx::Result<Option<CurrentSubRow>> {
    sqlx::query_as::<_, CurrentSubRow>(
        r#"
        SELECT s.id, s.title, s.order_index
        FROM submaterials s
        LEFT JOIN user_submaterial_progress usp
               ON usp.submaterial_id = s.id AND usp.user_id = $1
        WHERE s.module_id = $2 AND usp.id IS NULL
        ORDER BY s.order_index
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(module_id)
    .fetch_optional(pool)
    .await
}

async fn fetch_project_summary(
    pool: &PgPool,
    role_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<ProjectSubmissionRow>> {
    // Single query: get project for role + latest submission status
    sqlx::query_as::<_, ProjectSubmissionRow>(
        r#"
        SELECT p.id AS project_id,
               p.title AS project_title,
               COALESCE(latest.status, 'none') AS access_status,
               latest.submitted_at AS latest_submission_at
        FROM projects p
        LEFT JOIN LATERAL (
            SELECT ps.status, ps.submitted_at
            FROM project_submissions ps
            WHERE ps.project_id = p.id AND ps.user_id = $2
            ORDER BY ps.submitted_at DESC
            LIMIT 1
        ) latest ON true
        WHERE p.role_id = $1 AND p.is_published = true
        LIMIT 1
        "#,
    )
    .bind(role_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

async fn fetch_recent_activities(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> sqlx::Result<Vec<ActivityRow>> {
    // UNION ALL with inner LIMIT per source for efficiency
    sqlx::query_as::<_, ActivityRow>(
        r#"
        (
          SELECT 'submaterial_completed'::text AS kind,
                 s.title AS title,
                 'Modul: ' || m.title AS detail,
                 usp.completed_at AS occurred_at
          FROM user_submaterial_progress usp
          JOIN submaterials s ON s.id = usp.submaterial_id
          JOIN learning_modules m ON m.id = s.module_id
          WHERE usp.user_id = $1
          ORDER BY usp.completed_at DESC
          LIMIT 10
        )
        UNION ALL
        (
          SELECT CASE WHEN mqa.passed THEN 'final_quiz_passed' ELSE 'final_quiz_failed' END AS kind,
                 CASE WHEN mqa.passed
                      THEN 'Lulus Final Quiz: ' || lm.title
                      ELSE 'Final Quiz: ' || lm.title END AS title,
                 'Score: ' || ROUND(mqa.score::numeric, 1)::text AS detail,
                 mqa.created_at AS occurred_at
          FROM module_quiz_attempts mqa
          JOIN learning_modules lm ON lm.id = mqa.module_id
          WHERE mqa.user_id = $1
          ORDER BY mqa.created_at DESC
          LIMIT 10
        )
        UNION ALL
        (
          SELECT 'project_submitted'::text AS kind,
                 'Submit: ' || p.title AS title,
                 'Status: ' || ps.status AS detail,
                 ps.submitted_at AS occurred_at
          FROM project_submissions ps
          JOIN projects p ON p.id = ps.project_id
          WHERE ps.user_id = $1
          ORDER BY ps.submitted_at DESC
          LIMIT 10
        )
        ORDER BY occurred_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

// ── Learning progress builder ──────────────────────────────────────────────

/// Build LearningProgressSummaryDto + CurrentModuleInfo (for next_action).
/// Returns (progress_dto, current_module_info, completed_module_count).
async fn build_learning_progress(
    pool: &PgPool,
    role_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<(LearningProgressSummaryDto, Option<CurrentModuleInfo>, i64)> {
    let rows = module_repo::get_modules_with_counts(pool, role_id, user_id).await?;
    if rows.is_empty() {
        return Ok((LearningProgressSummaryDto::default(), None, 0));
    }

    let quiz_status = progress_repo::get_final_quiz_status_per_module(pool, role_id, user_id).await?;

    let module_infos: Vec<ModuleInfo> = rows
        .iter()
        .map(|r| ModuleInfo { module_id: r.id, order_index: r.order_index })
        .collect();

    let completed_ids: HashSet<Uuid> = rows.iter().filter(|r| {
        let all_subs = r.submaterials_total > 0 && r.submaterials_completed == r.submaterials_total;
        let quiz_ok = match quiz_status.get(&r.id) {
            None => true,
            Some(qs) => qs.total_questions == 0 || qs.best_score >= FINAL_QUIZ_PASSING_SCORE,
        };
        all_subs && quiz_ok
    }).map(|r| r.id).collect();

    let gates = gating::compute_module_gates(&module_infos, &completed_ids);

    let total_submaterials: i64 = rows.iter().map(|r| r.submaterials_total).sum();
    let completed_submaterials: i64 = rows.iter().map(|r| r.submaterials_completed).sum();
    let total_modules = rows.len() as i64;
    let completed_modules = completed_ids.len() as i64;

    let overall_pct = if total_submaterials == 0 {
        0.0
    } else {
        (completed_submaterials as f64 / total_submaterials as f64 * 100.0).round()
    };

    // Find the current module: first that is unlocked & not completed
    let current_module_dto;
    let current_module_info;
    if let Some((row, _gate)) = rows.iter().zip(gates.iter()).find(|(r, g)| {
        g.is_unlocked && !completed_ids.contains(&r.id)
    }) {
        let sub_pct = if row.submaterials_total == 0 {
            0.0
        } else {
            (row.submaterials_completed as f64 / row.submaterials_total as f64 * 100.0).round()
        };

        let all_subs_done = row.submaterials_total > 0
            && row.submaterials_completed == row.submaterials_total;

        // If all subs done but module not completed → final quiz pending
        let all_subs_quiz_pending = all_subs_done;

        // Fetch current submaterial only if subs are not all done
        let current_sub = if !all_subs_done {
            fetch_current_submaterial(pool, user_id, row.id).await?.map(|s| CurrentSubmaterialDto {
                id: s.id,
                title: s.title,
                order_index: s.order_index,
            })
        } else {
            None
        };

        current_module_dto = Some(CurrentModuleDto {
            id: row.id,
            title: row.title.clone(),
            order_index: row.order_index,
            completion_percentage: sub_pct,
            current_submaterial: current_sub.clone(),
        });

        current_module_info = Some(CurrentModuleInfo {
            id: row.id,
            title: row.title.clone(),
            order_index: row.order_index,
            completion_percentage: sub_pct,
            all_subs_done_quiz_pending: all_subs_quiz_pending,
            current_submaterial: current_sub,
        });
    } else {
        current_module_dto = None;
        current_module_info = None;
    }

    let progress_dto = LearningProgressSummaryDto {
        overall_percentage: overall_pct,
        total_modules,
        completed_modules,
        total_submaterials,
        completed_submaterials,
        current_module: current_module_dto,
    };

    Ok((progress_dto, current_module_info, completed_modules))
}

// ── Public service functions ───────────────────────────────────────────────

/// GET /api/dashboard — full dashboard response.
pub async fn get_full_dashboard(
    pool: &PgPool,
    user_id: Uuid,
    app_base_url: &str,
) -> Result<DashboardResponse, crate::error::AppError> {
    // Parallel: user info + career path + activities + certificates
    let (user_res, career_res, activities_res, certs_res) = tokio::join!(
        fetch_user(pool, user_id),
        fetch_career_path(pool, user_id),
        fetch_recent_activities(pool, user_id, 5),
        cert_repo::list_for_user_brief(pool, user_id),
    );

    let user_row = user_res?
        .ok_or_else(|| crate::error::AppError::NotFound("user not found".into()))?;

    let career_row = career_res?;
    let activities = activities_res?;
    let cert_rows = certs_res.unwrap_or_default();

    // Build career_path section
    let has_taken_quiz = career_row.is_some();

    // User belum prequiz — return early dengan data minimal
    if !has_taken_quiz {
        return Ok(DashboardResponse {
            prequiz_required: true,
            prequiz_message: Some(
                "Silahkan selesaikan prequiz terlebih dahulu untuk mengakses dashboard belajar."
                    .into(),
            ),
            user: UserSummaryDto {
                id: user_row.id,
                name: user_row.name,
                email: user_row.email,
                member_since: user_row.created_at,
            },
            career_path: CareerPathSummaryDto {
                has_taken_quiz: false,
                role: None,
                match_percentage: None,
                quiz_taken_at: None,
                last_quiz_attempt_id: None,
            },
            learning_progress: LearningProgressSummaryDto::default(),
            final_project: FinalProjectSummaryDto::default(),
            next_action: NextActionDto {
                code: "TAKE_QUIZ".into(),
                title: "Temukan Career Path kamu".into(),
                description:
                    "Jawab kuis singkat untuk mengetahui role yang paling cocok untukmu".into(),
                target_url: "/quiz".into(),
            },
            recent_activities: vec![],
            certificates: vec![],
        });
    }

    let career_path = match &career_row {
        None => CareerPathSummaryDto {
            has_taken_quiz: false,
            role: None,
            match_percentage: None,
            quiz_taken_at: None,
            last_quiz_attempt_id: None,
        },
        Some(cp) => CareerPathSummaryDto {
            has_taken_quiz: true,
            role: Some(RoleDto {
                id: cp.role_id,
                code: cp.role_code.clone(),
                name: cp.role_name.clone(),
                description: cp.role_description.clone(),
            }),
            match_percentage: cp.match_percentage,
            quiz_taken_at: cp.submitted_at,
            last_quiz_attempt_id: Some(cp.attempt_id),
        },
    };

    // Build learning + project sections (requires role)
    let learning_progress;
    let final_project;
    let current_module_info;
    let final_project_info;

    if let Some(cp) = &career_row {
        let role_id = cp.role_id;

        // Parallel: learning progress + project summary
        let (learning_res, project_res) = tokio::join!(
            build_learning_progress(pool, role_id, user_id),
            fetch_project_summary(pool, role_id, user_id),
        );

        let (progress_dto, cm_info, completed_modules_count) = learning_res?;
        let project_row = project_res?;

        learning_progress = progress_dto;
        current_module_info = cm_info;

        // Build final_project section
        let all_modules_done = learning_progress.total_modules > 0
            && completed_modules_count >= learning_progress.total_modules;

        match project_row {
            None => {
                final_project = FinalProjectSummaryDto::default();
                final_project_info = None;
            }
            Some(pr) => {
                // Compute real access_status using gating logic
                let summary = ModuleCompletionSummary {
                    total: learning_progress.total_modules,
                    completed: completed_modules_count,
                };
                let latest_status_opt = if pr.access_status == "none" {
                    None
                } else {
                    Some(pr.access_status.as_str())
                };
                let access = compute_project_access(&summary, latest_status_opt);
                let access_str = access.as_str().to_string();

                final_project_info = Some(FinalProjectInfo {
                    id: pr.project_id,
                    title: pr.project_title.clone(),
                    access_status: access_str.clone(),
                    latest_submission_at: pr.latest_submission_at,
                });

                final_project = FinalProjectSummaryDto {
                    is_available: all_modules_done,
                    project: Some(ProjectBriefDto { id: pr.project_id, title: pr.project_title }),
                    access_status: Some(access_str),
                    latest_submission_at: pr.latest_submission_at,
                };
            }
        }
    } else {
        learning_progress = LearningProgressSummaryDto::default();
        final_project = FinalProjectSummaryDto::default();
        current_module_info = None;
        final_project_info = None;
    }

    let next_action =
        determine_next_action(has_taken_quiz, current_module_info.as_ref(), final_project_info.as_ref());

    let recent_activities = activities
        .into_iter()
        .map(|a| ActivityDto { kind: a.kind, title: a.title, detail: a.detail, occurred_at: a.occurred_at })
        .collect();

    let certificates = cert_rows
        .into_iter()
        .map(|(id, code, role_name, issued_at, is_revoked)| CertificateBriefDto {
            id,
            verification_url: format!("{}/verify/{}", app_base_url, code),
            certificate_code: code,
            role_name,
            issued_at,
            is_revoked,
        })
        .collect();

    Ok(DashboardResponse {
        prequiz_required: false,
        prequiz_message: None,
        user: UserSummaryDto {
            id: user_row.id,
            name: user_row.name,
            email: user_row.email,
            member_since: user_row.created_at,
        },
        career_path,
        learning_progress,
        final_project,
        next_action,
        recent_activities,
        certificates,
    })
}

/// GET /api/dashboard/learning-summary — delegates to module service.
pub async fn get_learning_summary(
    pool: &PgPool,
    user_id: Uuid,
    role_id: Uuid,
    role_code: String,
    role_name: String,
    role_description: Option<String>,
) -> Result<LearningSummaryResponse, crate::error::AppError> {
    let rows = module_repo::get_modules_with_counts(pool, role_id, user_id).await?;
    let quiz_status = progress_repo::get_final_quiz_status_per_module(pool, role_id, user_id).await?;

    let module_infos: Vec<ModuleInfo> = rows.iter()
        .map(|r| ModuleInfo { module_id: r.id, order_index: r.order_index })
        .collect();

    let completed_ids: HashSet<Uuid> = rows.iter().filter(|r| {
        let all_subs = r.submaterials_total > 0 && r.submaterials_completed == r.submaterials_total;
        let quiz_ok = match quiz_status.get(&r.id) {
            None => true,
            Some(qs) => qs.total_questions == 0 || qs.best_score >= FINAL_QUIZ_PASSING_SCORE,
        };
        all_subs && quiz_ok
    }).map(|r| r.id).collect();

    let gates = gating::compute_module_gates(&module_infos, &completed_ids);

    let total_sub: i64 = rows.iter().map(|r| r.submaterials_total).sum();
    let done_sub: i64 = rows.iter().map(|r| r.submaterials_completed).sum();
    let overall_pct = if total_sub == 0 {
        0.0
    } else {
        (done_sub as f64 / total_sub as f64 * 100.0).round()
    };

    let modules = rows.into_iter().zip(gates.iter()).map(|(row, gate)| {
        let pct = if row.submaterials_total == 0 {
            0.0
        } else {
            (row.submaterials_completed as f64 / row.submaterials_total as f64 * 100.0).round()
        };
        let quiz_passed = match quiz_status.get(&row.id) {
            None => false,
            Some(qs) => qs.total_questions == 0 || qs.best_score >= FINAL_QUIZ_PASSING_SCORE,
        };
        let quiz_unlocked = row.submaterials_total > 0
            && row.submaterials_completed == row.submaterials_total;

        LearningSummaryModuleDto {
            id: row.id,
            title: row.title,
            order_index: row.order_index,
            is_unlocked: gate.is_unlocked,
            is_completed: gate.is_completed,
            completion_percentage: pct,
            submaterials_total: row.submaterials_total,
            submaterials_completed: row.submaterials_completed,
            final_quiz_passed: quiz_passed,
            final_quiz_unlocked: quiz_unlocked,
        }
    }).collect();

    Ok(LearningSummaryResponse {
        role: RoleDto { id: role_id, code: role_code, name: role_name, description: role_description },
        overall_percentage: overall_pct,
        modules,
    })
}

/// GET /api/dashboard/activity?limit=N
pub async fn get_activity_log(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<ActivityLogResponse, crate::error::AppError> {
    // Fetch limit+1 to detect has_more
    let rows = fetch_recent_activities(pool, user_id, limit + 1).await?;
    let has_more = rows.len() as i64 > limit;
    let activities = rows
        .into_iter()
        .take(limit as usize)
        .map(|a| ActivityDto { kind: a.kind, title: a.title, detail: a.detail, occurred_at: a.occurred_at })
        .collect();
    Ok(ActivityLogResponse { activities, has_more })
}

// ── Unit tests for determine_next_action ──────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sub(title: &str) -> CurrentSubmaterialDto {
        CurrentSubmaterialDto { id: Uuid::new_v4(), title: title.into(), order_index: 1 }
    }

    fn module_with_sub(title: &str, sub_title: &str) -> CurrentModuleInfo {
        CurrentModuleInfo {
            id: Uuid::new_v4(),
            title: title.into(),
            order_index: 1,
            completion_percentage: 33.0,
            all_subs_done_quiz_pending: false,
            current_submaterial: Some(sub(sub_title)),
        }
    }

    fn module_quiz_pending(title: &str) -> CurrentModuleInfo {
        CurrentModuleInfo {
            id: Uuid::new_v4(),
            title: title.into(),
            order_index: 1,
            completion_percentage: 100.0,
            all_subs_done_quiz_pending: true,
            current_submaterial: None,
        }
    }

    fn project(title: &str, status: &str) -> FinalProjectInfo {
        FinalProjectInfo {
            id: Uuid::new_v4(),
            title: title.into(),
            access_status: status.into(),
            latest_submission_at: None,
        }
    }

    // Test 1: quiz not taken → TAKE_QUIZ
    #[test]
    fn take_quiz_when_no_quiz() {
        let action = determine_next_action(false, None, None);
        assert_eq!(action.code, "TAKE_QUIZ");
        assert_eq!(action.target_url, "/quiz");
    }

    // Test 2: has quiz, current module with current submaterial → CONTINUE_LEARNING, target = sub URL
    #[test]
    fn continue_learning_with_current_submaterial() {
        let m = module_with_sub("JavaScript Fundamental", "Function dan Scope");
        let action = determine_next_action(true, Some(&m), None);
        assert_eq!(action.code, "CONTINUE_LEARNING");
        assert!(action.title.contains("JavaScript Fundamental"));
        assert!(action.description.contains("Function dan Scope"));
        assert!(action.target_url.starts_with("/learning/submaterials/"));
    }

    // Test 3: all subs done, final quiz pending → TAKE_FINAL_QUIZ
    #[test]
    fn take_final_quiz_when_all_subs_done() {
        let m = module_quiz_pending("React Advanced");
        let action = determine_next_action(true, Some(&m), None);
        assert_eq!(action.code, "TAKE_FINAL_QUIZ");
        assert!(action.title.contains("React Advanced"));
        assert!(action.target_url.contains("/final-quiz"));
    }

    // Test 4: no current module, project available → SUBMIT_PROJECT
    #[test]
    fn submit_project_when_available() {
        let p = project("Frontend Final Project", "available");
        let action = determine_next_action(true, None, Some(&p));
        assert_eq!(action.code, "SUBMIT_PROJECT");
        assert!(action.description.contains("upload"));
    }

    // Test 5: no current module, project rejected → SUBMIT_PROJECT with resubmit context
    #[test]
    fn submit_project_when_rejected() {
        let p = project("Frontend Final Project", "rejected");
        let action = determine_next_action(true, None, Some(&p));
        assert_eq!(action.code, "SUBMIT_PROJECT");
        assert!(action.description.contains("ditolak") || action.title.contains("Resubmit"));
    }

    // Test 6: no current module, project pending_review → WAIT_REVIEW
    #[test]
    fn wait_review_when_pending() {
        let p = project("Frontend Final Project", "pending_review");
        let action = determine_next_action(true, None, Some(&p));
        assert_eq!(action.code, "WAIT_REVIEW");
    }

    // Test 7: no current module, project approved → ALL_DONE
    #[test]
    fn all_done_when_approved() {
        let p = project("Frontend Final Project", "approved");
        let action = determine_next_action(true, None, Some(&p));
        assert_eq!(action.code, "ALL_DONE");
    }

    // Test 8: no current module, no project seeded → ALL_DONE fallback
    #[test]
    fn all_done_no_project_seeded() {
        let action = determine_next_action(true, None, None);
        assert_eq!(action.code, "ALL_DONE");
        assert!(action.description.contains("selesai") || action.description.contains("Selamat"));
    }
}
