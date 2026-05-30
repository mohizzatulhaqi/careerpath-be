use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    /// Nama baru (opsional, 1–100 karakter)
    #[validate(length(min = 1, max = 100, message = "name must be 1–100 characters"))]
    pub name: Option<String>,

    /// Password lama — wajib diisi jika ingin ganti password
    pub current_password: Option<String>,

    /// Password baru (opsional, min 8 karakter)
    #[validate(length(min = 8, message = "new_password must be at least 8 characters"))]
    pub new_password: Option<String>,
}
