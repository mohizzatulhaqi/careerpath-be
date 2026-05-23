use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::{
    audit::dto::PaginationDto,
    content::submaterial::dto::{
        AdminSubmaterialDto, AdminSubmaterialFilter, AdminSubmaterialListResponse,
        CreateSubmaterialRequest, UpdateSubmaterialRequest,
    },
};

pub async fn list_submaterials(
    pool: &PgPool,
    f: &AdminSubmaterialFilter,
) -> anyhow::Result<AdminSubmaterialListResponse> {
    let page = f.page.unwrap_or(1).max(1);
    let per_page = f.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let search = f.search.as_deref().map(|s| format!("%{s}%"));

    let total: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM submaterials
        WHERE ($1::uuid IS NULL OR module_id = $1)
          AND ($2::bool IS NULL OR is_published = $2)
          AND ($3::text IS NULL OR title ILIKE $3)
        "#,
        f.module_id as _,
        f.is_published as _,
        search as _,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT
            s.id, s.module_id, s.title, s.content, s.order_index,
            s.estimated_minutes, s.is_published, s.created_at, s.updated_at,
            EXISTS(SELECT 1 FROM submaterial_quizzes sq WHERE sq.submaterial_id = s.id AND sq.is_published=true)
                AS "has_mini_quiz!: bool"
        FROM submaterials s
        WHERE ($1::uuid IS NULL OR s.module_id = $1)
          AND ($2::bool IS NULL OR s.is_published = $2)
          AND ($3::text IS NULL OR s.title ILIKE $3)
        ORDER BY s.module_id, s.order_index
        LIMIT $4 OFFSET $5
        "#,
        f.module_id as _,
        f.is_published as _,
        search as _,
        per_page,
        offset,
    )
    .fetch_all(pool)
    .await
    .context("list_submaterials")?;

    let total_pages = (total + per_page - 1) / per_page;

    Ok(AdminSubmaterialListResponse {
        submaterials: rows
            .into_iter()
            .map(|r| AdminSubmaterialDto {
                id: r.id,
                module_id: r.module_id,
                title: r.title,
                content: r.content,
                order_index: r.order_index,
                estimated_minutes: r.estimated_minutes,
                is_published: r.is_published,
                has_mini_quiz: r.has_mini_quiz,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect(),
        pagination: PaginationDto { page, per_page, total_items: total, total_pages },
    })
}

pub async fn find_submaterial(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AdminSubmaterialDto>> {
    let row = sqlx::query!(
        r#"
        SELECT
            s.id, s.module_id, s.title, s.content, s.order_index,
            s.estimated_minutes, s.is_published, s.created_at, s.updated_at,
            EXISTS(SELECT 1 FROM submaterial_quizzes sq WHERE sq.submaterial_id = s.id AND sq.is_published=true)
                AS "has_mini_quiz!: bool"
        FROM submaterials s
        WHERE s.id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .context("find_submaterial")?;

    Ok(row.map(|r| AdminSubmaterialDto {
        id: r.id,
        module_id: r.module_id,
        title: r.title,
        content: r.content,
        order_index: r.order_index,
        estimated_minutes: r.estimated_minutes,
        is_published: r.is_published,
        has_mini_quiz: r.has_mini_quiz,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

pub async fn next_order_index(pool: &PgPool, module_id: Uuid) -> anyhow::Result<i32> {
    let max = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(order_index),0) FROM submaterials WHERE module_id=$1",
        module_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    Ok(max + 1)
}

pub async fn order_index_conflict(
    pool: &PgPool,
    module_id: Uuid,
    order_index: i32,
    exclude_id: Option<Uuid>,
) -> anyhow::Result<bool> {
    let exists = match exclude_id {
        None => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM submaterials WHERE module_id=$1 AND order_index=$2)",
            module_id, order_index
        ).fetch_one(pool).await?.unwrap_or(false),
        Some(eid) => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM submaterials WHERE module_id=$1 AND order_index=$2 AND id!=$3)",
            module_id, order_index, eid
        ).fetch_one(pool).await?.unwrap_or(false),
    };
    Ok(exists)
}

pub async fn module_exists(pool: &PgPool, module_id: Uuid) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM learning_modules WHERE id=$1)",
        module_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false))
}

pub async fn create_submaterial(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateSubmaterialRequest,
    order_index: i32,
) -> anyhow::Result<Uuid> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO submaterials (module_id, title, content, order_index, estimated_minutes)
        VALUES ($1,$2,$3,$4,$5) RETURNING id
        "#,
        req.module_id,
        req.title,
        req.content as _,
        order_index,
        req.estimated_minutes.unwrap_or(10),
    )
    .fetch_one(&mut **tx)
    .await
    .context("create_submaterial")?;
    Ok(id)
}

pub async fn update_submaterial(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    req: &UpdateSubmaterialRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE submaterials SET
            title             = COALESCE($2, title),
            content           = COALESCE($3, content),
            order_index       = COALESCE($4, order_index),
            estimated_minutes = COALESCE($5, estimated_minutes),
            is_published      = COALESCE($6, is_published),
            updated_at        = NOW()
        WHERE id = $1
        "#,
        id,
        req.title as _,
        req.content as _,
        req.order_index as _,
        req.estimated_minutes as _,
        req.is_published as _,
    )
    .execute(&mut **tx)
    .await
    .context("update_submaterial")?;
    Ok(())
}

pub async fn set_submaterial_published(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    published: bool,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE submaterials SET is_published=$2, updated_at=NOW() WHERE id=$1",
        id, published
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn hard_delete_submaterial(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM submaterials WHERE id=$1", id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub async fn count_progress_for_submaterial(pool: &PgPool, id: Uuid) -> anyhow::Result<i64> {
    Ok(sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_submaterial_progress WHERE submaterial_id=$1",
        id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}
