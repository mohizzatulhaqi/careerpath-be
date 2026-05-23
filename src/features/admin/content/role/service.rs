use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_svc,
    content::{
        error::ContentError,
        role::{
            dto::{CreateRoleRequest, RoleDetailDto, RoleListQuery, RoleListResponse, UpdateRoleRequest},
            repository,
        },
    },
};

pub async fn list_roles(
    pool: &sqlx::PgPool,
    q: &RoleListQuery,
) -> Result<RoleListResponse, ContentError> {
    repository::list_roles(pool, q).await.map_err(ContentError::Internal)
}

pub async fn get_role(pool: &sqlx::PgPool, id: Uuid) -> Result<RoleDetailDto, ContentError> {
    repository::find_role(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)
}

fn is_valid_role_code(code: &str) -> bool {
    !code.is_empty() && code.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub async fn create_role(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    req: &CreateRoleRequest,
) -> Result<Uuid, ContentError> {
    if !is_valid_role_code(&req.code) {
        return Err(ContentError::InvalidStructure(
            "code harus lowercase alphanumeric + underscore saja".into(),
        ));
    }

    if repository::code_exists(pool, &req.code, None).await.map_err(ContentError::Internal)? {
        return Err(ContentError::Conflict(format!("role code '{}' sudah ada", req.code)));
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    let id = repository::create_role(&mut tx, req).await.map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "role.created",
        "role",
        Some(id),
        Some(json!({ "code": req.code, "name": req.name })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(id)
}

pub async fn update_role(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
    req: &UpdateRoleRequest,
) -> Result<(), ContentError> {
    if req.name.is_none() && req.description.is_none() {
        return Err(ContentError::EmptyUpdate);
    }

    let _role = repository::find_role(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::update_role(&mut tx, id, req).await.map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "role.updated",
        "role",
        Some(id),
        Some(json!({ "name": req.name, "description": req.description })),
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn deactivate_role(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _role = repository::find_role(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let user_count =
        repository::count_users_with_role(pool, id).await.map_err(ContentError::Internal)?;

    if user_count > 0 {
        return Err(ContentError::RoleInUse { user_count });
    }

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_role_active(&mut tx, id, false).await.map_err(ContentError::Internal)?;

    audit_svc::log(
        &mut tx,
        admin_id,
        "role.deactivated",
        "role",
        Some(id),
        None,
    )
    .await
    .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}

pub async fn restore_role(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    id: Uuid,
) -> Result<(), ContentError> {
    let _role = repository::find_role(pool, id)
        .await
        .map_err(ContentError::Internal)?
        .ok_or(ContentError::NotFound)?;

    let mut tx = pool.begin().await.map_err(ContentError::Database)?;
    repository::set_role_active(&mut tx, id, true).await.map_err(ContentError::Internal)?;

    audit_svc::log(&mut tx, admin_id, "role.restored", "role", Some(id), None)
        .await
        .map_err(ContentError::Internal)?;

    tx.commit().await.map_err(ContentError::Database)?;
    Ok(())
}
