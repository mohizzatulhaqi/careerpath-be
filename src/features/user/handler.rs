use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

use super::dto::{
    ProfileActivityDto, ProfileCareerRoleDto, ProfileCertificateDto, ProfileResponse,
    ProfileStatsDto, ProfileSummaryResponse, UpdateProfileRequest,
};
use crate::{
    error::AppError,
    features::dashboard::service as dashboard_service,
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

    if body.name.is_none() && body.email.is_none() {
        return Err(AppError::BadRequest("provide at least one field to update: name or email".into()));
    }

    let new_name: Option<String> = match &body.name {
        Some(raw) => Some(
            sanitization::sanitize_plain_text(raw, 1, 100)
                .map_err(AppError::from)?,
        ),
        None => None,
    };

    // Check email uniqueness before updating
    if let Some(new_email) = &body.email {
        let taken = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)",
            new_email,
            auth.user_id,
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .unwrap_or(false);

        if taken {
            return Err(AppError::Conflict("email already in use".into()));
        }
    }

    let updated = sqlx::query!(
        r#"
        UPDATE users
        SET
            name       = COALESCE($2, name),
            email      = COALESCE($3, email),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, name, role AS "role: String", is_active, created_at, updated_at
        "#,
        auth.user_id,
        new_name,
        body.email,
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

// ── GET /api/users/me/summary ─────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/users/me/summary",
    tag = "User",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Profile summary for display page", body = ProfileSummaryResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_profile_summary(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Parallel: dashboard data + approved projects count
    let (dashboard_res, approved_res) = tokio::join!(
        dashboard_service::get_full_dashboard(&state.db, auth.user_id, &state.config.app_base_url),
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_submissions WHERE user_id = $1 AND status = 'approved'",
        )
        .bind(auth.user_id)
        .fetch_one(&state.db),
    );

    let dashboard = dashboard_res?;
    let projects_approved = approved_res.map_err(|e| AppError::Internal(e.into()))?;

    let base_url = &state.config.app_base_url;

    let certificates = dashboard
        .certificates
        .into_iter()
        .map(|c| ProfileCertificateDto {
            download_url: format!("{}/api/certificates/me/{}/download.pdf", base_url, c.id),
            id: c.id,
            certificate_code: c.certificate_code,
            role_name: c.role_name,
            issued_at: c.issued_at,
            is_revoked: c.is_revoked,
            verification_url: c.verification_url,
        })
        .collect();

    let recent_activities = dashboard
        .recent_activities
        .into_iter()
        .map(|a| ProfileActivityDto {
            kind: a.kind,
            title: a.title,
            detail: a.detail,
            occurred_at: a.occurred_at,
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": ProfileSummaryResponse {
            id: dashboard.user.id,
            name: dashboard.user.name,
            email: dashboard.user.email,
            member_since: dashboard.user.member_since,
            career_role: dashboard.career_path.role.map(|r| ProfileCareerRoleDto {
                code: r.code,
                name: r.name,
            }),
            stats: ProfileStatsDto {
                modules_completed: dashboard.learning_progress.completed_modules,
                modules_total: dashboard.learning_progress.total_modules,
                projects_approved,
            },
            certificates,
            recent_activities,
        }
    })))
}
