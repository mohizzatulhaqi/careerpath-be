use crate::{error::AppError, middleware::auth::AuthUser, state::AppState};
use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use std::sync::Arc;

/// Extractor that only succeeds for users with role == "admin".
/// Use it instead of `AuthUser` on admin-only handlers.
pub struct AdminUser(pub AuthUser);

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        if auth_user.role != "admin" {
            return Err(AppError::Forbidden("Admin access required".to_string()));
        }

        Ok(AdminUser(auth_user))
    }
}
