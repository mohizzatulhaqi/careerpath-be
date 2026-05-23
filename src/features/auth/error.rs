use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Email already registered")]
    EmailAlreadyExists,

    #[error("Invalid email or password")]
    InvalidCredentials,

    /// Returned when the account has been soft-deleted by an admin.
    /// Access tokens already issued remain valid until expiry (Pilihan A — see middleware/auth.rs).
    /// Refresh is denied immediately.
    #[error("Akun kamu telah dinonaktifkan, hubungi admin")]
    AccountDeactivated,

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Refresh token has expired")]
    RefreshTokenExpired,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<AuthError> for AppError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::EmailAlreadyExists => AppError::Conflict(e.to_string()),
            AuthError::InvalidCredentials => AppError::Unauthorized(e.to_string()),
            AuthError::AccountDeactivated => {
                // Special case: needs code "ACCOUNT_DEACTIVATED" in the JSON body
                // We encode it directly via a custom Unauthorized variant.
                // AppError::Unauthorized maps to 401; the code is included by the handler.
                AppError::AccountDeactivated(e.to_string())
            }
            AuthError::UserNotFound => AppError::NotFound(e.to_string()),
            AuthError::InvalidRefreshToken => AppError::Unauthorized(e.to_string()),
            AuthError::RefreshTokenExpired => AppError::Unauthorized(e.to_string()),
            AuthError::Database(inner) => AppError::Internal(inner.into()),
            AuthError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
