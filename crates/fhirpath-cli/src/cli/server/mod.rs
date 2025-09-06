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
    assets::{asset_count, serve_embedded_assets, serve_ui_root, ui_assets_available},
    config::ServerConfig,
    handlers::{analyze_handler, evaluate_handler, files_handler, health_handler, version_handler},
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
        "🚀 Starting FHIRPath server on {}:{} with storage at {}",
        config.host,
        config.port,
        config.storage_dir.display()
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

    info!("🌐 Starting FHIRPath server on http://{}", addr);
    info!(
        "📁 File storage directory: {}",
        config.storage_dir.display()
    );

    // Check if UI assets are available
    if ui_assets_available() {
        info!("🎨 Web UI available with {} embedded assets", asset_count());
    } else {
        warn!("⚠️  Web UI not available - run 'cd ui && pnpm install && pnpm build' to enable");
    }

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

    // Create versioned routes for each FHIR version
    let mut app = Router::new()
        // Health check
        .route("/health", get(health_handler))
        // File management endpoints
        .route(
            "/files",
            get(files_handler::list_files).post(files_handler::upload_file),
        )
        .route(
            "/files/{filename}",
            get(files_handler::get_file).delete(files_handler::delete_file),
        )
        // Static UI serving - root and SPA routes
        .route("/", get(serve_ui_root))
        .route("/{*path}", get(serve_embedded_assets));

    // Add health check and version endpoints (required by task)
    app = app
        .route("/healthz", get(health_handler))
        .route("/version", get(version_handler));

    // Add simplified test routes first
    app = app
        .route("/test/evaluate", post(evaluate_handler))
        .route("/test/analyze", post(analyze_handler));

    // TODO: Add versioned routes later
    // for version in ["r4", "r4b", "r5", "r6"] {
    //     app = app
    //         .route(&format!("/{}/evaluate", version), post(evaluate_handler))
    //         .route(&format!("/{}/analyze", version), post(analyze_handler));
    // }

    // Apply middleware
    let app = app
        .layer(DefaultBodyLimit::max(config.max_payload_size())) // Configurable limit
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(CatchPanicLayer::new())
        .with_state(registry);

    Ok(app)
}
