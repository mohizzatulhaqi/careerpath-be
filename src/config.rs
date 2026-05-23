use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    /// Access token TTL in seconds (default: 900 = 15 min)
    pub jwt_expires_in: u64,
    /// Refresh token TTL in seconds (default: 604800 = 7 days)
    pub refresh_token_expires_in: u64,
    pub server_port: u16,
    /// Root directory for local file storage
    pub storage_root: PathBuf,
    /// Maximum allowed upload size in bytes (default: 26_214_400 = 25 MB)
    pub max_upload_size_bytes: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            jwt_secret: std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?,
            jwt_expires_in: std::env::var("JWT_EXPIRES_IN")
                .unwrap_or_else(|_| "900".to_string())
                .parse()
                .context("JWT_EXPIRES_IN must be a positive integer")?,
            refresh_token_expires_in: std::env::var("REFRESH_TOKEN_EXPIRES_IN")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .context("REFRESH_TOKEN_EXPIRES_IN must be a positive integer")?,
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("SERVER_PORT must be a valid port number")?,
            storage_root: PathBuf::from(
                std::env::var("STORAGE_ROOT").unwrap_or_else(|_| "storage".to_string()),
            ),
            max_upload_size_bytes: std::env::var("MAX_UPLOAD_SIZE_BYTES")
                .unwrap_or_else(|_| "26214400".to_string())
                .parse()
                .context("MAX_UPLOAD_SIZE_BYTES must be a positive integer")?,
        })
    }
}
