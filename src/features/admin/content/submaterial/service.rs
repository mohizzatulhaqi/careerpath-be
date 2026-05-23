use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        submaterial::{
            dto::{
                AdminSubmaterialDto, AdminSubmaterialFilter, AdminSubmaterialListResponse,
                CreateSubmaterialRequest, UpdateSubmaterialRequest,
            },
            repository,
        },
    },
};

pub async fn list_submaterials(
    pool: &sqlx::PgPool,
    f: &AdminSubmaterialFilter,
) -> Result<AdminSubmaterialListResponse, ContentError> {
    repository::list_submaterials(pool, f).await.map_err(ContentError::Internal)
}

pub async fn get_submaterial(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> Result<AdminSubmaterialDto, ContentError> {
    repository::find_submaterial(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)
}

pub async fn create_submaterial(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    req: &CreateSubmaterialRequest,
) -> Result<(Uuid, bool), ContentError> {
    if !repository::module_exists(pool, req.module_id)
        .await
        .map_err(ContentError::Internal)?
    {
        return Err(ContentError::ParentNotFound);
    }

    let order_index = match req.order_index {
        Some(idx) => {
            if repository::order_index_conflict(pool, req.module_id, idx, None)
                .await
                .map_err(ContentError::Internal)?
            {
                return Err(ContentError::OrderConflict(idx));
            }
            idx
        }
        None => repository::next_order_index(pool, req.module_id)
            .await
            .map_err(ContentError::Internal)?,
    };

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    let id = repository::create_submaterial(&mut tx, req, order_index)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "submaterial.created",
        "submaterial",
        Some(id),
        Some(json!({ "title": req.title, "module_id": req.module_id })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;

    // requires_mini_quiz reminder: true because newly created sub has no quiz yet
    Ok((id, true))
}

pub async fn update_submaterial(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
    req: &UpdateSubmaterialRequest,
) -> Result<(), ContentError> {
    if req.title.is_none()
        && req.content.is_none()
        && req.order_index.is_none()
        && req.estimated_minutes.is_none()
        && req.is_published.is_none()
    {
        return Err(ContentError::EmptyUpdate);
    }

    let current = repository::find_submaterial(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    if let Some(new_idx) = req.order_index {
        if new_idx != current.order_index
            && repository::order_index_conflict(pool, current.module_id, new_idx, Some(id))
                .await
                .map_err(ContentError::Internal)?
        {
            return Err(ContentError::OrderConflict(new_idx));
        }
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::update_submaterial(&mut tx, id, req)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "submaterial.updated",
        "submaterial",
        Some(id),
        Some(json!({ "title": req.title, "order_index": req.order_index })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn unpublish_submaterial(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _ = repository::find_submaterial(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_submaterial_published(&mut tx, id, false)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "submaterial.unpublished", "submaterial", Some(id), None)
        .await
        .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn restore_submaterial(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _ = repository::find_submaterial(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_submaterial_published(&mut tx, id, true)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "submaterial.published", "submaterial", Some(id), None)
        .await
        .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn hard_delete_submaterial(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
    force: bool,
) -> Result<(), ContentError> {
    let _ = repository::find_submaterial(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let count = repository::count_progress_for_submaterial(pool, id)
        .await
        .map_err(ContentError::Internal)?;

    if count > 0 && !force {
        return Err(ContentError::RequiresForce {
            count,
            message: format!(
                "{count} user sudah menyelesaikan sub-materi ini. Set ?force=true untuk hard delete."
            ),
        });
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::hard_delete_submaterial(&mut tx, id)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "submaterial.deleted",
        "submaterial",
        Some(id),
        Some(json!({ "forced": force, "progress_count": count })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}
