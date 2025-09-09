//! Server configuration module
//!
//! Provides configuration structures for the FHIRPath Lab API server.

use std::net::IpAddr;

/// Configuration for the FHIRPath Lab API server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to bind the server to
    pub port: u16,
    /// Host IP address to bind to
    pub host: IpAddr,
    /// Enable CORS for all origins (development mode)
    pub cors_all: bool,
    /// Maximum request body size in MB
    pub max_body_size_mb: u64,
    /// Expression execution timeout in seconds
    pub timeout_seconds: u64,
    /// Rate limit: requests per minute per IP
    pub rate_limit_per_minute: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: [127, 0, 0, 1].into(),
            cors_all: false,
            max_body_size_mb: 60,
            timeout_seconds: 30,
            rate_limit_per_minute: 100,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration with custom values
    pub fn new(
        port: u16,
        host: IpAddr,
        cors_all: bool,
        max_body_size_mb: u64,
        timeout_seconds: u64,
        rate_limit_per_minute: u32,
    ) -> Self {
        Self {
            port,
            host,
            cors_all,
            max_body_size_mb,
            timeout_seconds,
            rate_limit_per_minute,
        }
    }

    /// Get maximum payload size in bytes
    pub fn max_payload_size(&self) -> usize {
        (self.max_body_size_mb as usize) * 1024 * 1024
    }
}
