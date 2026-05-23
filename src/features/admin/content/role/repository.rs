use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::content::role::dto::{
    CreateRoleRequest, RoleDetailDto, RoleDto, RoleListQuery, RoleListResponse, RoleStatsDto,
    UpdateRoleRequest,
};

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn list_roles(pool: &PgPool, q: &RoleListQuery) -> anyhow::Result<RoleListResponse> {
    let search = q.search.as_deref().map(|s| format!("%{s}%"));

    let rows = sqlx::query!(
        r#"
        SELECT id, code, name, description, is_active, created_at,
               COUNT(*) OVER() AS "total!: i64"
        FROM roles
        WHERE
            ($1::bool IS NULL OR is_active = $1)
            AND ($2::text IS NULL OR name ILIKE $2 OR code ILIKE $2)
        ORDER BY code
        "#,
        q.is_active as _,
        search as _,
    )
    .fetch_all(pool)
    .await
    .context("list_roles")?;

    let total = rows.first().map(|r| r.total).unwrap_or(0);
    let roles = rows
        .into_iter()
        .map(|r| RoleDto {
            id: r.id,
            code: r.code,
            name: r.name,
            description: r.description,
            is_active: r.is_active,
            created_at: r.created_at,
        })
        .collect();

    Ok(RoleListResponse { roles, total })
}

// ── Find single ───────────────────────────────────────────────────────────────

pub async fn find_role(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<RoleDetailDto>> {
    let row = sqlx::query!(
        "SELECT id, code, name, description, is_active, created_at FROM roles WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
    .context("find_role")?;

    let Some(r) = row else { return Ok(None) };

    let total_modules: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM learning_modules WHERE role_id = $1",
        id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let has_project: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE role_id = $1)",
        id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    let total_users_with_role: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM quiz_attempts WHERE result_role_id = $1 AND status = 'submitted'",
        id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    Ok(Some(RoleDetailDto {
        id: r.id,
        code: r.code,
        name: r.name,
        description: r.description,
        is_active: r.is_active,
        created_at: r.created_at,
        stats: RoleStatsDto { total_modules, has_project, total_users_with_role },
    }))
}

// ── Create ────────────────────────────────────────────────────────────────────

pub async fn create_role(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateRoleRequest,
) -> anyhow::Result<Uuid> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO roles (code, name, description)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        req.code,
        req.name,
        req.description as _,
    )
    .fetch_one(&mut **tx)
    .await
    .context("create_role")?;

    Ok(id)
}

// ── Update ────────────────────────────────────────────────────────────────────

pub async fn update_role(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    req: &UpdateRoleRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE roles
        SET name        = COALESCE($2, name),
            description = COALESCE($3, description)
        WHERE id = $1
        "#,
        id,
        req.name as _,
        req.description as _,
    )
    .execute(&mut **tx)
    .await
    .context("update_role")?;
    Ok(())
}

// ── Soft delete / restore ─────────────────────────────────────────────────────

pub async fn set_role_active(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    active: bool,
) -> anyhow::Result<()> {
    sqlx::query!("UPDATE roles SET is_active = $2 WHERE id = $1", id, active)
        .execute(&mut **tx)
        .await
        .context("set_role_active")?;
    Ok(())
}

// ── Guard: how many quiz attempts reference this role? ────────────────────────

pub async fn count_users_with_role(pool: &PgPool, role_id: Uuid) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!(
        "SELECT COUNT(*) FROM quiz_attempts WHERE result_role_id = $1 AND status = 'submitted'",
        role_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}

// ── Code uniqueness check ─────────────────────────────────────────────────────

pub async fn code_exists(pool: &PgPool, code: &str, exclude_id: Option<Uuid>) -> anyhow::Result<bool> {
    let exists = match exclude_id {
        None => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1)",
            code
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false),
        Some(eid) => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1 AND id != $2)",
            code,
            eid
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false),
    };
    Ok(exists)
}
