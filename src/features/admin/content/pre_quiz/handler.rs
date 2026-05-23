use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    features::admin::content::pre_quiz::{
        dto::{CreatePreQuizQuestionRequest, PreQuizFilter, ReplacePreQuizOptionsRequest, UpdatePreQuizQuestionRequest},
        service,
    },
    middleware::role_guard::AdminUser,
    state::AppState,
};

#[derive(Debug, Deserialize, Default)]
pub struct ForceQuery { pub force: Option<bool> }

#[utoipa::path(
    get,
    path = "/api/admin/pre-quiz-questions",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of pre-quiz questions", body = crate::features::admin::content::pre_quiz::dto::PreQuizListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_questions(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(f): Query<PreQuizFilter>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::list_questions(&state.db, &f).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    get,
    path = "/api/admin/pre-quiz-questions/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Question ID"),
    ),
    responses(
        (status = 200, description = "Pre-quiz question detail", body = crate::features::admin::content::pre_quiz::dto::PreQuizQuestionDto),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_question(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result = service::get_question(&state.db, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "data": result })))
}

#[utoipa::path(
    post,
    path = "/api/admin/pre-quiz-questions",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    request_body = CreatePreQuizQuestionRequest,
    responses(
        (status = 201, description = "Question created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn create_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePreQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    let id = service::create_question(&state.db, admin.0.user_id, &req).await.map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": { "id": id } }))))
}

#[utoipa::path(
    patch,
    path = "/api/admin/pre-quiz-questions/{id}",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Question ID"),
    ),
    request_body = UpdatePreQuizQuestionRequest,
    responses(
        (status = 200, description = "Question updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePreQuizQuestionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::update_question(&state.db, admin.0.user_id, id, &req).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question updated" })))
}

#[utoipa::path(
    patch,
    path = "/api/admin/pre-quiz-questions/{id}/options",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Question ID"),
    ),
    request_body = ReplacePreQuizOptionsRequest,
    responses(
        (status = 200, description = "Options replaced"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn replace_options(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<ForceQuery>,
    Json(req): Json<ReplacePreQuizOptionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;
    service::replace_options(&state.db, admin.0.user_id, id, &req, q.force.unwrap_or(false))
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Options replaced" })))
}

#[utoipa::path(
    post,
    path = "/api/admin/pre-quiz-questions/{id}/deactivate",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Question ID"),
    ),
    responses(
        (status = 200, description = "Question deactivated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn deactivate_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::deactivate_question(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question deactivated" })))
}

#[utoipa::path(
    post,
    path = "/api/admin/pre-quiz-questions/{id}/restore",
    tag = "Admin - Content",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Question ID"),
    ),
    responses(
        (status = 200, description = "Question restored"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn restore_question(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service::restore_question(&state.db, admin.0.user_id, id).await.map_err(AppError::from)?;
    Ok(Json(json!({ "success": true, "message": "Question restored" })))
}
