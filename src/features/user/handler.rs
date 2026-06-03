use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

use super::dto::{ProfileResponse, UpdateProfileRequest};
use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    shared::sanitization,
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

    // name is the only updatable field
    if body.name.is_none() {
        return Err(AppError::BadRequest("provide at least name to update".into()));
    }

    let new_name: Option<String> = match &body.name {
        Some(raw) => Some(
            sanitization::sanitize_plain_text(raw, 1, 100)
                .map_err(AppError::from)?,
        ),
        None => None,
    };

    let updated = sqlx::query!(
        r#"
        UPDATE users
        SET
            name       = COALESCE($2, name),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, name, role AS "role: String", is_active, created_at, updated_at
        "#,
        auth.user_id,
        new_name,
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
