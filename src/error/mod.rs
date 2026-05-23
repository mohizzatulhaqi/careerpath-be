use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Gone(String),

    /// 401 with code "ACCOUNT_DEACTIVATED" — account has been soft-deleted by admin.
    #[error("{0}")]
    AccountDeactivated(String),

    /// 409 with code "REQUIRES_FORCE" — action has side effects, needs ?force=true to confirm.
    #[error("{message}")]
    RequiresForce { message: String, affected_count: i64 },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // RequiresForce has a different shape (extra field), handle separately.
        if let AppError::RequiresForce { ref message, affected_count } = self {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "REQUIRES_FORCE",
                        "message": message,
                        "affected_count": affected_count
                    }
                })),
            )
                .into_response();
        }

        let (status, code, message) = match &self {
            AppError::Internal(e) => {
                tracing::error!("Internal error: {e:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                msg.clone(),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::Gone(msg) => (StatusCode::GONE, "GONE", msg.clone()),
            AppError::AccountDeactivated(msg) => {
                (StatusCode::UNAUTHORIZED, "ACCOUNT_DEACTIVATED", msg.clone())
            }
            AppError::RequiresForce { .. } => unreachable!("handled above"),
        };

        (
            status,
            Json(json!({
                "success": false,
                "error": { "code": code, "message": message }
            })),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(e.into())
    }
}
