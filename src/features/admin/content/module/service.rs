use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        module::{
            dto::{
                AdminModuleDto, AdminModuleFilter, AdminModuleListResponse, CreateModuleRequest,
                UpdateModuleRequest,
            },
            repository,
        },
    },
};

pub async fn list_modules(
    pool: &sqlx::PgPool,
    f: &AdminModuleFilter,
) -> Result<AdminModuleListResponse, ContentError> {
    repository::list_modules(pool, f).await.map_err(ContentError::Internal)
}

pub async fn get_module(pool: &sqlx::PgPool, id: Uuid) -> Result<AdminModuleDto, ContentError> {
    repository::find_module(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)
}

pub async fn create_module(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    req: &CreateModuleRequest,
) -> Result<Uuid, ContentError> {
    // Validate parent role
    if !repository::role_exists_and_active(pool, req.role_id)
        .await
        .map_err(ContentError::Internal)?
    {
        return Err(ContentError::ParentNotFound);
    }

    // Resolve order_index
    let order_index = match req.order_index {
        Some(idx) => {
            if repository::order_index_conflict(pool, req.role_id, idx, None)
                .await
                .map_err(ContentError::Internal)?
            {
                return Err(ContentError::OrderConflict(idx));
            }
            idx
        }
        None => repository::next_order_index(pool, req.role_id)
            .await
            .map_err(ContentError::Internal)?,
    };

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    let id = repository::create_module(&mut tx, req, order_index)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "module.created",
        "module",
        Some(id),
        Some(json!({ "title": req.title, "role_id": req.role_id, "order_index": order_index })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(id)
}

pub async fn update_module(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
    req: &UpdateModuleRequest,
) -> Result<(), ContentError> {
    if req.title.is_none()
        && req.description.is_none()
        && req.order_index.is_none()
        && req.is_published.is_none()
    {
        return Err(ContentError::EmptyUpdate);
    }

    let current = repository::find_module(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    // Check order_index conflict if changed
    if let Some(new_idx) = req.order_index {
        if new_idx != current.order_index
            && repository::order_index_conflict(pool, current.role_id, new_idx, Some(id))
                .await
                .map_err(ContentError::Internal)?
        {
            return Err(ContentError::OrderConflict(new_idx));
        }
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::update_module(&mut tx, id, req).await.map_err(ContentError::Internal)?;

    let action = if req.is_published == Some(false) {
        "module.unpublished"
    } else if req.is_published == Some(true) {
        "module.published"
    } else {
        "module.updated"
    };

    audit_svc::log(
        &mut tx,
        admin_id,
        action,
        "module",
        Some(id),
        Some(json!({
            "title": req.title,
            "order_index": req.order_index,
            "is_published": req.is_published
        })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn unpublish_module(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<i64, ContentError> {
    let _ = repository::find_module(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let affected = repository::count_affected_users(pool, id)
        .await
        .map_err(ContentError::Internal)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_module_published(&mut tx, id, false)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "module.unpublished",
        "module",
        Some(id),
        Some(json!({ "affected_users": affected })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(affected)
}

pub async fn restore_module(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _ = repository::find_module(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_module_published(&mut tx, id, true)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "module.published", "module", Some(id), None)
        .await
        .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn hard_delete_module(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
    force: bool,
) -> Result<(), ContentError> {
    let _ = repository::find_module(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let affected = repository::count_affected_users(pool, id)
        .await
        .map_err(ContentError::Internal)?;

    if affected > 0 && !force {
        return Err(ContentError::RequiresForce {
            count: affected,
            message: format!(
                "modul ini memiliki {affected} user yang sudah mulai belajar. \
                Set ?force=true untuk hard delete dan menghapus semua progress."
            ),
        });
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::hard_delete_module(&mut tx, id)
        .await
        .map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "module.deleted",
        "module",
        Some(id),
        Some(json!({ "forced": true, "affected_users": affected })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}
