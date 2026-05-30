use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum StorageBackend {
    Local,
    Database,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    /// Access token TTL in seconds (default: 900 = 15 min)
    pub jwt_expires_in: u64,
    /// Refresh token TTL in seconds (default: 604800 = 7 days)
    pub refresh_token_expires_in: u64,
    pub server_port: u16,
    /// Root directory for local file storage (used when STORAGE_BACKEND=local)
    pub storage_root: PathBuf,
    /// Maximum allowed upload size in bytes (default: 26_214_400 = 25 MB)
    pub max_upload_size_bytes: u64,
    /// Base URL used in certificate verification links (e.g. "https://careerpath.example.com")
    pub app_base_url: String,
    /// Storage backend: local (filesystem) or database (PostgreSQL BYTEA)
    pub storage_backend: StorageBackend,
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
            // Render injects PORT; SERVER_PORT is the local fallback
            server_port: std::env::var("PORT")
                .or_else(|_| std::env::var("SERVER_PORT"))
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("PORT/SERVER_PORT must be a valid port number")?,
            storage_root: PathBuf::from(
                std::env::var("STORAGE_ROOT").unwrap_or_else(|_| "storage".to_string()),
            ),
            max_upload_size_bytes: std::env::var("MAX_UPLOAD_SIZE_BYTES")
                .unwrap_or_else(|_| "26214400".to_string())
                .parse()
                .context("MAX_UPLOAD_SIZE_BYTES must be a positive integer")?,
            app_base_url: std::env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".to_string()),
            storage_backend: match std::env::var("STORAGE_BACKEND")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "database" | "db" => StorageBackend::Database,
                _ => StorageBackend::Local,
            },
        })
    }
}
