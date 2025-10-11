//! FHIRPath Language Server Protocol implementation
//!
//! Binary entry point for the FHIRPath LSP server.

use anyhow::Result;
use fhirpath_lsp::{Config, FhirPathLanguageServer};
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fhirpath_lsp=info".parse()?),
        )
        .init();

    tracing::info!("FHIRPath LSP server starting...");

    // Try to load config (use default if not found)
    let config = std::env::current_dir()
        .ok()
        .and_then(|dir| Config::find_config_file(&dir))
        .and_then(|path| {
            tracing::info!("Loading config from: {}", path.display());
            Config::from_file(&path).ok()
        })
        .unwrap_or_else(|| {
            tracing::info!("Using default configuration");
            Config::default()
        });

    tracing::info!("FHIR version: {:?}", config.fhir_version);

    // Create LSP service with loaded config
    let config_for_service = config.clone();
    let (service, socket) = LspService::build(move |client| {
        FhirPathLanguageServer::new_with_config(client, config_for_service)
    })
    .finish();

    // Note: Config file watching would require server to be Arc-wrapped
    // This will be implemented when needed

    // Start server on stdin/stdout
    tracing::info!("LSP server listening on stdin/stdout");
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;

    tracing::info!("FHIRPath LSP server stopped");

    Ok(())
}
