use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::admin::audit::dto::PaginationDto;

#[derive(Debug, Serialize, ToSchema)]
pub struct WeightDto {
    pub role_id: Uuid,
    pub role_code: String,
    pub weight: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreQuizOptionDto {
    pub id: Uuid,
    pub option_text: String,
    pub order_index: i32,
    pub weights: Vec<WeightDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreQuizQuestionDto {
    pub id: Uuid,
    pub question_text: String,
    pub order_index: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub options: Vec<PreQuizOptionDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PreQuizListResponse {
    pub questions: Vec<PreQuizQuestionDto>,
    pub pagination: PaginationDto,
}

#[derive(Debug, Deserialize, Default, ToSchema)]
pub struct PreQuizFilter {
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct WeightInput {
    pub role_id: Uuid,
    #[validate(range(min = 0, max = 10))]
    pub weight: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PreQuizOptionInput {
    #[validate(length(min = 1))]
    pub option_text: String,
    pub order_index: i32,
    #[validate(length(min = 1))]
    pub weights: Vec<WeightInput>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePreQuizQuestionRequest {
    #[validate(length(min = 1))]
    pub question_text: String,
    pub order_index: Option<i32>,
    #[validate(length(min = 2))]
    pub options: Vec<PreQuizOptionInput>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePreQuizQuestionRequest {
    #[validate(length(min = 1))]
    pub question_text: Option<String>,
    pub order_index: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ReplacePreQuizOptionsRequest {
    #[validate(length(min = 2))]
    pub options: Vec<PreQuizOptionInput>,
}
