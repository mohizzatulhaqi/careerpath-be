use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::{
    dto::{AdminCertificateFilter, AdminCertificateSummaryDto, CompletedModuleDto},
    entity::Certificate,
};

// ── Eligibility queries ───────────────────────────────────────────────────────

/// Returns the role_id from the user's latest submitted quiz attempt, if any.
pub async fn get_user_role_id(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> anyhow::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        r#"
        SELECT result_role_id
        FROM quiz_attempts
        WHERE user_id = $1 AND status = 'submitted'
        ORDER BY submitted_at DESC
        LIMIT 1
        "#,
        user_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .map(|opt| opt.flatten())
    .context("get user role id")
}

/// Count total published modules for a role.
pub async fn count_total_modules(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> anyhow::Result<i64> {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM learning_modules WHERE role_id = $1 AND is_published = true",
        role_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map(|c| c.unwrap_or(0))
    .context("count total modules")
}

/// Count how many modules the user has passed the final quiz for.
pub async fn count_completed_modules(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<i64> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT mqa.module_id)
        FROM module_quiz_attempts mqa
        JOIN learning_modules lm ON lm.id = mqa.module_id
        WHERE mqa.user_id = $1 AND mqa.passed = true AND lm.role_id = $2
        "#,
        user_id,
        role_id,
    )
    .fetch_one(&mut **tx)
    .await
    .map(|c| c.unwrap_or(0))
    .context("count completed modules")
}

/// Find the latest approved submission for the user's role project.
pub async fn get_approved_submission_id(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    role_id: Uuid,
) -> anyhow::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        r#"
        SELECT ps.id
        FROM project_submissions ps
        JOIN projects p ON p.id = ps.project_id
        WHERE ps.user_id = $1 AND p.role_id = $2 AND ps.status = 'approved'
        ORDER BY ps.reviewed_at DESC
        LIMIT 1
        "#,
        user_id,
        role_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .context("get approved submission id")
}

// ── Certificate existence check ───────────────────────────────────────────────

pub async fn find_existing(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    role_id: Uuid,
) -> anyhow::Result<Option<(Uuid, String)>> {
    let row = sqlx::query!(
        "SELECT id, certificate_code FROM certificates WHERE user_id = $1 AND role_id = $2",
        user_id,
        role_id,
    )
    .fetch_optional(&mut **tx)
    .await
    .context("find existing certificate")?;

    Ok(row.map(|r| (r.id, r.certificate_code)))
}

// ── Snapshot queries ──────────────────────────────────────────────────────────

pub async fn get_user_name(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> anyhow::Result<String> {
    sqlx::query_scalar!("SELECT name FROM users WHERE id = $1", user_id)
        .fetch_one(&mut **tx)
        .await
        .context("get user name for snapshot")
}

pub async fn get_role_snapshot(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> anyhow::Result<(String, String)> {
    let row = sqlx::query!(
        "SELECT name, code FROM roles WHERE id = $1",
        role_id,
    )
    .fetch_one(&mut **tx)
    .await
    .context("get role snapshot")?;
    Ok((row.name, row.code))
}

// ── Insert certificate ────────────────────────────────────────────────────────

pub async fn insert_certificate(
    tx: &mut Transaction<'_, Postgres>,
    code: &str,
    user_id: Uuid,
    role_id: Uuid,
    recipient_name: &str,
    role_name: &str,
    role_code: &str,
    modules_completed_count: i32,
    final_project_submission_id: Uuid,
) -> Result<(Uuid, DateTime<Utc>), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO certificates
            (certificate_code, user_id, role_id, recipient_name, role_name, role_code,
             modules_completed_count, final_project_submission_id)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        RETURNING id, issued_at
        "#,
        code,
        user_id,
        role_id,
        recipient_name,
        role_name,
        role_code,
        modules_completed_count,
        final_project_submission_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok((row.id, row.issued_at))
}

// ── User-facing read queries ──────────────────────────────────────────────────

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Certificate>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, certificate_code, user_id, role_id, recipient_name, role_name, role_code,
               issued_at, modules_completed_count, final_project_submission_id,
               is_revoked, revoked_at, revoked_by, revocation_reason
        FROM certificates
        WHERE user_id = $1
        ORDER BY issued_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("list_for_user")?;

    Ok(rows
        .into_iter()
        .map(|r| Certificate {
            id: r.id,
            certificate_code: r.certificate_code,
            user_id: r.user_id,
            role_id: r.role_id,
            recipient_name: r.recipient_name,
            role_name: r.role_name,
            role_code: r.role_code,
            issued_at: r.issued_at,
            modules_completed_count: r.modules_completed_count,
            final_project_submission_id: r.final_project_submission_id,
            is_revoked: r.is_revoked,
            revoked_at: r.revoked_at,
            revoked_by: r.revoked_by,
            revocation_reason: r.revocation_reason,
        })
        .collect())
}

pub async fn get_by_id(
    pool: &PgPool,
    cert_id: Uuid,
) -> anyhow::Result<Option<Certificate>> {
    let row = sqlx::query!(
        r#"
        SELECT id, certificate_code, user_id, role_id, recipient_name, role_name, role_code,
               issued_at, modules_completed_count, final_project_submission_id,
               is_revoked, revoked_at, revoked_by, revocation_reason
        FROM certificates
        WHERE id = $1
        "#,
        cert_id,
    )
    .fetch_optional(pool)
    .await
    .context("get_by_id")?;

    Ok(row.map(|r| Certificate {
        id: r.id,
        certificate_code: r.certificate_code,
        user_id: r.user_id,
        role_id: r.role_id,
        recipient_name: r.recipient_name,
        role_name: r.role_name,
        role_code: r.role_code,
        issued_at: r.issued_at,
        modules_completed_count: r.modules_completed_count,
        final_project_submission_id: r.final_project_submission_id,
        is_revoked: r.is_revoked,
        revoked_at: r.revoked_at,
        revoked_by: r.revoked_by,
        revocation_reason: r.revocation_reason,
    }))
}

pub async fn get_by_code(pool: &PgPool, code: &str) -> anyhow::Result<Option<Certificate>> {
    let row = sqlx::query!(
        r#"
        SELECT id, certificate_code, user_id, role_id, recipient_name, role_name, role_code,
               issued_at, modules_completed_count, final_project_submission_id,
               is_revoked, revoked_at, revoked_by, revocation_reason
        FROM certificates
        WHERE certificate_code = $1
        "#,
        code,
    )
    .fetch_optional(pool)
    .await
    .context("get_by_code")?;

    Ok(row.map(|r| Certificate {
        id: r.id,
        certificate_code: r.certificate_code,
        user_id: r.user_id,
        role_id: r.role_id,
        recipient_name: r.recipient_name,
        role_name: r.role_name,
        role_code: r.role_code,
        issued_at: r.issued_at,
        modules_completed_count: r.modules_completed_count,
        final_project_submission_id: r.final_project_submission_id,
        is_revoked: r.is_revoked,
        revoked_at: r.revoked_at,
        revoked_by: r.revoked_by,
        revocation_reason: r.revocation_reason,
    }))
}

// ── Achievement detail queries ────────────────────────────────────────────────

pub async fn get_completed_modules(
    pool: &PgPool,
    role_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<Vec<CompletedModuleDto>> {
    let rows = sqlx::query!(
        r#"
        SELECT lm.id, lm.title,
               (SELECT MAX(mqa.created_at)
                FROM module_quiz_attempts mqa
                WHERE mqa.module_id = lm.id AND mqa.user_id = $2 AND mqa.passed = true
               ) AS completion_date
        FROM learning_modules lm
        WHERE lm.role_id = $1 AND lm.is_published = true
        ORDER BY lm.order_index
        "#,
        role_id,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("get_completed_modules")?;

    Ok(rows
        .into_iter()
        .map(|r| CompletedModuleDto {
            id: r.id,
            title: r.title,
            completion_date: r.completion_date,
        })
        .collect())
}

pub async fn get_project_title_and_approved_at(
    pool: &PgPool,
    submission_id: Uuid,
) -> anyhow::Result<(String, Option<DateTime<Utc>>)> {
    let row = sqlx::query!(
        r#"
        SELECT p.title, ps.reviewed_at AS approved_at
        FROM project_submissions ps
        JOIN projects p ON p.id = ps.project_id
        WHERE ps.id = $1
        "#,
        submission_id,
    )
    .fetch_one(pool)
    .await
    .context("get project title and approved_at")?;

    Ok((row.title, row.approved_at))
}

// ── Admin queries ─────────────────────────────────────────────────────────────

pub async fn admin_list(
    pool: &PgPool,
    filter: &AdminCertificateFilter,
) -> anyhow::Result<(Vec<AdminCertificateSummaryDto>, i64)> {
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let search = filter.search.as_deref().map(|s| format!("%{s}%"));

    let rows = sqlx::query!(
        r#"
        SELECT
            c.id, c.certificate_code, c.recipient_name, c.user_id,
            u.email AS user_email,
            c.role_name, c.role_code, c.issued_at,
            c.modules_completed_count, c.is_revoked,
            c.revoked_at, c.revocation_reason,
            COUNT(*) OVER() AS "total!: i64"
        FROM certificates c
        JOIN users u ON u.id = c.user_id
        WHERE ($1::uuid  IS NULL OR c.user_id  = $1)
          AND ($2::uuid  IS NULL OR c.role_id  = $2)
          AND ($3::bool  IS NULL OR c.is_revoked = $3)
          AND ($4::timestamptz IS NULL OR c.issued_at >= $4)
          AND ($5::timestamptz IS NULL OR c.issued_at <= $5)
          AND ($6::text  IS NULL OR c.certificate_code ILIKE $6 OR c.recipient_name ILIKE $6)
        ORDER BY c.issued_at DESC
        LIMIT $7 OFFSET $8
        "#,
        filter.user_id as _,
        filter.role_id as _,
        filter.is_revoked as _,
        filter.from_date as _,
        filter.to_date as _,
        search as _,
        per_page,
        offset,
    )
    .fetch_all(pool)
    .await
    .context("admin_list certificates")?;

    let total = rows.first().map(|r| r.total).unwrap_or(0);
    let items = rows
        .into_iter()
        .map(|r| AdminCertificateSummaryDto {
            id: r.id,
            certificate_code: r.certificate_code,
            recipient_name: r.recipient_name,
            user_id: r.user_id,
            user_email: r.user_email,
            role_name: r.role_name,
            role_code: r.role_code,
            issued_at: r.issued_at,
            modules_completed_count: r.modules_completed_count,
            is_revoked: r.is_revoked,
            revoked_at: r.revoked_at,
            revocation_reason: r.revocation_reason,
        })
        .collect();

    Ok((items, total))
}

// ── Admin revoke / restore ────────────────────────────────────────────────────

pub async fn apply_revoke(
    tx: &mut Transaction<'_, Postgres>,
    cert_id: Uuid,
    admin_id: Uuid,
    reason: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE certificates
        SET is_revoked = true, revoked_at = NOW(), revoked_by = $2, revocation_reason = $3
        WHERE id = $1
        "#,
        cert_id,
        admin_id,
        reason,
    )
    .execute(&mut **tx)
    .await
    .context("apply_revoke")?;
    Ok(())
}

pub async fn apply_restore(
    tx: &mut Transaction<'_, Postgres>,
    cert_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE certificates
        SET is_revoked = false, revoked_at = NULL, revoked_by = NULL, revocation_reason = NULL
        WHERE id = $1
        "#,
        cert_id,
    )
    .execute(&mut **tx)
    .await
    .context("apply_restore")?;
    Ok(())
}

// ── Dashboard helper ──────────────────────────────────────────────────────────

pub async fn list_for_user_brief(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<(Uuid, String, String, DateTime<Utc>, bool)>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, certificate_code, role_name, issued_at, is_revoked
        FROM certificates
        WHERE user_id = $1
        ORDER BY issued_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
    .context("list_for_user_brief")?;

    Ok(rows
        .into_iter()
        .map(|r| (r.id, r.certificate_code, r.role_name, r.issued_at, r.is_revoked))
        .collect())
}
