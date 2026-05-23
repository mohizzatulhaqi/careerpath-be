use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self { success: true, data: Some(data), message: None }),
        )
    }

    pub fn created(data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::CREATED,
            Json(Self { success: true, data: Some(data), message: None }),
        )
    }

    pub fn ok_with_msg(data: T, msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self { success: true, data: Some(data), message: Some(msg.into()) }),
        )
    }
}
