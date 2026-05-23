use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("resource tidak ditemukan")]
    NotFound,

    #[error("role masih digunakan oleh {user_count} user — nonaktifkan role dulu")]
    RoleInUse { user_count: i64 },

    #[error("parent resource tidak ditemukan")]
    ParentNotFound,

    #[error("parent resource tidak aktif/dipublish — aktifkan dulu sebelum menambah child")]
    ParentInactive,

    #[error("order_index {0} sudah digunakan di scope yang sama")]
    OrderConflict(i32),

    #[error("struktur tidak valid: {0}")]
    InvalidStructure(String),

    #[error("tidak ada field yang diupdate")]
    EmptyUpdate,

    #[error("{0}")]
    Conflict(String),

    #[error("operasi ini mempengaruhi {count} data user — set ?force=true untuk konfirmasi: {message}")]
    RequiresForce { count: i64, message: String },

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<ContentError> for AppError {
    fn from(e: ContentError) -> Self {
        match e {
            ContentError::NotFound | ContentError::ParentNotFound => {
                AppError::NotFound(e.to_string())
            }
            ContentError::RoleInUse { .. } | ContentError::Conflict(_) => AppError::Conflict(e.to_string()),
            ContentError::ParentInactive => AppError::BadRequest(e.to_string()),
            ContentError::OrderConflict(_) => AppError::Conflict(e.to_string()),
            ContentError::InvalidStructure(_) | ContentError::EmptyUpdate => {
                AppError::BadRequest(e.to_string())
            }
            ContentError::RequiresForce { count, message } => {
                AppError::RequiresForce { message, affected_count: count }
            }
            ContentError::Database(inner) => AppError::Internal(inner.into()),
            ContentError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
