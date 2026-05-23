use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct AdminModuleQuizOptionDto {
    pub id: Uuid,
    pub text: String,
    pub is_correct: bool,
    pub order_index: i32,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ModuleQuizOptionInput {
    #[validate(length(min = 1))]
    pub text: String,
    pub is_correct: bool,
    pub order_index: i32,
}

#[derive(Debug, Serialize)]
pub struct AdminModuleQuizQuestionDto {
    pub id: Uuid,
    pub module_id: Uuid,
    pub question: String,
    pub order_index: i32,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub options: Vec<AdminModuleQuizOptionDto>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateModuleQuizQuestionRequest {
    pub module_id: Uuid,
    #[validate(length(min = 1))]
    pub question_text: String,
    pub order_index: Option<i32>,
    #[validate(length(min = 2))]
    pub options: Vec<ModuleQuizOptionInput>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateModuleQuizQuestionRequest {
    #[validate(length(min = 1))]
    pub question_text: Option<String>,
    pub order_index: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReplaceModuleQuizOptionsRequest {
    #[validate(length(min = 2))]
    pub options: Vec<ModuleQuizOptionInput>,
}
