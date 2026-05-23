use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        project::{
            dto::{AdminProjectDto, AdminProjectFilter, AdminProjectListResponse, CreateProjectRequest, UpdateProjectRequest},
            repository,
        },
    },
};

pub async fn list_projects(pool: &sqlx::PgPool, f: &AdminProjectFilter) -> Result<AdminProjectListResponse, ContentError> {
    repository::list_projects(pool, f).await.map_err(ContentError::Internal)
}

pub async fn get_project(pool: &sqlx::PgPool, id: Uuid) -> Result<AdminProjectDto, ContentError> {
    repository::find_project(pool, id).await.map_err(ContentError::Internal)?.ok_or(ContentError::NotFound)
}

pub async fn create_project(pool: &sqlx::PgPool, admin_id: Uuid, req: &CreateProjectRequest) -> Result<Uuid, ContentError> {
    if repository::role_has_project(pool, req.role_id, None).await.map_err(ContentError::Internal)? {
        return Err(ContentError::Conflict(format!("role sudah memiliki satu project (constraint UNIQUE)")));
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    let id = repository::create_project(&mut tx, req).await.map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "project.created", "project", Some(id),
        Some(json!({ "title": req.title, "role_id": req.role_id })))
        .await.map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(id)
}

pub async fn update_project(pool: &sqlx::PgPool, admin_id: Uuid, id: Uuid, req: &UpdateProjectRequest) -> Result<(), ContentError> {
    if req.title.is_none() && req.description.is_none() && req.requirements.is_none()
        && req.estimated_hours.is_none() && req.is_published.is_none()
    {
        return Err(ContentError::EmptyUpdate);
    }

    let _ = repository::find_project(pool, id).await.map_err(ContentError::Internal)?.ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::update_project(&mut tx, id, req).await.map_err(ContentError::Internal)?;

    let action = if req.is_published == Some(false) { "project.unpublished" }
                 else if req.is_published == Some(true) { "project.published" }
                 else { "project.updated" };

    audit_svc::log(&mut tx, admin_id, action, "project", Some(id),
        Some(json!({ "title": req.title, "is_published": req.is_published })))
        .await.map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn unpublish_project(pool: &sqlx::PgPool, admin_id: Uuid, id: Uuid) -> Result<(), ContentError> {
    let _ = repository::find_project(pool, id).await.map_err(ContentError::Internal)?.ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_project_published(&mut tx, id, false).await.map_err(ContentError::Internal)?;
    audit_svc::log(&mut tx, admin_id, "project.unpublished", "project", Some(id), None).await.map_err(ContentError::Internal)?;
    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn restore_project(pool: &sqlx::PgPool, admin_id: Uuid, id: Uuid) -> Result<(), ContentError> {
    let _ = repository::find_project(pool, id).await.map_err(ContentError::Internal)?.ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_project_published(&mut tx, id, true).await.map_err(ContentError::Internal)?;
    audit_svc::log(&mut tx, admin_id, "project.published", "project", Some(id), None).await.map_err(ContentError::Internal)?;
    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}
