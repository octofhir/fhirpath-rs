//! HTTP server module for FHIRPath REST API
//!
//! Provides a web server with REST endpoints for FHIRPath evaluation across all supported FHIR versions.
//! Features include:
//! - Versioned evaluation endpoints for R4, R4B, R5, R6
//! - Expression analysis with validation
//! - File management for FHIR resource storage
//! - Embedded SolidJS web interface
//! - CORS support for web-based tools

pub mod assets;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod registry;
pub mod response;
pub mod trace;
pub mod version;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    routing::{get, post},
};
use std::net::SocketAddr;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};

use crate::cli::server::{
    config::ServerConfig,
    handlers::{
        fhirpath_lab_handler, fhirpath_lab_r4_handler, fhirpath_lab_r4b_handler,
        fhirpath_lab_r5_handler, fhirpath_lab_r6_handler, health_handler, version_handler,
    },
    registry::ServerRegistry,
};

/// Start the FHIRPath HTTP server
pub async fn start_server(config: ServerConfig) -> anyhow::Result<()> {
    // Initialize tracing/logging subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!(
        "🚀 Starting FHIRPath Lab API server on {}:{}",
        config.host, config.port
    );

    // Initialize server registry with all FHIR versions
    info!("🔧 Initializing FhirPathEngine registry for all FHIR versions...");
    let registry = ServerRegistry::new().await?;
    info!(
        "✅ Registry initialized with {} FHIR versions",
        registry.version_count()
    );

    // Create the main router
    let app = create_app(registry, config.clone()).await?;

    // Create socket address
    let addr = SocketAddr::from((config.host, config.port));

    info!("🌐 Starting FHIRPath Lab API server on http://{}", addr);

    if config.cors_all {
        warn!("⚠️  CORS enabled for all origins (development mode)");
    }

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the main application router
async fn create_app(registry: ServerRegistry, config: ServerConfig) -> anyhow::Result<Router> {
    // Setup CORS
    let cors = if config.cors_all {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([CONTENT_TYPE])
            .allow_origin(Any)
    } else {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([CONTENT_TYPE])
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap()) // Default frontend dev server
    };

    // Create FHIRPath Lab API routes (start with just health checks)
    let app = Router::new()
        // Health check endpoints
        .route("/health", get(health_handler))
        .route("/healthz", get(health_handler))
        .route("/version", get(version_handler))
        .route("/", post(fhirpath_lab_handler))
        .route("/r4", post(fhirpath_lab_r4_handler))
        .route("/r4b", post(fhirpath_lab_r4b_handler))
        .route("/r5", post(fhirpath_lab_r5_handler))
        .route("/r6", post(fhirpath_lab_r6_handler));

    // Apply middleware
    let app = app
        .layer(DefaultBodyLimit::max(config.max_payload_size())) // Configurable limit
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(CatchPanicLayer::new())
        .with_state(registry);

    Ok(app)
}
