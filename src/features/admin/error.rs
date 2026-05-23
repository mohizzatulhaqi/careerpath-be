use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("tidak bisa mengubah atau menonaktifkan akun sendiri")]
    CannotModifySelf,

    #[error("user tidak ditemukan")]
    UserNotFound,

    #[error("user sudah dalam status non-aktif")]
    UserAlreadyDeactivated,

    #[error("user sudah dalam status aktif")]
    UserAlreadyActive,

    #[error("tidak ada field yang perlu diupdate")]
    EmptyUpdate,

    #[error("role tidak valid — harus 'user' atau 'admin'")]
    InvalidRole,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<AdminError> for AppError {
    fn from(e: AdminError) -> Self {
        match e {
            AdminError::CannotModifySelf | AdminError::EmptyUpdate | AdminError::InvalidRole => {
                AppError::BadRequest(e.to_string())
            }
            AdminError::UserNotFound => AppError::NotFound(e.to_string()),
            AdminError::UserAlreadyDeactivated | AdminError::UserAlreadyActive => {
                AppError::Conflict(e.to_string())
            }
            AdminError::Database(inner) => AppError::Internal(inner.into()),
            AdminError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
