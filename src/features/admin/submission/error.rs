use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubmissionError {
    #[error("submission tidak ditemukan")]
    NotFound,

    #[error("submission sudah di-approve sebelumnya")]
    AlreadyApproved,

    #[error("submission sudah di-reject sebelumnya")]
    AlreadyRejected,

    /// Attempted to transition to a status that is not reachable from the current one
    /// (e.g., approved -> pending_review).
    #[error("transisi status tidak valid: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Input validation failure (notes too short/long/empty) -> 400.
    #[error("{0}")]
    ValidationFailed(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<SubmissionError> for AppError {
    fn from(e: SubmissionError) -> Self {
        match e {
            SubmissionError::NotFound => AppError::NotFound(e.to_string()),
            SubmissionError::AlreadyApproved | SubmissionError::AlreadyRejected => {
                AppError::Conflict(e.to_string())
            }
            SubmissionError::InvalidStateTransition { .. } => {
                AppError::Validation(e.to_string())
            }
            SubmissionError::ValidationFailed(msg) => AppError::BadRequest(msg),
            SubmissionError::Database(inner) => AppError::Internal(inner.into()),
            SubmissionError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
