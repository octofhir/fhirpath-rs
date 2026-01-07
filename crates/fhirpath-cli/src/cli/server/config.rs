//! Server configuration module
//!
//! Provides configuration structures for the FHIRPath Lab API server.

use std::net::IpAddr;

/// Trace provider configuration
#[derive(Debug, Clone, Default)]
pub enum TraceConfig {
    /// No trace output (silent)
    None,
    /// Output traces to stderr (CLI mode)
    #[default]
    Cli,
    /// Collect traces for API responses (server mode)
    Server,
}

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
    /// Trace provider configuration
    pub trace_config: TraceConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8084,
            host: [127, 0, 0, 1].into(),
            cors_all: false,
            max_body_size_mb: 60,
            timeout_seconds: 30,
            rate_limit_per_minute: 100,
            trace_config: TraceConfig::Server, // Default to server mode for API responses
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
        trace_config: TraceConfig,
    ) -> Self {
        Self {
            port,
            host,
            cors_all,
            max_body_size_mb,
            timeout_seconds,
            rate_limit_per_minute,
            trace_config,
        }
    }

    /// Create the appropriate trace provider based on configuration
    pub fn create_trace_provider(&self) -> octofhir_fhirpath::core::SharedTraceProvider {
        match self.trace_config {
            TraceConfig::None => octofhir_fhirpath::core::trace::create_noop_provider(),
            TraceConfig::Cli => octofhir_fhirpath::core::trace::create_cli_provider(),
            TraceConfig::Server => {
                crate::cli::server::trace::ServerApiTraceProvider::create_shared()
            }
        }
    }

    /// Get maximum payload size in bytes
    pub fn max_payload_size(&self) -> usize {
        (self.max_body_size_mb as usize) * 1024 * 1024
    }
}
