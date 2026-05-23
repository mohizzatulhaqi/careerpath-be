use anyhow::Context;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::audit::dto::PaginationDto;
use crate::features::admin::user::{
    dto::{
        ModuleProgressDto, ProjectSubmissionDto, QuizAttemptDto, UpdateUserRequest, UserDetailDto,
        UserListQuery, UserListResponse, UserStatsDto, UserSummaryDto,
    },
    entity::{UserDetailRow, UserRow},
};

// ── Paginated list with stats (no N+1) ───────────────────────────────────────

pub async fn list_users(
    pool: &sqlx::PgPool,
    q: &UserListQuery,
) -> anyhow::Result<UserListResponse> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Validate sort column to prevent SQL injection
    let sort_col = match q.sort.as_deref().unwrap_or("created_at") {
        "name" => "u.name",
        "email" => "u.email",
        "role" => "u.role",
        _ => "u.created_at",
    };
    let sort_dir = match q.order.as_deref().unwrap_or("desc") {
        "asc" => "ASC",
        _ => "DESC",
    };

    let search_pattern = q.search.as_deref().map(|s| format!("%{s}%"));

    // Dynamic ORDER BY via format! is safe because sort_col/sort_dir are whitelisted above.
    let count_sql = format!(
        r#"
        SELECT COUNT(*) FROM users u
        WHERE
            ($1::text IS NULL OR u.name ILIKE $1 OR u.email ILIKE $1)
            AND ($2::text IS NULL OR u.role::text = $2)
            AND ($3::bool IS NULL OR u.is_active = $3)
        "#
    );

    let total_items: i64 = sqlx::query_scalar::<_, Option<i64>>(&count_sql)
        .bind(&search_pattern)
        .bind(&q.role)
        .bind(q.is_active)
        .fetch_one(pool)
        .await
        .context("count users")?
        .unwrap_or(0);

    let list_sql = format!(
        r#"
        SELECT
            u.id,
            u.email,
            u.name,
            u.role::text               AS role,
            u.is_active,
            u.deactivated_at,
            u.deactivated_by,
            u.deactivation_reason,
            u.created_at,
            u.updated_at,
            COALESCE(mqa.cnt, 0)       AS quiz_attempts,
            COALESCE(mc.cnt, 0)        AS modules_completed,
            COALESCE(ps.cnt, 0)        AS projects_submitted
        FROM users u
        LEFT JOIN LATERAL (
            SELECT COUNT(*) AS cnt
            FROM module_quiz_attempts
            WHERE user_id = u.id
        ) mqa ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*) AS cnt
            FROM module_quiz_attempts
            WHERE user_id = u.id AND passed = true
        ) mc ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*) AS cnt
            FROM project_submissions
            WHERE user_id = u.id
        ) ps ON true
        WHERE
            ($1::text IS NULL OR u.name ILIKE $1 OR u.email ILIKE $1)
            AND ($2::text IS NULL OR u.role::text = $2)
            AND ($3::bool IS NULL OR u.is_active = $3)
        ORDER BY {sort_col} {sort_dir}
        LIMIT $4 OFFSET $5
        "#
    );

    let rows: Vec<UserRow> = sqlx::query_as(&list_sql)
        .bind(&search_pattern)
        .bind(&q.role)
        .bind(q.is_active)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .context("list users")?;

    let total_pages = (total_items + per_page - 1) / per_page;

    let users = rows
        .into_iter()
        .map(|r| UserSummaryDto {
            id: r.id,
            email: r.email,
            name: r.name,
            role: r.role,
            is_active: r.is_active,
            deactivated_at: r.deactivated_at,
            deactivation_reason: r.deactivation_reason,
            created_at: r.created_at,
            stats: UserStatsDto {
                quiz_attempts: r.quiz_attempts.unwrap_or(0),
                modules_completed: r.modules_completed.unwrap_or(0),
                projects_submitted: r.projects_submitted.unwrap_or(0),
            },
        })
        .collect();

    Ok(UserListResponse {
        users,
        pagination: PaginationDto {
            page,
            per_page,
            total_items,
            total_pages,
        },
    })
}

// ── Find by id ────────────────────────────────────────────────────────────────

pub async fn find_by_id(pool: &sqlx::PgPool, user_id: Uuid) -> anyhow::Result<Option<UserDetailRow>> {
    let row = sqlx::query_as!(
        UserDetailRow,
        r#"
        SELECT
            id, email, name, role::text AS "role!", is_active,
            deactivated_at, deactivated_by, deactivation_reason,
            created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .context("find user by id")?;

    Ok(row)
}

/// Full detail: user row + 3 history lists (no N+1 — separate batch queries).
pub async fn get_user_detail(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> anyhow::Result<Option<UserDetailDto>> {
    let Some(user) = find_by_id(pool, user_id).await? else {
        return Ok(None);
    };

    // Stats
    let quiz_attempts: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM module_quiz_attempts WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let modules_completed: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM module_quiz_attempts WHERE user_id = $1 AND passed = true",
        user_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let projects_submitted: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM project_submissions WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    // Module progress — latest module_quiz_attempt per module.
    // Wrap lateral-join columns in CASE WHEN so PostgreSQL marks them nullable;
    // otherwise the query planner reports NOT NULL (from the inner table definition)
    // but runtime values are NULL when no attempt row exists → decode error.
    let module_progress = sqlx::query_as!(
        crate::features::admin::user::entity::UserModuleProgress,
        r#"
        SELECT
            lm.id          AS module_id,
            lm.title       AS module_title,
            CASE
                WHEN mqa.passed = true THEN 'completed'
                WHEN mqa.id IS NOT NULL THEN 'in_progress'
                ELSE 'not_started'
            END            AS "status!",
            CASE WHEN mqa.id IS NOT NULL THEN mqa.score       END AS score,
            CASE WHEN mqa.id IS NOT NULL THEN mqa.created_at  END AS completed_at
        FROM learning_modules lm
        LEFT JOIN LATERAL (
            SELECT id, score, passed, created_at
            FROM module_quiz_attempts
            WHERE user_id = $1 AND module_id = lm.id
            ORDER BY created_at DESC
            LIMIT 1
        ) mqa ON true
        WHERE lm.is_published = true
        ORDER BY lm.order_index
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("fetch module progress")?;

    // Recent quiz attempts (last 10 module quiz attempts)
    let recent_quiz_attempts = sqlx::query_as!(
        crate::features::admin::user::entity::UserQuizAttempt,
        r#"
        SELECT
            mqa.module_id  AS quiz_id,
            lm.title       AS quiz_title,
            mqa.score,
            mqa.passed,
            mqa.created_at AS attempted_at
        FROM module_quiz_attempts mqa
        JOIN learning_modules lm ON lm.id = mqa.module_id
        WHERE mqa.user_id = $1
        ORDER BY mqa.created_at DESC
        LIMIT 10
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("fetch quiz attempts")?;

    // Project submissions
    let project_submissions = sqlx::query_as!(
        crate::features::admin::user::entity::UserProjectSubmission,
        r#"
        SELECT
            ps.project_id,
            p.title AS project_title,
            ps.status,
            ps.submitted_at
        FROM project_submissions ps
        JOIN projects p ON p.id = ps.project_id
        WHERE ps.user_id = $1
        ORDER BY ps.submitted_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("fetch project submissions")?;

    Ok(Some(UserDetailDto {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        is_active: user.is_active,
        deactivated_at: user.deactivated_at,
        deactivated_by: user.deactivated_by,
        deactivation_reason: user.deactivation_reason,
        created_at: user.created_at,
        updated_at: user.updated_at,
        stats: UserStatsDto {
            quiz_attempts,
            modules_completed,
            projects_submitted,
        },
        module_progress: module_progress
            .into_iter()
            .map(|r| ModuleProgressDto {
                module_id: r.module_id,
                module_title: r.module_title,
                status: r.status,
                score: r.score,
                completed_at: r.completed_at,
            })
            .collect(),
        recent_quiz_attempts: recent_quiz_attempts
            .into_iter()
            .map(|r| QuizAttemptDto {
                quiz_id: r.quiz_id,
                quiz_title: r.quiz_title,
                score: r.score,
                passed: r.passed,
                attempted_at: r.attempted_at,
            })
            .collect(),
        project_submissions: project_submissions
            .into_iter()
            .map(|r| ProjectSubmissionDto {
                project_id: r.project_id,
                project_title: r.project_title,
                status: r.status,
                submitted_at: r.submitted_at,
            })
            .collect(),
    }))
}

// ── Update user (name and/or role) inside a transaction ──────────────────────

pub async fn update_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    req: &UpdateUserRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            name       = COALESCE($2, name),
            role       = COALESCE($3::user_role, role),
            updated_at = NOW()
        WHERE id = $1
        "#,
        user_id,
        req.name,
        req.role as _,
    )
    .execute(&mut **tx)
    .await
    .context("update_user")?;

    Ok(())
}

// ── Deactivate user inside a transaction ─────────────────────────────────────

pub async fn deactivate_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    by_admin_id: Uuid,
    reason: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            is_active           = false,
            deactivated_at      = NOW(),
            deactivated_by      = $2,
            deactivation_reason = $3,
            updated_at          = NOW()
        WHERE id = $1
        "#,
        user_id,
        by_admin_id,
        reason,
    )
    .execute(&mut **tx)
    .await
    .context("deactivate_user")?;

    Ok(())
}

// ── Activate user inside a transaction ───────────────────────────────────────

pub async fn activate_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            is_active           = true,
            deactivated_at      = NULL,
            deactivated_by      = NULL,
            deactivation_reason = NULL,
            updated_at          = NOW()
        WHERE id = $1
        "#,
        user_id,
    )
    .execute(&mut **tx)
    .await
    .context("activate_user")?;

    Ok(())
}
