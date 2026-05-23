use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LearningError {
    #[error("Selesaikan quiz penentuan role dulu")]
    QuizNotCompleted,

    #[error("Modul tidak ditemukan")]
    ModuleNotFound,

    #[error("Modul ini bukan untuk role kamu")]
    ModuleNotForUserRole,

    #[error("Selesaikan modul sebelumnya dulu")]
    ModuleLocked,

    #[error("Sub materi tidak ditemukan")]
    SubmaterialNotFound,

    #[error("Sub materi ini bukan untuk role kamu")]
    SubmaterialNotForUserRole,

    #[error("Selesaikan sub materi sebelumnya dulu")]
    SubmaterialLocked,

    #[error("Tidak ada soal quiz untuk materi ini")]
    QuizNotFound,

    #[error("Jawaban tidak lengkap — semua soal harus dijawab")]
    IncompleteAnswers,

    #[error("Pilihan jawaban tidak valid untuk soal ini")]
    InvalidOption,

    #[error("Selesaikan semua sub materi dulu sebelum mengerjakan final quiz")]
    FinalQuizLocked,

    #[error("Endpoint ini sudah dihapus — gunakan mini quiz untuk menyelesaikan sub materi")]
    SubmaterialCompletionDeprecated,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<LearningError> for AppError {
    fn from(e: LearningError) -> Self {
        match e {
            LearningError::QuizNotCompleted => AppError::Validation(e.to_string()),
            LearningError::ModuleNotFound | LearningError::SubmaterialNotFound | LearningError::QuizNotFound => {
                AppError::NotFound(e.to_string())
            }
            LearningError::ModuleNotForUserRole
            | LearningError::ModuleLocked
            | LearningError::SubmaterialNotForUserRole
            | LearningError::SubmaterialLocked
            | LearningError::FinalQuizLocked => AppError::Forbidden(e.to_string()),
            LearningError::IncompleteAnswers | LearningError::InvalidOption => {
                AppError::BadRequest(e.to_string())
            }
            LearningError::SubmaterialCompletionDeprecated => AppError::Gone(e.to_string()),
            LearningError::Database(inner) => AppError::Internal(inner.into()),
            LearningError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
