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
    handlers::{
        analyze_handler, evaluate_handler, fhirpath_lab_handler, fhirpath_lab_r4_handler,
        fhirpath_lab_r4b_handler, fhirpath_lab_r5_handler, fhirpath_lab_r6_handler, files_handler,
        health_handler, version_handler,
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

    let mode = if config.no_ui {
        "API-only"
    } else {
        "Full (API + Web UI)"
    };
    info!(
        "🚀 Starting FHIRPath server ({}) on {}:{} with storage at {}",
        mode,
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

    // Check if UI assets are available (only relevant if not in no-ui mode)
    if !config.no_ui {
        if ui_assets_available() {
            info!("🎨 Web UI available with {} embedded assets", asset_count());
        } else {
            warn!("⚠️  Web UI not available - run 'cd ui && pnpm install && pnpm build' to enable");
        }
    } else {
        info!("🔧 Running in API-only mode - Web UI disabled");
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

    // Create basic API routes - always include health and version
    let mut app = Router::new()
        // Health check endpoints
        .route("/health", get(health_handler))
        .route("/healthz", get(health_handler))
        .route("/version", get(version_handler))
        // FHIRPath Lab API endpoints (core functionality)
        .route("/r4", post(fhirpath_lab_r4_handler))
        .route("/r4b", post(fhirpath_lab_r4b_handler))
        .route("/r5", post(fhirpath_lab_r5_handler))
        .route("/r6", post(fhirpath_lab_r6_handler));

    // Conditionally add features based on no_ui flag
    if !config.no_ui {
        // Add UI-specific routes: file management, legacy test endpoints, UI serving
        app = app
            // File management endpoints (needed for UI)
            .route(
                "/files",
                get(files_handler::list_files).post(files_handler::upload_file),
            )
            .route(
                "/files/{filename}",
                get(files_handler::get_file).delete(files_handler::delete_file),
            )
            // Legacy test routes for backward compatibility (used by UI)
            .route("/test/evaluate", post(evaluate_handler))
            .route("/test/analyze", post(analyze_handler))
            // UI serving routes
            .route("/", get(serve_ui_root))
            .route("/{*path}", get(serve_embedded_assets));
    } else {
        // In API-only mode, add FHIRPath Lab API root endpoint only
        app = app.route("/", post(fhirpath_lab_handler));
    }

    // Apply middleware
    let app = app
        .layer(DefaultBodyLimit::max(config.max_payload_size())) // Configurable limit
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(CatchPanicLayer::new())
        .with_state(registry);

    Ok(app)
}
