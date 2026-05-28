use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CertificateError {
    #[error("certificate not found")]
    NotFound,

    #[error("access denied")]
    NotOwned,

    #[error("certificate is already revoked")]
    AlreadyRevoked,

    #[error("certificate is not revoked")]
    NotRevoked,

    #[error("not eligible: {reason}")]
    NotEligible { reason: String },

    #[error("pdf generation failed: {0}")]
    PdfGeneration(String),

    #[error("qr code generation failed: {0}")]
    QrGeneration(String),

    #[error("failed to generate unique certificate code after retries")]
    CodeGenerationFailed,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<CertificateError> for AppError {
    fn from(e: CertificateError) -> Self {
        match e {
            CertificateError::NotFound => AppError::NotFound(e.to_string()),
            CertificateError::NotOwned => AppError::Forbidden(e.to_string()),
            CertificateError::AlreadyRevoked | CertificateError::NotRevoked => {
                AppError::Conflict(e.to_string())
            }
            CertificateError::NotEligible { .. } => AppError::BadRequest(e.to_string()),
            CertificateError::PdfGeneration(_)
            | CertificateError::QrGeneration(_)
            | CertificateError::CodeGenerationFailed => {
                AppError::Internal(anyhow::anyhow!("{e}"))
            }
            CertificateError::Database(inner) => AppError::Internal(inner.into()),
            CertificateError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
