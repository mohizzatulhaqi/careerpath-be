use crate::{
    error::AppError,
    features::{
        learning::{error::LearningError, progress::service as progress_service},
        project::{dto::SubmitProjectInput, error::ProjectError, service},
    },
    middleware::auth::AuthUser,
    shared::{body::chunked_body, response::ApiResponse},
    state::AppState,
};
use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::sync::Arc;
use uuid::Uuid;

fn learning_err_to_project(e: LearningError) -> ProjectError {
    let _ = e;
    ProjectError::NotFound
}

#[utoipa::path(
    get,
    path = "/api/projects/me",
    tag = "Project",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "My project info", body = crate::features::project::dto::MyProjectResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/projects/{id}",
    tag = "Project",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project detail", body = crate::features::project::dto::ProjectDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// GET /api/projects/:id
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_project(&state, auth.user_id, project_id).await?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    post,
    path = "/api/projects/{id}/submit",
    tag = "Project",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    request_body(content_type = "multipart/form-data", description = "ZIP file + submission notes"),
    responses(
        (status = 200, description = "Project submitted", body = crate::features::project::dto::SubmitResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/projects/{id}/submissions",
    tag = "Project",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Submission history", body = Vec<crate::features::project::dto::SubmissionHistoryItem>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// GET /api/projects/:id/submissions
pub async fn get_submissions(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let data = service::get_submissions(&state, auth.user_id, project_id).await?;
    Ok(ApiResponse::ok(data))
}

#[utoipa::path(
    get,
    path = "/api/projects/submissions/{submission_id}/download",
    tag = "Project",
    security(("bearer_auth" = [])),
    params(
        ("submission_id" = Uuid, Path, description = "Submission ID"),
    ),
    responses(
        (status = 200, description = "Download submission file"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
    )
)]
/// GET /api/projects/submissions/:submission_id/download
pub async fn download_submission(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(submission_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let (data, file_name) = service::download_submission(&state, auth.user_id, submission_id).await?;

    let ascii_name: String = file_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' })
        .collect();
    let encoded_name = utf8_percent_encode(&file_name, NON_ALPHANUMERIC).to_string();
    let content_disposition =
        format!("attachment; filename=\"{ascii_name}\"; filename*=UTF-8''{encoded_name}");

    let content_length = data.len();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CONTENT_LENGTH, content_length.to_string())
        .body(chunked_body(data))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("response build: {e}")))
}
