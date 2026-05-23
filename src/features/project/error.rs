use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project not found")]
    NotFound,
    #[error("final project belum tersedia untuk role ini")]
    ProjectNotForUserRole,
    #[error("selesaikan semua modul untuk role kamu dulu")]
    Locked,
    #[error("submission pending review, tidak bisa resubmit")]
    SubmissionPendingReview,
    #[error("submission sudah di-approve, tidak bisa resubmit")]
    SubmissionAlreadyApproved,
    #[error("submission not found")]
    SubmissionNotFound,
    #[error("kamu tidak berhak mengakses submission ini")]
    SubmissionNotOwned,
    #[error("file validation failed: {0}")]
    InvalidFile(String),
    #[error("notes must be between 20 and 5000 characters")]
    NotesLengthInvalid,
    #[error("{0}")]
    NotesInvalid(String),
    #[error("field 'file' wajib")]
    MissingFile,
    #[error("field 'submission_notes' wajib")]
    MissingNotes,
    #[error("storage error: {0}")]
    Storage(String),
    #[error("multipart error: {0}")]
    MultipartError(String),
}

impl From<ProjectError> for AppError {
    fn from(e: ProjectError) -> Self {
        match e {
            ProjectError::NotFound | ProjectError::SubmissionNotFound => {
                AppError::NotFound(e.to_string())
            }
            ProjectError::ProjectNotForUserRole
            | ProjectError::Locked
            | ProjectError::SubmissionNotOwned => AppError::Forbidden(e.to_string()),
            ProjectError::SubmissionPendingReview | ProjectError::SubmissionAlreadyApproved => {
                AppError::Conflict(e.to_string())
            }
            ProjectError::InvalidFile(_)
            | ProjectError::NotesLengthInvalid
            | ProjectError::NotesInvalid(_)
            | ProjectError::MissingFile
            | ProjectError::MissingNotes
            | ProjectError::MultipartError(_) => AppError::BadRequest(e.to_string()),
            ProjectError::Storage(err) => AppError::Internal(anyhow::anyhow!("Storage error: {}", err)),
        }
    }
}

impl From<crate::features::project::storage::StorageError> for ProjectError {
    fn from(e: crate::features::project::storage::StorageError) -> Self {
        ProjectError::Storage(e.to_string())
    }
}
