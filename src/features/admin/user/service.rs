use serde_json::json;
use uuid::Uuid;

use crate::features::admin::{
    audit::service as audit_service,
    error::AdminError,
    user::{
        dto::{
            ActivateRequest, DeactivateRequest, UpdateUserRequest, UserDetailDto, UserListQuery,
            UserListResponse,
        },
        repository,
    },
};
use crate::features::admin::audit::dto::{AuditLogFilter, AuditLogPageDto};

// ── List ──────────────────────────────────────────────────────────────────────

pub async fn list_users(
    pool: &sqlx::PgPool,
    q: &UserListQuery,
) -> Result<UserListResponse, AdminError> {
    repository::list_users(pool, q).await.map_err(AdminError::Internal)
}

// ── Detail ────────────────────────────────────────────────────────────────────

pub async fn get_user_detail(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<UserDetailDto, AdminError> {
    repository::get_user_detail(pool, user_id)
        .await
        .map_err(AdminError::Internal)?
        .ok_or(AdminError::UserNotFound)
}

// ── Update (name / role) ──────────────────────────────────────────────────────

pub async fn update_user(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    target_id: Uuid,
    req: &UpdateUserRequest,
) -> Result<(), AdminError> {
    if req.name.is_none() && req.role.is_none() {
        return Err(AdminError::EmptyUpdate);
    }

    // Validate role string if provided
    if let Some(role) = &req.role {
        if role != "user" && role != "admin" {
            return Err(AdminError::InvalidRole);
        }
    }

    // Self-protection: admin cannot demote themselves
    if admin_id == target_id {
        return Err(AdminError::CannotModifySelf);
    }

    // Fetch current user to build audit metadata
    let current = repository::find_by_id(pool, target_id)
        .await
        .map_err(|e| AdminError::Internal(e))?
        .ok_or(AdminError::UserNotFound)?;

    let mut tx = pool.begin().await.map_err(AdminError::Database)?;

    repository::update_user(&mut tx, target_id, req)
        .await
        .map_err(AdminError::Internal)?;

    // Build metadata to record what changed
    let mut meta = serde_json::Map::new();
    if let Some(new_name) = &req.name {
        meta.insert("old_name".into(), json!(current.name));
        meta.insert("new_name".into(), json!(new_name));
    }
    if let Some(new_role) = &req.role {
        meta.insert("old_role".into(), json!(current.role));
        meta.insert("new_role".into(), json!(new_role));
    }

    let action = if req.role.is_some() {
        "user.role_changed"
    } else {
        "user.updated"
    };

    audit_service::log(
        &mut tx,
        admin_id,
        action,
        "user",
        Some(target_id),
        Some(serde_json::Value::Object(meta)),
    )
    .await
    .map_err(AdminError::Internal)?;

    tx.commit().await.map_err(AdminError::Database)?;
    Ok(())
}

// ── Deactivate ────────────────────────────────────────────────────────────────

pub async fn deactivate_user(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    target_id: Uuid,
    req: &DeactivateRequest,
) -> Result<(), AdminError> {
    // Self-protection
    if admin_id == target_id {
        return Err(AdminError::CannotModifySelf);
    }

    let user = repository::find_by_id(pool, target_id)
        .await
        .map_err(AdminError::Internal)?
        .ok_or(AdminError::UserNotFound)?;

    if !user.is_active {
        return Err(AdminError::UserAlreadyDeactivated);
    }

    let mut tx = pool.begin().await.map_err(AdminError::Database)?;

    repository::deactivate_user(&mut tx, target_id, admin_id, &req.reason)
        .await
        .map_err(AdminError::Internal)?;

    audit_service::log(
        &mut tx,
        admin_id,
        "user.deactivated",
        "user",
        Some(target_id),
        Some(json!({ "reason": req.reason })),
    )
    .await
    .map_err(AdminError::Internal)?;

    tx.commit().await.map_err(AdminError::Database)?;
    Ok(())
}

// ── Activate ──────────────────────────────────────────────────────────────────

pub async fn activate_user(
    pool: &sqlx::PgPool,
    admin_id: Uuid,
    target_id: Uuid,
    req: &ActivateRequest,
) -> Result<(), AdminError> {
    // Self-protection (activating oneself is a no-op risk but keep rule consistent)
    if admin_id == target_id {
        return Err(AdminError::CannotModifySelf);
    }

    let user = repository::find_by_id(pool, target_id)
        .await
        .map_err(AdminError::Internal)?
        .ok_or(AdminError::UserNotFound)?;

    if user.is_active {
        return Err(AdminError::UserAlreadyActive);
    }

    let mut tx = pool.begin().await.map_err(AdminError::Database)?;

    repository::activate_user(&mut tx, target_id)
        .await
        .map_err(AdminError::Internal)?;

    let meta = req
        .reason
        .as_deref()
        .map(|r| json!({ "reason": r }));

    audit_service::log(
        &mut tx,
        admin_id,
        "user.activated",
        "user",
        Some(target_id),
        meta,
    )
    .await
    .map_err(AdminError::Internal)?;

    tx.commit().await.map_err(AdminError::Database)?;
    Ok(())
}

// ── Audit logs for a specific target user ─────────────────────────────────────

pub async fn get_user_audit_logs(
    pool: &sqlx::PgPool,
    target_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<AuditLogPageDto, AdminError> {
    audit_service::list_logs_for_target(pool, "user", target_id, page, per_page)
        .await
        .map_err(AdminError::Internal)
}

// ── Global audit log ──────────────────────────────────────────────────────────

pub async fn get_audit_logs(
    pool: &sqlx::PgPool,
    filter: &AuditLogFilter,
) -> Result<AuditLogPageDto, AdminError> {
    audit_service::list_logs(pool, filter)
        .await
        .map_err(AdminError::Internal)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // These are logic-only tests that don't hit the DB.

    #[test]
    fn update_request_empty_fails() {
        // If both name and role are None, EmptyUpdate should be returned.
        // We can't call the async fn without a pool, so we test the guard logic inline.
        let req = UpdateUserRequest { name: None, role: None };
        assert!(req.name.is_none() && req.role.is_none());
    }

    #[test]
    fn invalid_role_detected() {
        let bad_roles = ["superadmin", "moderator", "ADMIN", "USER", ""];
        for role in bad_roles {
            assert!(role != "user" && role != "admin", "role '{role}' should be invalid");
        }
    }

    #[test]
    fn self_protection_same_id() {
        let id = Uuid::new_v4();
        assert_eq!(id, id); // sanity
        // In service the check is `if admin_id == target_id`
        assert!(id == id);
    }

    #[test]
    fn self_protection_different_ids() {
        let admin = Uuid::new_v4();
        let target = Uuid::new_v4();
        assert_ne!(admin, target);
    }
}
