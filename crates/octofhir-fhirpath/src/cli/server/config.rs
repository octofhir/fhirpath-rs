//! Server configuration module
//!
//! Provides configuration structures for the FHIRPath HTTP server.

use std::path::PathBuf;

/// Configuration for the FHIRPath HTTP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to bind the server to
    pub port: u16,
    /// Host to bind to
    pub host: String,
    /// Directory for JSON file storage
    pub storage_dir: PathBuf,
    /// Enable CORS for all origins (development mode)
    pub cors_all: bool,
    /// Maximum request payload size in bytes
    pub max_payload_size: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            storage_dir: PathBuf::from("./storage"),
            cors_all: false,
            max_payload_size: 60 * 1024 * 1024, // 10MB
            request_timeout: 30,                // 30 seconds
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration with custom values
    pub fn new(port: u16, host: String, storage_dir: PathBuf, cors_all: bool) -> Self {
        Self {
            port,
            host,
            storage_dir,
            cors_all,
            ..Default::default()
        }
    }

    /// Ensure the storage directory exists
    pub async fn ensure_storage_dir(&self) -> anyhow::Result<()> {
        if !self.storage_dir.exists() {
            tokio::fs::create_dir_all(&self.storage_dir).await?;
            tracing::info!(
                "📁 Created storage directory: {}",
                self.storage_dir.display()
            );
        }
        Ok(())
    }
}
