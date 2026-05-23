use super::{
    dto::{
        LatestSubmissionDto, ModuleProgressDto, MyProjectResponse, ProjectDetailDto, RoleDto,
        SubmissionHistoryItem, SubmitProjectInput, SubmitResponse,
    },
    error::ProjectError,
    gating::{can_submit, compute_project_access},
    repository,
    storage::validator::{validate_zip, ValidationConfig},
};
use crate::{shared::sanitization, state::AppState};
use std::sync::Arc;
use uuid::Uuid;

const MIN_NOTES_LEN: usize = 20;
const MAX_NOTES_LEN: usize = 5000;

fn to_submission_history_item(s: super::entity::ProjectSubmission) -> SubmissionHistoryItem {
    SubmissionHistoryItem {
        id: s.id,
        file_original_name: s.file_original_name,
        file_size_bytes: s.file_size_bytes,
        submission_notes: s.submission_notes,
        status: s.status,
        reviewer_notes: s.reviewer_notes,
        reviewed_at: s.reviewed_at,
        submitted_at: s.submitted_at,
    }
}

async fn build_project_detail(
    state: &Arc<AppState>,
    project: super::entity::Project,
    user_id: Uuid,
) -> Result<ProjectDetailDto, ProjectError> {
    let counts = repository::count_module_completion(&state.db, project.role_id, user_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?;

    let latest_status = repository::get_latest_submission_status(&state.db, project.id, user_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?;

    let access_status = compute_project_access(&counts, latest_status.as_deref());

    let submissions =
        repository::get_submissions_by_project_and_user(&state.db, project.id, user_id)
            .await
            .map_err(|e| ProjectError::Storage(e.to_string()))?;

    let latest_submission = submissions.into_iter().next().map(|s| LatestSubmissionDto {
        id: s.id,
        submission_notes: s.submission_notes,
        file_original_name: s.file_original_name,
        file_size_bytes: s.file_size_bytes,
        status: s.status,
        reviewer_notes: s.reviewer_notes,
        reviewed_at: s.reviewed_at,
        submitted_at: s.submitted_at,
    });

    Ok(ProjectDetailDto {
        id: project.id,
        title: project.title,
        description: project.description,
        requirements: project.requirements,
        estimated_hours: project.estimated_hours,
        module_progress: ModuleProgressDto {
            total: counts.total,
            completed: counts.completed,
        },
        access_status: access_status.as_str().to_string(),
        latest_submission,
    })
}

/// GET /api/projects/me — project for the user's current role
pub async fn get_my_project(
    state: &Arc<AppState>,
    user_id: Uuid,
    role_id: Uuid,
    role_code: String,
    role_name: String,
) -> Result<MyProjectResponse, ProjectError> {
    let project = repository::get_project_by_role(&state.db, role_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?
        .ok_or(ProjectError::ProjectNotForUserRole)?;

    let project_detail = build_project_detail(state, project, user_id).await?;

    Ok(MyProjectResponse {
        role: RoleDto {
            id: role_id,
            code: role_code,
            name: role_name,
        },
        project: project_detail,
    })
}

/// GET /api/projects/:id
pub async fn get_project(
    state: &Arc<AppState>,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<ProjectDetailDto, ProjectError> {
    let project = repository::get_project_by_id(&state.db, project_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?
        .ok_or(ProjectError::NotFound)?;
    build_project_detail(state, project, user_id).await
}

/// POST /api/projects/:id/submit
pub async fn submit_project(
    state: &Arc<AppState>,
    user_id: Uuid,
    project_id: Uuid,
    input: SubmitProjectInput,
) -> Result<SubmitResponse, ProjectError> {
    // Sanitize HTML + validate length (replaces the old char-count check)
    let sanitized_notes = sanitization::sanitize_plain_text(
        &input.submission_notes,
        MIN_NOTES_LEN,
        MAX_NOTES_LEN,
    )
    .map_err(|e| ProjectError::NotesInvalid(e.to_string()))?;

    let project = repository::get_project_by_id(&state.db, project_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?
        .ok_or(ProjectError::NotFound)?;

    let counts = repository::count_module_completion(&state.db, project.role_id, user_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?;

    let latest_status = repository::get_latest_submission_status(&state.db, project.id, user_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?;

    let access_status = compute_project_access(&counts, latest_status.as_deref());

    if !can_submit(&access_status) {
        if latest_status.as_deref() == Some("pending_review") {
            return Err(ProjectError::SubmissionPendingReview);
        } else if latest_status.as_deref() == Some("approved") {
            return Err(ProjectError::SubmissionAlreadyApproved);
        }
        return Err(ProjectError::Locked);
    }

    let cfg = ValidationConfig {
        max_bytes: state.config.max_upload_size_bytes,
    };
    let validated = validate_zip(&input.file_bytes, &input.file_name, &cfg)
        .map_err(|e| ProjectError::InvalidFile(e.to_string()))?;

    let file_id = uuid::Uuid::new_v4();
    let relative_path = state
        .storage
        .store(file_id, input.file_bytes)
        .await
        .map_err(ProjectError::from)?;

    let submission = repository::insert_submission(
        &state.db,
        project_id,
        user_id,
        &relative_path,
        &validated.original_name,
        validated.size as i64,
        &validated.mime_type,
        &sanitized_notes,
    )
    .await
    .map_err(|e| ProjectError::Storage(e.to_string()))?;

    Ok(SubmitResponse {
        submission_id: submission.id,
        project_id: submission.project_id,
        status: submission.status,
        submitted_at: submission.submitted_at,
        file_original_name: submission.file_original_name,
        file_size_bytes: submission.file_size_bytes,
        submission_notes: submission.submission_notes,
    })
}

/// GET /api/projects/:id/submissions
pub async fn get_submissions(
    state: &Arc<AppState>,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<SubmissionHistoryItem>, ProjectError> {
    let _project = repository::get_project_by_id(&state.db, project_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?
        .ok_or(ProjectError::NotFound)?;

    let submissions =
        repository::get_submissions_by_project_and_user(&state.db, project_id, user_id)
            .await
            .map_err(|e| ProjectError::Storage(e.to_string()))?;

    Ok(submissions.into_iter().map(to_submission_history_item).collect())
}

/// GET /api/projects/submissions/:submission_id/download
pub async fn download_submission(
    state: &Arc<AppState>,
    user_id: Uuid,
    submission_id: Uuid,
) -> Result<(bytes::Bytes, String), ProjectError> {
    let submission = repository::get_submission_by_id(&state.db, submission_id)
        .await
        .map_err(|e| ProjectError::Storage(e.to_string()))?
        .ok_or(ProjectError::SubmissionNotFound)?;

    if submission.user_id != user_id {
        return Err(ProjectError::SubmissionNotOwned);
    }

    let stored = state
        .storage
        .retrieve(&submission.file_path)
        .await
        .map_err(ProjectError::from)?;

    Ok((stored.data, submission.file_original_name))
}
