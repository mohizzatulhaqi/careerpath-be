use crate::{
    features::auth::{
        dto::{AuthResponse, LoginRequest, RegisterRequest, TokenResponse, UserResponse},
        error::AuthError,
        repository,
    },
    shared::{jwt, password},
    state::AppState,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn new_refresh_token() -> String {
    Uuid::new_v4().to_string()
}

async fn issue_tokens(
    state: &Arc<AppState>,
    user_id: Uuid,
    role: &str,
) -> Result<(String, String), AuthError> {
    let access_token = jwt::create_token(
        user_id,
        role,
        &state.config.jwt_secret,
        state.config.jwt_expires_in,
    )?;

    let refresh_token = new_refresh_token();
    let expires_at =
        Utc::now() + chrono::Duration::seconds(state.config.refresh_token_expires_in as i64);

    repository::create_refresh_token(&state.db, user_id, &refresh_token, expires_at).await?;

    Ok((access_token, refresh_token))
}

// ── Public service functions ──────────────────────────────────────────────────

pub async fn register(
    state: &Arc<AppState>,
    req: RegisterRequest,
) -> Result<AuthResponse, AuthError> {
    if repository::find_by_email(&state.db, &req.email).await?.is_some() {
        return Err(AuthError::EmailAlreadyExists);
    }

    let hash = password::hash_password(&req.password)?;
    let id = Uuid::new_v4();
    let user = repository::create(&state.db, id, &req.email, &hash, &req.name).await?;

    let (access_token, refresh_token) =
        issue_tokens(state, user.id, &user.role.to_string()).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expires_in,
        user: UserResponse::from(user),
    })
}

pub async fn login(
    state: &Arc<AppState>,
    req: LoginRequest,
) -> Result<AuthResponse, AuthError> {
    let user = repository::find_by_email(&state.db, &req.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !password::verify_password(&req.password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    // Check account is active AFTER verifying credentials (avoids timing-based enumeration)
    if !user.is_active {
        return Err(AuthError::AccountDeactivated);
    }

    let (access_token, refresh_token) =
        issue_tokens(state, user.id, &user.role.to_string()).await?;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expires_in,
        user: UserResponse::from(user),
    })
}

pub async fn me(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<UserResponse, AuthError> {
    let user = repository::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    Ok(UserResponse::from(user))
}

/// Exchange a valid refresh token for a new access token + rotated refresh token.
pub async fn refresh(
    state: &Arc<AppState>,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    let stored = repository::find_refresh_token(&state.db, refresh_token)
        .await?
        .ok_or(AuthError::InvalidRefreshToken)?;

    if stored.expires_at < Utc::now() {
        // Expired — clean it up and reject
        repository::delete_refresh_token(&state.db, refresh_token).await?;
        return Err(AuthError::RefreshTokenExpired);
    }

    let user = repository::find_by_id(&state.db, stored.user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // Deny refresh if account was deactivated after the refresh token was issued.
    // Access token (already issued, short-lived) remains valid until expiry — Pilihan A.
    if !user.is_active {
        repository::delete_refresh_token(&state.db, refresh_token).await?;
        return Err(AuthError::AccountDeactivated);
    }

    let new_access = jwt::create_token(
        user.id,
        &user.role.to_string(),
        &state.config.jwt_secret,
        state.config.jwt_expires_in,
    )?;

    let new_refresh = new_refresh_token();
    let expires_at =
        Utc::now() + chrono::Duration::seconds(state.config.refresh_token_expires_in as i64);

    // Atomic swap: old token out, new token in
    repository::rotate_refresh_token(
        &state.db,
        refresh_token,
        user.id,
        &new_refresh,
        expires_at,
    )
    .await?;

    Ok(TokenResponse {
        access_token: new_access,
        refresh_token: new_refresh,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expires_in,
    })
}

/// Invalidate a refresh token (logout from current device).
pub async fn logout(state: &Arc<AppState>, refresh_token: &str) -> Result<(), AuthError> {
    // Silently succeed even if token doesn't exist (idempotent)
    repository::delete_refresh_token(&state.db, refresh_token).await?;
    Ok(())
}
