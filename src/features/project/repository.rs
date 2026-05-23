use super::entity::{Project, ProjectSubmission};
use crate::features::project::gating::ModuleCompletionSummary;
use sqlx::PgPool;
use uuid::Uuid;

/// User-facing: only published projects.
pub async fn get_project_by_role(pool: &PgPool, role_id: Uuid) -> sqlx::Result<Option<Project>> {
    sqlx::query_as!(
        Project,
        "SELECT id, role_id, title, description, requirements, estimated_hours, is_published, created_at, updated_at
         FROM projects WHERE role_id = $1 AND is_published = true",
        role_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_project_by_id(pool: &PgPool, project_id: Uuid) -> sqlx::Result<Option<Project>> {
    sqlx::query_as!(
        Project,
        "SELECT id, role_id, title, description, requirements, estimated_hours, is_published, created_at, updated_at
         FROM projects WHERE id = $1",
        project_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn count_module_completion(
    pool: &PgPool,
    role_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<ModuleCompletionSummary> {
    let row = sqlx::query!(
        r#"
        WITH module_sub_counts AS (
            SELECT
                lm.id AS module_id,
                COUNT(DISTINCT ls.id) AS total_subs,
                COUNT(DISTINCT usp.submaterial_id) AS completed_subs
            FROM learning_modules lm
            LEFT JOIN submaterials ls ON ls.module_id = lm.id
            LEFT JOIN user_submaterial_progress usp
                ON usp.submaterial_id = ls.id AND usp.user_id = $2
            WHERE lm.role_id = $1 AND lm.is_published = true
            GROUP BY lm.id
        ),
        module_quiz_best AS (
            SELECT
                lm.id AS module_id,
                COUNT(DISTINCT mq.id) AS total_questions,
                COALESCE(MAX(mqa.score), -1.0::float8) AS best_score
            FROM learning_modules lm
            LEFT JOIN module_quizzes mq ON mq.module_id = lm.id
            LEFT JOIN module_quiz_attempts mqa
                ON mqa.module_id = lm.id AND mqa.user_id = $2
            WHERE lm.role_id = $1 AND lm.is_published = true
            GROUP BY lm.id
        )
        SELECT
            COUNT(*) AS "total!: i64",
            COUNT(*) FILTER (
                WHERE msc.total_subs > 0
                  AND msc.completed_subs = msc.total_subs
                  AND (mqb.total_questions = 0 OR mqb.best_score >= 70.0)
            ) AS "completed!: i64"
        FROM module_sub_counts msc
        JOIN module_quiz_best mqb USING (module_id)
        "#,
        role_id,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(ModuleCompletionSummary {
        total: row.total,
        completed: row.completed,
    })
}

pub async fn insert_submission(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    file_path: &str,
    file_original_name: &str,
    file_size_bytes: i64,
    file_mime_type: &str,
    notes: &str,
) -> sqlx::Result<ProjectSubmission> {
    sqlx::query_as!(
        ProjectSubmission,
        r#"INSERT INTO project_submissions (project_id, user_id, file_path, file_original_name, file_size_bytes, file_mime_type, submission_notes)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, project_id, user_id, file_path, file_original_name, file_size_bytes, file_mime_type, submission_notes, status, reviewer_notes, submitted_at, reviewed_at, reviewed_by"#,
        project_id,
        user_id,
        file_path,
        file_original_name,
        file_size_bytes,
        file_mime_type,
        notes
    )
    .fetch_one(pool)
    .await
}

pub async fn get_submissions_by_project_and_user(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Vec<ProjectSubmission>> {
    sqlx::query_as!(
        ProjectSubmission,
        r#"SELECT id, project_id, user_id, file_path, file_original_name, file_size_bytes, file_mime_type, submission_notes, status, reviewer_notes, submitted_at, reviewed_at, reviewed_by
           FROM project_submissions
           WHERE project_id = $1 AND user_id = $2
           ORDER BY submitted_at DESC"#,
        project_id,
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_submission_by_id(
    pool: &PgPool,
    submission_id: Uuid,
) -> sqlx::Result<Option<ProjectSubmission>> {
    sqlx::query_as!(
        ProjectSubmission,
        r#"SELECT id, project_id, user_id, file_path, file_original_name, file_size_bytes, file_mime_type, submission_notes, status, reviewer_notes, submitted_at, reviewed_at, reviewed_by
           FROM project_submissions WHERE id = $1"#,
        submission_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_latest_submission_status(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Option<String>> {
    let row = sqlx::query!(
        r#"SELECT status
           FROM project_submissions
           WHERE project_id = $1 AND user_id = $2
           ORDER BY submitted_at DESC
           LIMIT 1"#,
        project_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.status))
}
