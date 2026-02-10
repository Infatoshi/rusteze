use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("file not found")]
    NotFound,
    #[error("file too large")]
    TooLarge,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("db error: {0}")]
    Db(#[from] rusteze_db::DbError),
}

/// Local filesystem storage backend. Swap for S3 in production.
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub async fn store(&self, data: &[u8], filename: &str) -> Result<String, MediaError> {
        let id = Uuid::now_v7();
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let path = format!("{id}.{ext}");
        let full_path = self.base_path.join(&path);

        // Ensure parent dir exists
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&full_path, data).await?;
        tracing::info!("stored file: {path} ({} bytes)", data.len());
        Ok(path)
    }

    pub async fn fetch(&self, path: &str) -> Result<Vec<u8>, MediaError> {
        let full_path = self.base_path.join(path);
        tokio::fs::read(&full_path)
            .await
            .map_err(|_| MediaError::NotFound)
    }

    pub async fn delete(&self, path: &str) -> Result<(), MediaError> {
        let full_path = self.base_path.join(path);
        tokio::fs::remove_file(&full_path).await?;
        Ok(())
    }
}
