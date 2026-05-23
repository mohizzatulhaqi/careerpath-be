use anyhow::Context;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use serde_json::Value;

use crate::features::admin::audit::{
    dto::{AuditLogFilter, AuditLogPageDto, PaginationDto},
    entity::AuditLogWithAdmin,
};
use crate::features::admin::audit::dto::{AuditAdminDto, AuditLogDto};

// ── Insert (must be called inside a transaction) ─────────────────────────────

pub async fn insert_audit_log(
    tx: &mut Transaction<'_, Postgres>,
    admin_id: Uuid,
    action: &str,
    target_type: &str,
    target_id: Option<Uuid>,
    metadata: Option<Value>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO admin_audit_logs (admin_id, action, target_type, target_id, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        admin_id,
        action,
        target_type,
        target_id,
        metadata,
    )
    .execute(&mut **tx)
    .await
    .context("insert_audit_log")?;

    Ok(())
}

// ── List with filters (paginated, joined with users) ─────────────────────────

pub async fn list_logs_with_admin(
    pool: &sqlx::PgPool,
    filter: &AuditLogFilter,
) -> anyhow::Result<AuditLogPageDto> {
    let page = filter.page.unwrap_or(1).max(1);
    let per_page = filter.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Count total matching rows
    let total_items: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM admin_audit_logs a
        WHERE
            ($1::text   IS NULL OR a.action      = $1)
            AND ($2::uuid   IS NULL OR a.admin_id   = $2)
            AND ($3::text   IS NULL OR a.target_type = $3)
            AND ($4::uuid   IS NULL OR a.target_id  = $4)
            AND ($5::timestamptz IS NULL OR a.created_at >= $5)
            AND ($6::timestamptz IS NULL OR a.created_at <= $6)
        "#,
        filter.action as _,
        filter.admin_id as _,
        filter.target_type as _,
        filter.target_id as _,
        filter.from_date as _,
        filter.to_date as _,
    )
    .fetch_one(pool)
    .await
    .context("count audit logs")?
    .unwrap_or(0);

    let rows = sqlx::query_as!(
        AuditLogWithAdmin,
        r#"
        SELECT
            a.id,
            a.admin_id,
            u.name  AS admin_name,
            u.email AS admin_email,
            a.action,
            a.target_type,
            a.target_id,
            a.metadata,
            a.created_at
        FROM admin_audit_logs a
        JOIN users u ON u.id = a.admin_id
        WHERE
            ($1::text   IS NULL OR a.action      = $1)
            AND ($2::uuid   IS NULL OR a.admin_id   = $2)
            AND ($3::text   IS NULL OR a.target_type = $3)
            AND ($4::uuid   IS NULL OR a.target_id  = $4)
            AND ($5::timestamptz IS NULL OR a.created_at >= $5)
            AND ($6::timestamptz IS NULL OR a.created_at <= $6)
        ORDER BY a.created_at DESC
        LIMIT $7 OFFSET $8
        "#,
        filter.action as _,
        filter.admin_id as _,
        filter.target_type as _,
        filter.target_id as _,
        filter.from_date as _,
        filter.to_date as _,
        per_page,
        offset,
    )
    .fetch_all(pool)
    .await
    .context("list audit logs")?;

    let total_pages = (total_items + per_page - 1) / per_page;

    Ok(AuditLogPageDto {
        logs: rows.into_iter().map(map_row).collect(),
        pagination: PaginationDto {
            page,
            per_page,
            total_items,
            total_pages,
        },
    })
}

// ── List for a specific target (paginated) ───────────────────────────────────

pub async fn list_logs_for_target(
    pool: &sqlx::PgPool,
    target_type: &str,
    target_id: Uuid,
    page: i64,
    per_page: i64,
) -> anyhow::Result<AuditLogPageDto> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total_items: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM admin_audit_logs
        WHERE target_type = $1 AND target_id = $2
        "#,
        target_type,
        target_id,
    )
    .fetch_one(pool)
    .await
    .context("count target audit logs")?
    .unwrap_or(0);

    let rows = sqlx::query_as!(
        AuditLogWithAdmin,
        r#"
        SELECT
            a.id,
            a.admin_id,
            u.name  AS admin_name,
            u.email AS admin_email,
            a.action,
            a.target_type,
            a.target_id,
            a.metadata,
            a.created_at
        FROM admin_audit_logs a
        JOIN users u ON u.id = a.admin_id
        WHERE a.target_type = $1 AND a.target_id = $2
        ORDER BY a.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        target_type,
        target_id,
        per_page,
        offset,
    )
    .fetch_all(pool)
    .await
    .context("list target audit logs")?;

    let total_pages = (total_items + per_page - 1) / per_page;

    Ok(AuditLogPageDto {
        logs: rows.into_iter().map(map_row).collect(),
        pagination: PaginationDto {
            page,
            per_page,
            total_items,
            total_pages,
        },
    })
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn map_row(r: AuditLogWithAdmin) -> AuditLogDto {
    AuditLogDto {
        id: r.id,
        admin: AuditAdminDto {
            id: r.admin_id,
            name: r.admin_name,
            email: r.admin_email,
        },
        action: r.action,
        target_type: r.target_type,
        target_id: r.target_id,
        metadata: r.metadata,
        created_at: r.created_at,
    }
}
