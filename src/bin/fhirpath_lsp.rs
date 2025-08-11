//! FHIRPath Language Server Protocol binary
//!
//! Async-first LSP server for FHIRPath with comprehensive IDE support including:
//! - Intelligent code completion with type awareness
//! - Real-time diagnostics and error reporting
//! - Hover information with type details and documentation
//! - Go-to-definition for properties and functions
//! - Built on the high-performance analyzer framework

use clap::Parser;
use octofhir_fhirpath::model::mock_provider::MockModelProvider;
use octofhir_fhirpath::registry::create_standard_registries;
use std::sync::Arc;
use tokio;

#[cfg(feature = "lsp")]
use octofhir_fhirpath::lsp::{LspConfig, start_server};

/// FHIRPath Language Server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Log file path (optional)
    #[arg(short, long)]
    log_file: Option<String>,

    /// Enable all features (completion, diagnostics, hover, navigation)
    #[arg(long, default_value = "true")]
    enable_all: bool,

    /// Maximum analysis depth
    #[arg(long, default_value = "50")]
    max_depth: u32,

    /// Diagnostic delay in milliseconds
    #[arg(long, default_value = "300")]
    diagnostic_delay: u64,
}

#[cfg(not(feature = "lsp"))]
fn main() {
    eprintln!("Error: LSP support is not enabled. Please compile with --features lsp");
    std::process::exit(1);
}

#[cfg(feature = "lsp")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    log::info!(
        "Starting FHIRPath Language Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create mock model provider (in real usage, this would be a proper FHIR schema provider)
    let model_provider = Arc::new(MockModelProvider::with_fhir_r4());
    log::info!(
        "Initialized model provider: {}",
        model_provider.fhir_version()
    );

    // Create function registry with all built-in functions
    let (function_registry, _operator_registry) = create_standard_registries();
    let function_registry = Arc::new(function_registry);
    log::info!(
        "Initialized function registry with {} functions",
        function_registry.function_names().len()
    );

    // Create LSP configuration
    let config = LspConfig {
        enable_logging: args.verbose,
        max_analysis_depth: args.max_depth,
        enable_completions: args.enable_all,
        enable_diagnostics: args.enable_all,
        enable_hover: args.enable_all,
        enable_navigation: args.enable_all,
        diagnostic_delay_ms: args.diagnostic_delay,
        completion_triggers: vec![".".to_string(), "(".to_string(), " ".to_string()],
    };

    log::info!("LSP Configuration:");
    log::info!("  Completions: {}", config.enable_completions);
    log::info!("  Diagnostics: {}", config.enable_diagnostics);
    log::info!("  Hover: {}", config.enable_hover);
    log::info!("  Navigation: {}", config.enable_navigation);
    log::info!("  Max Depth: {}", config.max_analysis_depth);

    // Start the LSP server
    log::info!("Starting LSP server on stdio...");
    if let Err(err) = start_server(model_provider, function_registry, config).await {
        log::error!("LSP server error: {}", err);
        return Err(err);
    }

    log::info!("LSP server stopped");
    Ok(())
}
