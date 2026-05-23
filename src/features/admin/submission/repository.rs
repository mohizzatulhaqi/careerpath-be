use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::dto::{
    CurrentReviewDto, FileBriefDto, ProjectBriefDto, ProjectDetailForReviewDto, QueueSummaryDto,
    QueueStatsDto, RoleBriefDto, RolePendingCountDto, ReviewerBriefDto, ReviewHistoryEntryDto,
    SubmissionDetailDto, SubmissionHistoryItemDto, SubmissionSummaryDto, SubmissionFilter,
    UserBriefDto, UserDetailForReviewDto,
};

// ── Row types (internal, not public) ─────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct SubmissionListRow {
    pub id: Uuid,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    // user
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    // project
    pub project_id: Uuid,
    pub project_title: String,
    // role
    pub role_code: String,
    pub role_name: String,
    // file
    pub file_original_name: String,
    pub file_size_bytes: i64,
    pub file_mime_type: String,
    // notes preview (first 200 chars)
    pub submission_notes_preview: String,
    // reviewer (nullable)
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    // previous attempts
    pub previous_attempts_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SubmissionLockRow {
    pub id: Uuid,
    pub status: String,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub reviewer_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub file_path: String,
    pub file_original_name: String,
    pub file_size_bytes: i64,
    pub file_mime_type: String,
    pub submission_notes: String,
    pub submitted_at: DateTime<Utc>,
}

// ── Aggregate stats row ───────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct StatusCountRow {
    pub status: String,
    pub cnt: i64,
}

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn list_submissions(
    pool: &PgPool,
    filter: &SubmissionFilter,
) -> anyhow::Result<(Vec<SubmissionSummaryDto>, i64)> {
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let effective_status = filter
        .status
        .as_deref()
        .filter(|s| *s != "all")
        .unwrap_or("pending_review");
    let filter_all = filter.status.as_deref() == Some("all");

    let sort_asc = filter.sort.as_deref() != Some("submitted_at_desc");

    // Count query
    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM project_submissions ps
        JOIN projects p ON p.id = ps.project_id
        JOIN roles r ON r.id = p.role_id
        JOIN users u ON u.id = ps.user_id
        WHERE
            ($1::boolean OR ps.status = $2)
            AND ($3::uuid IS NULL OR ps.project_id = $3)
            AND ($4::uuid IS NULL OR ps.user_id = $4)
            AND ($5::uuid IS NULL OR p.role_id = $5)
            AND ($6::timestamptz IS NULL OR ps.submitted_at >= $6)
            AND ($7::timestamptz IS NULL OR ps.submitted_at <= $7)
        "#,
        filter_all,
        effective_status,
        filter.project_id,
        filter.user_id,
        filter.role_id,
        filter.from_date,
        filter.to_date,
    )
    .fetch_one(pool)
    .await
    .context("count submissions")?
    .unwrap_or(0);

    // Data rows — use CASE for sort direction since format! for ORDER BY is
    // whitelisted (no user input reaches sort_asc).
    let rows = if sort_asc {
        sqlx::query_as!(
            SubmissionListRow,
            r#"
            SELECT
                ps.id,
                ps.status,
                ps.submitted_at,
                u.id               AS user_id,
                u.name             AS user_name,
                u.email            AS user_email,
                p.id               AS project_id,
                p.title            AS project_title,
                r.code             AS role_code,
                r.name             AS role_name,
                ps.file_original_name,
                ps.file_size_bytes,
                ps.file_mime_type,
                LEFT(ps.submission_notes, 200) AS "submission_notes_preview!",
                -- Wrap LEFT JOIN columns in CASE WHEN so PG reports them as nullable;
                -- otherwise PG reports users.id as NOT NULL even after a LEFT JOIN
                -- → sqlx generates non-null decode → runtime error when reviewed_by IS NULL.
                CASE WHEN ps.reviewed_by IS NOT NULL THEN ru.id   END AS reviewer_id,
                CASE WHEN ps.reviewed_by IS NOT NULL THEN ru.name END AS reviewer_name,
                ps.reviewed_at,
                (
                    SELECT COUNT(*) - 1
                    FROM project_submissions x
                    WHERE x.user_id = ps.user_id AND x.project_id = ps.project_id
                ) AS "previous_attempts_count!"
            FROM project_submissions ps
            JOIN projects p ON p.id = ps.project_id
            JOIN roles r ON r.id = p.role_id
            JOIN users u ON u.id = ps.user_id
            LEFT JOIN users ru ON ru.id = ps.reviewed_by
            WHERE
                ($1::boolean OR ps.status = $2)
                AND ($3::uuid IS NULL OR ps.project_id = $3)
                AND ($4::uuid IS NULL OR ps.user_id = $4)
                AND ($5::uuid IS NULL OR p.role_id = $5)
                AND ($6::timestamptz IS NULL OR ps.submitted_at >= $6)
                AND ($7::timestamptz IS NULL OR ps.submitted_at <= $7)
            ORDER BY ps.submitted_at ASC
            LIMIT $8 OFFSET $9
            "#,
            filter_all,
            effective_status,
            filter.project_id,
            filter.user_id,
            filter.role_id,
            filter.from_date,
            filter.to_date,
            per_page,
            offset,
        )
        .fetch_all(pool)
        .await
        .context("list submissions asc")?
    } else {
        sqlx::query_as!(
            SubmissionListRow,
            r#"
            SELECT
                ps.id,
                ps.status,
                ps.submitted_at,
                u.id               AS user_id,
                u.name             AS user_name,
                u.email            AS user_email,
                p.id               AS project_id,
                p.title            AS project_title,
                r.code             AS role_code,
                r.name             AS role_name,
                ps.file_original_name,
                ps.file_size_bytes,
                ps.file_mime_type,
                LEFT(ps.submission_notes, 200) AS "submission_notes_preview!",
                CASE WHEN ps.reviewed_by IS NOT NULL THEN ru.id   END AS reviewer_id,
                CASE WHEN ps.reviewed_by IS NOT NULL THEN ru.name END AS reviewer_name,
                ps.reviewed_at,
                (
                    SELECT COUNT(*) - 1
                    FROM project_submissions x
                    WHERE x.user_id = ps.user_id AND x.project_id = ps.project_id
                ) AS "previous_attempts_count!"
            FROM project_submissions ps
            JOIN projects p ON p.id = ps.project_id
            JOIN roles r ON r.id = p.role_id
            JOIN users u ON u.id = ps.user_id
            LEFT JOIN users ru ON ru.id = ps.reviewed_by
            WHERE
                ($1::boolean OR ps.status = $2)
                AND ($3::uuid IS NULL OR ps.project_id = $3)
                AND ($4::uuid IS NULL OR ps.user_id = $4)
                AND ($5::uuid IS NULL OR p.role_id = $5)
                AND ($6::timestamptz IS NULL OR ps.submitted_at >= $6)
                AND ($7::timestamptz IS NULL OR ps.submitted_at <= $7)
            ORDER BY ps.submitted_at DESC
            LIMIT $8 OFFSET $9
            "#,
            filter_all,
            effective_status,
            filter.project_id,
            filter.user_id,
            filter.role_id,
            filter.from_date,
            filter.to_date,
            per_page,
            offset,
        )
        .fetch_all(pool)
        .await
        .context("list submissions desc")?
    };

    let dtos = rows
        .into_iter()
        .map(|r| SubmissionSummaryDto {
            id: r.id,
            status: r.status,
            submitted_at: r.submitted_at,
            user: UserBriefDto {
                id: r.user_id,
                name: r.user_name,
                email: r.user_email,
            },
            project: ProjectBriefDto {
                id: r.project_id,
                title: r.project_title,
                role: RoleBriefDto {
                    code: r.role_code,
                    name: r.role_name,
                },
            },
            file: FileBriefDto {
                original_name: r.file_original_name,
                size_bytes: r.file_size_bytes,
                mime_type: r.file_mime_type,
                download_url: format!("/api/admin/submissions/{}/download", r.id),
            },
            submission_notes_preview: r.submission_notes_preview,
            reviewer: r.reviewer_id.zip(r.reviewer_name).map(|(id, name)| ReviewerBriefDto { id, name }),
            reviewed_at: r.reviewed_at,
            previous_attempts_count: r.previous_attempts_count,
        })
        .collect();

    Ok((dtos, total))
}

// ── Global summary counts ─────────────────────────────────────────────────────

pub async fn get_summary_counts(pool: &PgPool) -> anyhow::Result<QueueSummaryDto> {
    let rows = sqlx::query_as!(
        StatusCountRow,
        r#"
        SELECT status, COUNT(*) AS "cnt!"
        FROM project_submissions
        GROUP BY status
        "#,
    )
    .fetch_all(pool)
    .await
    .context("summary counts")?;

    let mut pending_count = 0i64;
    let mut approved_count = 0i64;
    let mut rejected_count = 0i64;
    for r in rows {
        match r.status.as_str() {
            "pending_review" => pending_count = r.cnt,
            "approved" => approved_count = r.cnt,
            "rejected" => rejected_count = r.cnt,
            _ => {}
        }
    }
    Ok(QueueSummaryDto { pending_count, approved_count, rejected_count })
}

// ── Single submission (no lock) ───────────────────────────────────────────────

pub async fn get_submission_detail(
    pool: &PgPool,
    submission_id: Uuid,
) -> anyhow::Result<Option<SubmissionDetailDto>> {
    // Main row
    let row = sqlx::query!(
        r#"
        SELECT
            ps.id,
            ps.status,
            ps.submitted_at,
            ps.submission_notes,
            ps.file_original_name,
            ps.file_size_bytes,
            ps.file_mime_type,
            ps.reviewed_by,
            ps.reviewed_at,
            ps.reviewer_notes,
            -- user
            u.id   AS user_id,
            u.name AS user_name,
            u.email AS user_email,
            -- role (nullable: user might have no role yet or role deactivated)
            r.code AS "role_code?",
            r.name AS "role_name?",
            -- project
            p.id          AS project_id,
            p.title       AS project_title,
            p.description AS project_description,
            p.requirements AS project_requirements,
            p.estimated_hours AS project_estimated_hours,
            -- reviewer (CASE WHEN to force nullable type in sqlx query_as!)
            CASE WHEN ps.reviewed_by IS NOT NULL THEN ru.id   END AS "reviewer_id?",
            CASE WHEN ps.reviewed_by IS NOT NULL THEN ru.name END AS "reviewer_name?"
        FROM project_submissions ps
        JOIN users u ON u.id = ps.user_id
        JOIN projects p ON p.id = ps.project_id
        LEFT JOIN roles r ON r.id = (
            SELECT qa.result_role_id FROM quiz_attempts qa
            WHERE qa.user_id = ps.user_id AND qa.status = 'submitted' AND qa.result_role_id IS NOT NULL
            ORDER BY qa.submitted_at DESC NULLS LAST LIMIT 1
        )
        LEFT JOIN users ru ON ru.id = ps.reviewed_by
        WHERE ps.id = $1
        "#,
        submission_id,
    )
    .fetch_optional(pool)
    .await
    .context("get submission detail")?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    // Module progress counts for the user's role
    let (modules_completed, modules_total) = if let (Some(role_code), Some(_)) =
        (&row.role_code, &row.role_name)
    {
        let role_id = sqlx::query_scalar!(
            "SELECT id FROM roles WHERE code = $1",
            role_code.as_str(),
        )
        .fetch_optional(pool)
        .await
        .context("get role id")?;

        if let Some(role_id) = role_id {
            let total: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM learning_modules WHERE role_id = $1 AND is_published = true",
                role_id
            )
            .fetch_one(pool)
            .await
            .context("count modules")?
            .unwrap_or(0);

            let completed: i64 = sqlx::query_scalar!(
                r#"SELECT COUNT(DISTINCT module_id) FROM module_quiz_attempts
                   WHERE user_id = $1 AND passed = true"#,
                row.user_id
            )
            .fetch_one(pool)
            .await
            .context("count completed")?
            .unwrap_or(0);

            (completed, total)
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    // Review history
    let history_rows = sqlx::query!(
        r#"
        SELECT
            srh.id,
            srh.old_status,
            srh.new_status,
            srh.reviewer_notes,
            srh.created_at,
            u.id   AS reviewer_id,
            u.name AS reviewer_name
        FROM submission_review_history srh
        JOIN users u ON u.id = srh.reviewed_by
        WHERE srh.submission_id = $1
        ORDER BY srh.created_at ASC
        "#,
        submission_id,
    )
    .fetch_all(pool)
    .await
    .context("review history")?;

    let review_history: Vec<ReviewHistoryEntryDto> = history_rows
        .into_iter()
        .map(|h| ReviewHistoryEntryDto {
            id: h.id,
            old_status: h.old_status,
            new_status: h.new_status,
            reviewer: ReviewerBriefDto {
                id: h.reviewer_id,
                name: h.reviewer_name,
            },
            reviewer_notes: h.reviewer_notes,
            created_at: h.created_at,
        })
        .collect();

    // Other submissions by same user for same project
    let history_rows2 = sqlx::query!(
        r#"
        SELECT id, status, submitted_at
        FROM project_submissions
        WHERE user_id = $1 AND project_id = $2
        ORDER BY submitted_at DESC
        "#,
        row.user_id,
        row.project_id,
    )
    .fetch_all(pool)
    .await
    .context("submission history")?;

    let submission_history: Vec<SubmissionHistoryItemDto> = history_rows2
        .into_iter()
        .map(|h| SubmissionHistoryItemDto {
            id: h.id,
            status: h.status,
            submitted_at: h.submitted_at,
        })
        .collect();

    Ok(Some(SubmissionDetailDto {
        id: row.id,
        status: row.status,
        submitted_at: row.submitted_at,
        user: UserDetailForReviewDto {
            id: row.user_id,
            name: row.user_name,
            email: row.user_email,
            role: row.role_code.zip(row.role_name).map(|(code, name)| RoleBriefDto { code, name }),
            modules_completed,
            modules_total,
        },
        project: ProjectDetailForReviewDto {
            id: row.project_id,
            title: row.project_title,
            description: row.project_description,
            requirements: row.project_requirements,
            estimated_hours: row.project_estimated_hours,
        },
        file: FileBriefDto {
            original_name: row.file_original_name.clone(),
            size_bytes: row.file_size_bytes,
            mime_type: row.file_mime_type,
            download_url: format!("/api/admin/submissions/{submission_id}/download"),
        },
        submission_notes: row.submission_notes,
        current_review: CurrentReviewDto {
            reviewer: row.reviewer_id.zip(row.reviewer_name).map(|(id, name)| ReviewerBriefDto { id, name }),
            reviewed_at: row.reviewed_at,
            reviewer_notes: row.reviewer_notes,
        },
        review_history,
        submission_history,
    }))
}

// ── Lock + fetch for review ───────────────────────────────────────────────────

pub async fn find_for_update(
    tx: &mut Transaction<'_, Postgres>,
    submission_id: Uuid,
) -> anyhow::Result<Option<SubmissionLockRow>> {
    sqlx::query_as!(
        SubmissionLockRow,
        r#"
        SELECT id, status, project_id, user_id, reviewed_by, reviewer_notes,
               reviewed_at, file_path, file_original_name, file_size_bytes, file_mime_type,
               submission_notes, submitted_at
        FROM project_submissions
        WHERE id = $1
        FOR UPDATE
        "#,
        submission_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .context("find submission for update")
}

// ── Apply review decision ─────────────────────────────────────────────────────

pub async fn apply_review(
    tx: &mut Transaction<'_, Postgres>,
    submission_id: Uuid,
    admin_id: Uuid,
    old_status: &str,
    new_status: &str,
    notes: &str,
) -> anyhow::Result<DateTime<Utc>> {
    let row = sqlx::query!(
        r#"
        UPDATE project_submissions
        SET status = $1, reviewed_by = $2, reviewed_at = NOW(), reviewer_notes = $3
        WHERE id = $4
        RETURNING reviewed_at
        "#,
        new_status,
        admin_id,
        notes,
        submission_id,
    )
    .fetch_one(&mut **tx)
    .await
    .context("update submission status")?;

    // NULL old_status means "first review from initial pending state"
    let old_status_opt: Option<&str> = if old_status == "pending_review" {
        None
    } else {
        Some(old_status)
    };

    // Insert review history entry
    sqlx::query!(
        r#"
        INSERT INTO submission_review_history (submission_id, reviewed_by, old_status, new_status, reviewer_notes)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        submission_id,
        admin_id,
        old_status_opt as Option<&str>,
        new_status,
        notes,
    )
    .execute(&mut **tx)
    .await
    .context("insert review history")?;

    Ok(row.reviewed_at.unwrap_or_else(Utc::now))
}

// ── File info for download ────────────────────────────────────────────────────

pub async fn get_file_info(
    pool: &PgPool,
    submission_id: Uuid,
) -> anyhow::Result<Option<(String, String, i64, String)>> {
    // Returns (file_path, original_name, size_bytes, mime_type)
    let row = sqlx::query!(
        r#"SELECT file_path, file_original_name, file_size_bytes, file_mime_type
           FROM project_submissions WHERE id = $1"#,
        submission_id,
    )
    .fetch_optional(pool)
    .await
    .context("get file info")?;

    Ok(row.map(|r| (r.file_path, r.file_original_name, r.file_size_bytes, r.file_mime_type)))
}

// ── Brief info for review result response ─────────────────────────────────────

pub async fn get_review_brief(
    pool: &PgPool,
    submission_id: Uuid,
) -> anyhow::Result<Option<(UserBriefDto, ProjectBriefDto)>> {
    let row = sqlx::query!(
        r#"
        SELECT
            u.id   AS user_id,
            u.name AS user_name,
            u.email AS user_email,
            p.id   AS project_id,
            p.title AS project_title,
            r.code AS role_code,
            r.name AS role_name
        FROM project_submissions ps
        JOIN users u ON u.id = ps.user_id
        JOIN projects p ON p.id = ps.project_id
        JOIN roles r ON r.id = p.role_id
        WHERE ps.id = $1
        "#,
        submission_id,
    )
    .fetch_optional(pool)
    .await
    .context("get review brief")?;

    Ok(row.map(|r| {
        (
            UserBriefDto { id: r.user_id, name: r.user_name, email: r.user_email },
            ProjectBriefDto {
                id: r.project_id,
                title: r.project_title,
                role: RoleBriefDto { code: r.role_code, name: r.role_name },
            },
        )
    }))
}

// ── Queue stats ───────────────────────────────────────────────────────────────

pub async fn get_queue_stats(pool: &PgPool) -> anyhow::Result<QueueStatsDto> {
    // pending count + oldest pending
    let pending_row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)                          AS "pending_count!",
            MIN(submitted_at)                 AS oldest_pending
        FROM project_submissions
        WHERE status = 'pending_review'
        "#,
    )
    .fetch_one(pool)
    .await
    .context("pending stats")?;

    // today & this week aggregates
    let review_stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'approved' AND reviewed_at >= NOW()::date)          AS "approved_today!",
            COUNT(*) FILTER (WHERE status = 'rejected' AND reviewed_at >= NOW()::date)          AS "rejected_today!",
            COUNT(*) FILTER (WHERE status = 'approved' AND reviewed_at >= date_trunc('week', NOW())) AS "approved_week!",
            COUNT(*) FILTER (WHERE status = 'rejected' AND reviewed_at >= date_trunc('week', NOW())) AS "rejected_week!"
        FROM project_submissions
        WHERE reviewed_at IS NOT NULL
        "#,
    )
    .fetch_one(pool)
    .await
    .context("review stats")?;

    // by-role pending breakdown
    let by_role_rows = sqlx::query!(
        r#"
        SELECT
            r.code,
            r.name,
            COUNT(*) AS "pending_count!"
        FROM project_submissions ps
        JOIN projects p ON p.id = ps.project_id
        JOIN roles r ON r.id = p.role_id
        WHERE ps.status = 'pending_review'
        GROUP BY r.id, r.code, r.name
        ORDER BY r.name
        "#,
    )
    .fetch_all(pool)
    .await
    .context("by-role stats")?;

    Ok(QueueStatsDto {
        pending_count: pending_row.pending_count,
        oldest_pending_submitted_at: pending_row.oldest_pending,
        approved_today: review_stats.approved_today,
        rejected_today: review_stats.rejected_today,
        approved_this_week: review_stats.approved_week,
        rejected_this_week: review_stats.rejected_week,
        by_role: by_role_rows
            .into_iter()
            .map(|r| RolePendingCountDto {
                role: RoleBriefDto { code: r.code, name: r.name },
                pending_count: r.pending_count,
            })
            .collect(),
    })
}
