use async_trait::async_trait;
use bytes::Bytes;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("file not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("storage error: {0}")]
    Other(String),
}

// ── Stored file ───────────────────────────────────────────────────────────────

pub struct StoredFile {
    pub data: Bytes,
    pub mime_type: String,
}

// ── Storage trait ─────────────────────────────────────────────────────────────

#[async_trait]
pub trait Storage: Send + Sync + std::fmt::Debug {
    /// Store a file and return its relative path.
    async fn store(&self, id: Uuid, data: Bytes) -> Result<String, StorageError>;

    /// Retrieve a file by its relative path.
    async fn retrieve(&self, path: &str) -> Result<StoredFile, StorageError>;

    /// Delete a file by its relative path.
    async fn delete(&self, path: &str) -> Result<(), StorageError>;
}

// ── Local filesystem implementation ──────────────────────────────────────────

pub mod local {
    use super::*;
    use std::path::Path;

    #[derive(Debug, Clone)]
    pub struct LocalStorage {
        root: PathBuf,
    }

    impl LocalStorage {
        pub fn new(root: PathBuf) -> Result<Self, StorageError> {
            std::fs::create_dir_all(&root)?;
            Ok(Self { root })
        }

        fn full_path(&self, relative: &str) -> PathBuf {
            self.root.join(relative)
        }
    }

    #[async_trait]
    impl Storage for LocalStorage {
        async fn store(&self, id: Uuid, data: Bytes) -> Result<String, StorageError> {
            let relative = format!("{}.zip", id);
            let path = self.full_path(&relative);
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, &data).await?;
            Ok(relative)
        }

        async fn retrieve(&self, relative: &str) -> Result<StoredFile, StorageError> {
            let path = self.full_path(relative);
            if !path.exists() {
                return Err(StorageError::NotFound(relative.to_string()));
            }
            let data = tokio::fs::read(&path).await?;
            let mime_type = mime_guess(Path::new(relative));
            Ok(StoredFile {
                data: Bytes::from(data),
                mime_type,
            })
        }

        async fn delete(&self, relative: &str) -> Result<(), StorageError> {
            let path = self.full_path(relative);
            if path.exists() {
                tokio::fs::remove_file(&path).await?;
            }
            Ok(())
        }
    }

    fn mime_guess(path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("zip") => "application/zip".to_string(),
            Some("pdf") => "application/pdf".to_string(),
            Some("png") => "image/png".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}

// ── ZIP validator ─────────────────────────────────────────────────────────────

pub mod validator {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum ValidationError {
        #[error("file size {size} bytes exceeds limit of {limit} bytes")]
        TooLarge { size: u64, limit: u64 },

        #[error("file must be a .zip archive, got: {0}")]
        NotZip(String),

        #[error("file is empty")]
        Empty,
    }

    pub struct ValidationConfig {
        pub max_bytes: u64,
    }

    pub struct ValidatedFile {
        pub original_name: String,
        pub size: u64,
        pub mime_type: String,
    }

    /// Validate that the uploaded bytes are a non-empty ZIP within the size limit.
    pub fn validate_zip(
        data: &[u8],
        file_name: &str,
        cfg: &ValidationConfig,
    ) -> Result<ValidatedFile, ValidationError> {
        if data.is_empty() {
            return Err(ValidationError::Empty);
        }

        let size = data.len() as u64;
        if size > cfg.max_bytes {
            return Err(ValidationError::TooLarge { size, limit: cfg.max_bytes });
        }

        let ext = std::path::Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Verify ZIP magic bytes: PK\x03\x04
        let is_zip_magic = data.len() >= 4 && &data[0..4] == b"PK\x03\x04";
        if ext != "zip" || !is_zip_magic {
            return Err(ValidationError::NotZip(file_name.to_string()));
        }

        Ok(ValidatedFile {
            original_name: file_name.to_string(),
            size,
            mime_type: "application/zip".to_string(),
        })
    }
}
