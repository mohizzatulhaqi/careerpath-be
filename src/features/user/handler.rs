use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

use super::dto::{ProfileResponse, UpdateProfileRequest};
use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    shared::{password, sanitization},
    state::AppState,
};

// ── GET /api/users/me ─────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "User",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_profile(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        "SELECT id, email, name, role AS \"role: String\", is_active, created_at, updated_at
         FROM users WHERE id = $1",
        auth.user_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    Ok(Json(json!({
        "success": true,
        "data": ProfileResponse {
            id: row.id,
            email: row.email,
            name: row.name,
            role: row.role,
            is_active: row.is_active,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    })))
}

// ── PATCH /api/users/me ───────────────────────────────────────────────────────

#[utoipa::path(
    patch,
    path = "/api/users/me",
    tag = "User",
    security(("bearer_auth" = [])),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = ProfileResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Wrong current password"),
    )
)]
pub async fn update_profile(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate field constraints
    body.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // At least one field must be provided
    if body.name.is_none() && body.new_password.is_none() {
        return Err(AppError::BadRequest(
            "provide at least one field to update: name or new_password".into(),
        ));
    }

    // If changing password, current_password is required
    if body.new_password.is_some() && body.current_password.is_none() {
        return Err(AppError::BadRequest(
            "current_password is required to change password".into(),
        ));
    }

    // Fetch current user record
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE id = $1",
        auth.user_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    // Resolve new values
    let new_name: Option<String> = match &body.name {
        Some(raw) => Some(
            sanitization::sanitize_plain_text(raw, 1, 100)
                .map_err(AppError::from)?,
        ),
        None => None,
    };

    let new_password_hash: Option<String> = match &body.new_password {
        Some(new_pw) => {
            // Verify current password first
            let current_pw = body.current_password.as_deref().unwrap_or("");
            let valid = password::verify_password(current_pw, &user.password_hash)
                .map_err(|e| AppError::Internal(e))?;
            if !valid {
                return Err(AppError::Conflict("current_password is incorrect".into()));
            }
            let hash = password::hash_password(new_pw)
                .map_err(|e| AppError::Internal(e))?;
            Some(hash)
        }
        None => None,
    };

    // Apply update — only touch columns that changed
    let updated = sqlx::query!(
        r#"
        UPDATE users
        SET
            name          = COALESCE($2, name),
            password_hash = COALESCE($3, password_hash),
            updated_at    = NOW()
        WHERE id = $1
        RETURNING id, email, name, role AS "role: String", is_active, created_at, updated_at
        "#,
        auth.user_id,
        new_name,
        new_password_hash,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(json!({
        "success": true,
        "data": ProfileResponse {
            id: updated.id,
            email: updated.email,
            name: updated.name,
            role: updated.role,
            is_active: updated.is_active,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        }
    })))
}
