//! Server configuration module
//!
//! Provides configuration structures for the FHIRPath HTTP server.

use std::net::IpAddr;
use std::path::PathBuf;

/// Configuration for the FHIRPath HTTP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to bind the server to
    pub port: u16,
    /// Host IP address to bind to
    pub host: IpAddr,
    /// Directory for JSON file storage
    pub storage_dir: PathBuf,
    /// Enable CORS for all origins (development mode)
    pub cors_all: bool,
    /// Maximum request body size in MB
    pub max_body_size_mb: u64,
    /// Expression execution timeout in seconds
    pub timeout_seconds: u64,
    /// Rate limit: requests per minute per IP
    pub rate_limit_per_minute: u32,
    /// Run server without web UI (API-only mode)
    pub no_ui: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: [127, 0, 0, 1].into(),
            storage_dir: PathBuf::from("./storage"),
            cors_all: false,
            max_body_size_mb: 60,
            timeout_seconds: 30,
            rate_limit_per_minute: 100,
            no_ui: false,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration with custom values
    pub fn new(
        port: u16,
        host: IpAddr,
        storage_dir: PathBuf,
        cors_all: bool,
        max_body_size_mb: u64,
        timeout_seconds: u64,
        rate_limit_per_minute: u32,
        no_ui: bool,
    ) -> Self {
        Self {
            port,
            host,
            storage_dir,
            cors_all,
            max_body_size_mb,
            timeout_seconds,
            rate_limit_per_minute,
            no_ui,
        }
    }

    /// Get maximum payload size in bytes
    pub fn max_payload_size(&self) -> usize {
        (self.max_body_size_mb as usize) * 1024 * 1024
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
