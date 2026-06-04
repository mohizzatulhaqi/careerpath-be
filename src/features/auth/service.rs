use crate::{
    features::auth::{
        dto::{AuthResponse, ChangePasswordRequest, LoginRequest, RegisterRequest, ResetPasswordRequest, TokenResponse, UserResponse},
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
    let family_id = Uuid::new_v4();
    let expires_at =
        Utc::now() + chrono::Duration::seconds(state.config.refresh_token_expires_in as i64);

    repository::create_refresh_token(&state.db, user_id, family_id, &refresh_token, expires_at)
        .await?;

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
///
/// Theft detection: each token is marked `used` after rotation rather than deleted.
/// If a used token is replayed, the entire token family is revoked (all devices
/// in that session chain are forced to log in again).
pub async fn refresh(
    state: &Arc<AppState>,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    let stored = repository::find_refresh_token(&state.db, refresh_token)
        .await?
        .ok_or(AuthError::InvalidRefreshToken)?;

    // Replay of a previously-rotated token — possible token theft.
    // Revoke the entire family to protect the legitimate user.
    if stored.used {
        tracing::warn!(
            family_id = %stored.family_id,
            user_id = %stored.user_id,
            "Refresh token replay detected — revoking token family (possible theft)"
        );
        repository::delete_token_family(&state.db, stored.family_id).await?;
        return Err(AuthError::InvalidRefreshToken);
    }

    if stored.expires_at < Utc::now() {
        repository::delete_refresh_token(&state.db, refresh_token).await?;
        return Err(AuthError::RefreshTokenExpired);
    }

    let user = repository::find_by_id(&state.db, stored.user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    if !user.is_active {
        repository::delete_token_family(&state.db, stored.family_id).await?;
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

    repository::rotate_refresh_token(
        &state.db,
        refresh_token,
        user.id,
        stored.family_id,
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

pub async fn change_password(
    state: &Arc<AppState>,
    user_id: Uuid,
    req: ChangePasswordRequest,
) -> Result<(), AuthError> {
    let user = repository::find_by_id(&state.db, user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    if !password::verify_password(&req.old_password, &user.password_hash)? {
        return Err(AuthError::WrongPassword);
    }

    let new_hash = password::hash_password(&req.new_password)?;
    repository::update_password(&state.db, user_id, &new_hash).await?;

    Ok(())
}

pub async fn reset_password(
    state: &Arc<AppState>,
    req: ResetPasswordRequest,
) -> Result<(), AuthError> {
    let user = repository::find_by_email(&state.db, &req.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !password::verify_password(&req.old_password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    let new_hash = password::hash_password(&req.new_password)?;
    repository::update_password(&state.db, user.id, &new_hash).await?;

    Ok(())
}

pub async fn logout(state: &Arc<AppState>, refresh_token: &str) -> Result<(), AuthError> {
    repository::delete_refresh_token(&state.db, refresh_token).await?;
    Ok(())
}
