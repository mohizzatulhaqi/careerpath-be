use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use serde_json::Value;

use crate::features::admin::audit::{
    dto::{AuditLogFilter, AuditLogPageDto},
    repository,
};

// ── Write path (inside a transaction) ────────────────────────────────────────

/// Log a single audit event. Must be called inside an open transaction so the
/// audit row is committed atomically with the mutation it describes.
pub async fn log(
    tx: &mut Transaction<'_, Postgres>,
    admin_id: Uuid,
    action: &str,
    target_type: &str,
    target_id: Option<Uuid>,
    metadata: Option<Value>,
) -> anyhow::Result<()> {
    repository::insert_audit_log(tx, admin_id, action, target_type, target_id, metadata).await
}

// ── Read paths ───────────────────────────────────────────────────────────────

/// Paginated global audit log with optional filters.
pub async fn list_logs(
    pool: &sqlx::PgPool,
    filter: &AuditLogFilter,
) -> anyhow::Result<AuditLogPageDto> {
    repository::list_logs_with_admin(pool, filter).await
}

/// Paginated audit history for a single target (e.g., a specific user).
pub async fn list_logs_for_target(
    pool: &sqlx::PgPool,
    target_type: &str,
    target_id: Uuid,
    page: i64,
    per_page: i64,
) -> anyhow::Result<AuditLogPageDto> {
    repository::list_logs_for_target(pool, target_type, target_id, page, per_page).await
}
