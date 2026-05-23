use crate::{
    error::AppError,
    features::{
        learning::{error::LearningError, progress::service as progress_service},
        project::{dto::SubmitProjectInput, error::ProjectError, service},
    },
    middleware::auth::AuthUser,
    shared::response::ApiResponse,
    state::AppState,
};
use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use std::sync::Arc;
use uuid::Uuid;

fn learning_err_to_project(e: LearningError) -> ProjectError {
    let _ = e;
    ProjectError::NotFound
}

/// GET /api/projects/me
pub async fn get_my_project(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let (role_id, role_info) = progress_service::resolve_user_role(&state, auth.user_id)
        .await
        .map_err(learning_err_to_project)?;
    let data = service::get_my_project(&state, auth.user_id, role_id, role_info.code, role_info.name).await?;
    Ok(ApiResponse::ok(data))
}

/// GET /api/projects/:id
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_project(&state, auth.user_id, project_id).await?;
    Ok(ApiResponse::ok(data))
}

/// POST /api/projects/:id/submit  (multipart/form-data)
pub async fn submit_project(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut notes: Option<String> = None;
    let mut file_bytes: Option<Bytes> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ProjectError::MultipartError(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "submission_notes" => {
                notes = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| ProjectError::MultipartError(e.to_string()))?,
                );
            }
            "file" => {
                file_name = field.file_name().map(String::from);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| ProjectError::MultipartError(e.to_string()))?,
                );
            }
            _ => {}
        }
    }

    let input = SubmitProjectInput {
        submission_notes: notes.ok_or(ProjectError::MissingNotes)?,
        file_bytes: file_bytes.ok_or(ProjectError::MissingFile)?,
        file_name: file_name.ok_or(ProjectError::MissingFile)?,
    };

    let data = service::submit_project(&state, auth.user_id, project_id, input).await?;
    Ok(ApiResponse::ok(data))
}

/// GET /api/projects/:id/submissions
pub async fn get_submissions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_submissions(&state, auth.user_id, project_id).await?;
    Ok(ApiResponse::ok(data))
}

/// GET /api/projects/submissions/:submission_id/download
pub async fn download_submission(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(submission_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let (data, file_name) = service::download_submission(&state, auth.user_id, submission_id).await?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{file_name}\""),
        )
        .header(header::CONTENT_LENGTH, data.len().to_string())
        .body(Body::from(data))
        .unwrap();
    Ok(response)
}
