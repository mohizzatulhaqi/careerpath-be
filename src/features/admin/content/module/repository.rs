use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::{
    audit::dto::PaginationDto,
    content::module::dto::{
        AdminModuleDto, AdminModuleFilter, AdminModuleListResponse, AdminModuleStatsDto,
        CreateModuleRequest, UpdateModuleRequest,
    },
};

pub async fn list_modules(
    pool: &PgPool,
    f: &AdminModuleFilter,
) -> anyhow::Result<AdminModuleListResponse> {
    let page = f.page.unwrap_or(1).max(1);
    let per_page = f.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let search = f.search.as_deref().map(|s| format!("%{s}%"));

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM learning_modules
        WHERE ($1::uuid IS NULL OR role_id = $1)
          AND ($2::bool IS NULL OR is_published = $2)
          AND ($3::text IS NULL OR title ILIKE $3)
        "#,
        f.role_id as _,
        f.is_published as _,
        search as _,
    )
    .fetch_one(pool)
    .await
    .context("count modules")?
    .unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT
            m.id, m.role_id, m.title, m.description, m.order_index,
            m.is_published, m.created_at, m.updated_at,
            COUNT(DISTINCT s.id) AS "total_subs!: i64",
            COUNT(DISTINCT usp.user_id) AS "total_users!: i64"
        FROM learning_modules m
        LEFT JOIN submaterials s ON s.module_id = m.id
        LEFT JOIN user_submaterial_progress usp ON usp.submaterial_id = s.id
        WHERE ($1::uuid IS NULL OR m.role_id = $1)
          AND ($2::bool IS NULL OR m.is_published = $2)
          AND ($3::text IS NULL OR m.title ILIKE $3)
        GROUP BY m.id
        ORDER BY m.role_id, m.order_index
        LIMIT $4 OFFSET $5
        "#,
        f.role_id as _,
        f.is_published as _,
        search as _,
        per_page,
        offset,
    )
    .fetch_all(pool)
    .await
    .context("list modules")?;

    let total_pages = (total + per_page - 1) / per_page;

    Ok(AdminModuleListResponse {
        modules: rows
            .into_iter()
            .map(|r| AdminModuleDto {
                id: r.id,
                role_id: r.role_id,
                title: r.title,
                description: r.description,
                order_index: r.order_index,
                is_published: r.is_published,
                created_at: r.created_at,
                updated_at: r.updated_at,
                stats: AdminModuleStatsDto {
                    total_submaterials: r.total_subs,
                    total_users_started: r.total_users,
                },
            })
            .collect(),
        pagination: PaginationDto { page, per_page, total_items: total, total_pages },
    })
}

pub async fn find_module(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AdminModuleDto>> {
    let row = sqlx::query!(
        r#"
        SELECT
            m.id, m.role_id, m.title, m.description, m.order_index,
            m.is_published, m.created_at, m.updated_at,
            COUNT(DISTINCT s.id) AS "total_subs!: i64",
            COUNT(DISTINCT usp.user_id) AS "total_users!: i64"
        FROM learning_modules m
        LEFT JOIN submaterials s ON s.module_id = m.id
        LEFT JOIN user_submaterial_progress usp ON usp.submaterial_id = s.id
        WHERE m.id = $1
        GROUP BY m.id
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .context("find_module")?;

    Ok(row.map(|r| AdminModuleDto {
        id: r.id,
        role_id: r.role_id,
        title: r.title,
        description: r.description,
        order_index: r.order_index,
        is_published: r.is_published,
        created_at: r.created_at,
        updated_at: r.updated_at,
        stats: AdminModuleStatsDto {
            total_submaterials: r.total_subs,
            total_users_started: r.total_users,
        },
    }))
}

pub async fn next_order_index(pool: &PgPool, role_id: Uuid) -> anyhow::Result<i32> {
    let max: i32 = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(order_index), 0) FROM learning_modules WHERE role_id = $1",
        role_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    Ok(max + 1)
}

pub async fn order_index_conflict(
    pool: &PgPool,
    role_id: Uuid,
    order_index: i32,
    exclude_id: Option<Uuid>,
) -> anyhow::Result<bool> {
    let exists = match exclude_id {
        None => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM learning_modules WHERE role_id=$1 AND order_index=$2)",
            role_id,
            order_index
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false),
        Some(eid) => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM learning_modules WHERE role_id=$1 AND order_index=$2 AND id!=$3)",
            role_id,
            order_index,
            eid
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false),
    };
    Ok(exists)
}

pub async fn create_module(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateModuleRequest,
    order_index: i32,
) -> anyhow::Result<Uuid> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO learning_modules (role_id, title, description, order_index, is_published)
        VALUES ($1, $2, $3, $4, $5) RETURNING id
        "#,
        req.role_id,
        req.title,
        req.description as _,
        order_index,
        req.is_published.unwrap_or(true),
    )
    .fetch_one(&mut **tx)
    .await
    .context("create_module")?;
    Ok(id)
}

pub async fn update_module(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    req: &UpdateModuleRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE learning_modules SET
            title        = COALESCE($2, title),
            description  = COALESCE($3, description),
            order_index  = COALESCE($4, order_index),
            is_published = COALESCE($5, is_published),
            updated_at   = NOW()
        WHERE id = $1
        "#,
        id,
        req.title as _,
        req.description as _,
        req.order_index as _,
        req.is_published as _,
    )
    .execute(&mut **tx)
    .await
    .context("update_module")?;
    Ok(())
}

pub async fn set_module_published(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    published: bool,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE learning_modules SET is_published=$2, updated_at=NOW() WHERE id=$1",
        id,
        published
    )
    .execute(&mut **tx)
    .await
    .context("set_module_published")?;
    Ok(())
}

pub async fn hard_delete_module(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM learning_modules WHERE id=$1", id)
        .execute(&mut **tx)
        .await
        .context("hard_delete_module")?;
    Ok(())
}

pub async fn count_affected_users(pool: &PgPool, module_id: Uuid) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT usp.user_id)
        FROM user_submaterial_progress usp
        JOIN submaterials s ON s.id = usp.submaterial_id
        WHERE s.module_id = $1
        "#,
        module_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}

pub async fn role_exists_and_active(pool: &PgPool, role_id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM roles WHERE id=$1 AND is_active=true)",
        role_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false))
}
