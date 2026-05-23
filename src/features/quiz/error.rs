use crate::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuizError {
    #[error("Quiz attempt not found")]
    AttemptNotFound,

    #[error("You do not own this attempt")]
    AttemptNotOwned,

    #[error("This attempt has already been submitted")]
    AttemptAlreadySubmitted,

    #[error("This attempt has not been submitted yet")]
    AttemptNotSubmitted,

    #[error("Quiz incomplete: {answered} of {required} questions answered")]
    IncompleteAnswers { answered: usize, required: usize },

    #[error("The selected option does not belong to the given question")]
    InvalidOption,

    #[error("Question not found")]
    QuestionNotFound,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<QuizError> for AppError {
    fn from(e: QuizError) -> Self {
        match e {
            QuizError::AttemptNotFound | QuizError::QuestionNotFound => {
                AppError::NotFound(e.to_string())
            }
            QuizError::AttemptNotOwned => AppError::Forbidden(e.to_string()),
            QuizError::AttemptAlreadySubmitted | QuizError::AttemptNotSubmitted => {
                AppError::Conflict(e.to_string())
            }
            QuizError::IncompleteAnswers { .. } | QuizError::InvalidOption => {
                AppError::Validation(e.to_string())
            }
            QuizError::Database(inner) => AppError::Internal(inner.into()),
            QuizError::Internal(inner) => AppError::Internal(inner),
        }
    }
}
