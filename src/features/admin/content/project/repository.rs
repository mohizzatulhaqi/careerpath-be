use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::features::admin::{
    audit::dto::PaginationDto,
    content::project::dto::{
        AdminProjectDto, AdminProjectFilter, AdminProjectListResponse, CreateProjectRequest,
        UpdateProjectRequest,
    },
};

pub async fn list_projects(
    pool: &PgPool,
    f: &AdminProjectFilter,
) -> anyhow::Result<AdminProjectListResponse> {
    let page = f.page.unwrap_or(1).max(1);
    let per_page = f.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM projects WHERE ($1::uuid IS NULL OR role_id=$1) AND ($2::bool IS NULL OR is_published=$2)",
        f.role_id as _, f.is_published as _
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    let rows = sqlx::query!(
        r#"
        SELECT p.id, p.role_id, p.title, p.description, p.requirements,
               p.estimated_hours, p.is_published, p.created_at, p.updated_at,
               COUNT(ps.id) AS "submission_count!: i64"
        FROM projects p
        LEFT JOIN project_submissions ps ON ps.project_id = p.id
        WHERE ($1::uuid IS NULL OR p.role_id=$1) AND ($2::bool IS NULL OR p.is_published=$2)
        GROUP BY p.id
        ORDER BY p.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        f.role_id as _, f.is_published as _, per_page, offset
    )
    .fetch_all(pool)
    .await
    .context("list_projects")?;

    let total_pages = (total + per_page - 1) / per_page;

    Ok(AdminProjectListResponse {
        projects: rows
            .into_iter()
            .map(|r| AdminProjectDto {
                id: r.id,
                role_id: r.role_id,
                title: r.title,
                description: r.description,
                requirements: r.requirements,
                estimated_hours: r.estimated_hours,
                is_published: r.is_published,
                created_at: r.created_at,
                updated_at: r.updated_at,
                submission_count: r.submission_count,
            })
            .collect(),
        pagination: PaginationDto { page, per_page, total_items: total, total_pages },
    })
}

pub async fn find_project(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<AdminProjectDto>> {
    let row = sqlx::query!(
        r#"
        SELECT p.id, p.role_id, p.title, p.description, p.requirements,
               p.estimated_hours, p.is_published, p.created_at, p.updated_at,
               COUNT(ps.id) AS "submission_count!: i64"
        FROM projects p
        LEFT JOIN project_submissions ps ON ps.project_id = p.id
        WHERE p.id=$1
        GROUP BY p.id
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .context("find_project")?;

    Ok(row.map(|r| AdminProjectDto {
        id: r.id,
        role_id: r.role_id,
        title: r.title,
        description: r.description,
        requirements: r.requirements,
        estimated_hours: r.estimated_hours,
        is_published: r.is_published,
        created_at: r.created_at,
        updated_at: r.updated_at,
        submission_count: r.submission_count,
    }))
}

pub async fn role_has_project(pool: &PgPool, role_id: Uuid, exclude_id: Option<Uuid>) -> anyhow::Result<bool> {
    let exists = match exclude_id {
        None => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE role_id=$1)",
            role_id
        ).fetch_one(pool).await?.unwrap_or(false),
        Some(eid) => sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE role_id=$1 AND id!=$2)",
            role_id, eid
        ).fetch_one(pool).await?.unwrap_or(false),
    };
    Ok(exists)
}

pub async fn create_project(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateProjectRequest,
) -> anyhow::Result<Uuid> {
    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO projects (role_id, title, description, requirements, estimated_hours)
        VALUES ($1,$2,$3,$4,$5) RETURNING id
        "#,
        req.role_id, req.title, req.description, req.requirements as _, req.estimated_hours.unwrap_or(20)
    )
    .fetch_one(&mut **tx)
    .await
    .context("create_project")?;
    Ok(id)
}

pub async fn update_project(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    req: &UpdateProjectRequest,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE projects SET
            title           = COALESCE($2, title),
            description     = COALESCE($3, description),
            requirements    = COALESCE($4, requirements),
            estimated_hours = COALESCE($5, estimated_hours),
            is_published    = COALESCE($6, is_published),
            updated_at      = NOW()
        WHERE id=$1
        "#,
        id, req.title as _, req.description as _, req.requirements as _,
        req.estimated_hours as _, req.is_published as _
    )
    .execute(&mut **tx)
    .await
    .context("update_project")?;
    Ok(())
}

pub async fn set_project_published(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    published: bool,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE projects SET is_published=$2, updated_at=NOW() WHERE id=$1",
        id, published
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
