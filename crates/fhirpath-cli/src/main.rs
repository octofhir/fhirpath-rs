// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! FHIRPath CLI - Streamlined entry point

use clap::Parser;
use fhirpath_cli::EmbeddedModelProvider;
use fhirpath_cli::cli::context::CliContext;
use fhirpath_cli::cli::handlers;
use fhirpath_cli::cli::{Cli, Commands};
use octofhir_fhir_model::provider::FhirVersion;
use std::process;
use std::sync::Arc;
use tokio::runtime::Builder;

fn main() {
    human_panic::setup_panic!();

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .expect("Failed to build Tokio runtime");

    let result = runtime.block_on(async_main());

    if let Err(err) = result {
        // Check if it's a clap error (includes built-in suggestions)
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            // Clap already formats errors nicely with suggestions
            clap_err.exit();
        }

        eprintln!("❌ {err}");
        process::exit(1);
    }
}

async fn async_main() -> anyhow::Result<()> {
    // Load configuration file (if exists)
    let config = fhirpath_cli::cli::config::CliConfig::load().unwrap_or_default();

    // Parse CLI arguments
    let cli_args = Cli::parse();

    // Merge config with CLI args (CLI args take precedence)
    let cli = config.merge_with_cli(&cli_args);

    // Create shared context from CLI options
    let context = CliContext::from_cli(&cli);

    // Show progress indicator for model provider initialization
    let model_provider = create_model_provider_with_progress(&context).await?;

    // Dispatch to appropriate handler
    match &cli.command {
        Commands::Evaluate {
            expression,
            input,
            variables,
            pretty,
            output_format,
            no_color,
            quiet,
            verbose,
            analyze,
            watch,
            batch,
            continue_on_error,
            template,
            pipe,
            profile,
        } => {
            let ctx = context
                .with_subcommand_options(output_format.clone(), *no_color, *quiet, *verbose)
                .with_profile(*profile)
                .with_template(template.clone());

            // Handle pipe mode (either explicit --pipe or auto-detected)
            let is_pipe_mode = *pipe || (input.is_none() && handlers::is_stdin_pipe());

            if is_pipe_mode {
                // Pipe mode: process NDJSON from stdin
                handlers::handle_pipe_mode(expression, variables, &ctx, &model_provider).await?;
            } else if let Some(batch_pattern) = batch {
                handle_evaluate_batch(
                    expression,
                    batch_pattern,
                    variables,
                    *pretty,
                    *analyze,
                    *continue_on_error,
                    &ctx,
                    &model_provider,
                )
                .await?;
            } else {
                #[cfg(feature = "watch")]
                if *watch {
                    handle_evaluate_watch(
                        expression,
                        input.as_deref(),
                        variables,
                        *pretty,
                        *analyze,
                        &ctx,
                        &model_provider,
                    )
                    .await?;
                }
                #[cfg(not(feature = "watch"))]
                {
                    handlers::handle_evaluate(
                        expression,
                        input.as_deref(),
                        variables,
                        *pretty,
                        *analyze,
                        &ctx,
                        &model_provider,
                    )
                    .await;
                }
                #[cfg(feature = "watch")]
                if !*watch {
                    handlers::handle_evaluate(
                        expression,
                        input.as_deref(),
                        variables,
                        *pretty,
                        *analyze,
                        &ctx,
                        &model_provider,
                    )
                    .await;
                }
            }
        }

        Commands::Validate {
            expression,
            output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let ctx =
                context.with_subcommand_options(output_format.clone(), *no_color, *quiet, *verbose);
            handlers::handle_validate(expression, &ctx, &model_provider).await;
        }

        Commands::Analyze {
            expression,
            variables,
            validate_only,
            no_inference,
            output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let ctx =
                context.with_subcommand_options(output_format.clone(), *no_color, *quiet, *verbose);
            handlers::handle_analyze(
                expression,
                variables,
                *validate_only,
                *no_inference,
                &ctx,
                &model_provider,
            )
            .await;
        }

        Commands::Docs { error_code } => {
            handlers::handle_docs(error_code, &context);
        }

        #[cfg(feature = "repl")]
        Commands::Repl {
            input,
            variables,
            history_file,
            history_size,
        } => {
            handle_repl(
                input.as_deref(),
                variables,
                history_file.as_deref(),
                *history_size,
                &model_provider,
            )
            .await;
        }

        Commands::Registry { command } => {
            handlers::handle_registry(command, &context).await;
        }

        #[cfg(feature = "server")]
        Commands::Server {
            port,
            storage,
            host,
            cors_all,
            max_body_size,
            timeout,
            rate_limit,
        } => {
            handle_server(
                *port,
                storage.clone(),
                host.clone(),
                *cors_all,
                *max_body_size,
                *timeout,
                *rate_limit,
            )
            .await;
        }

        Commands::Completions { shell } => {
            handlers::handle_completions(*shell)?;
            return Ok(());
        }

        Commands::Config { command } => {
            handlers::handle_config(command)?;
            return Ok(());
        }

        #[cfg(feature = "tui")]
        Commands::Tui {
            input,
            variables,
            config,
            theme,
            no_mouse,
            no_syntax_highlighting,
            no_auto_completion,
            performance_monitoring,
            check_terminal,
        } => {
            handle_tui(
                input.as_deref(),
                variables,
                config.as_deref(),
                theme,
                *no_mouse,
                *no_syntax_highlighting,
                *no_auto_completion,
                *performance_monitoring,
                *check_terminal,
                &model_provider,
            )
            .await;
        }
    }

    Ok(())
}

/// Create model provider with progress indicator
/// Progress is suppressed for JSON/raw output formats to ensure clean, parseable output
async fn create_model_provider_with_progress(
    context: &CliContext,
) -> anyhow::Result<Arc<EmbeddedModelProvider>> {
    use fhirpath_cli::cli::output::OutputFormat;
    use indicatif::{ProgressBar, ProgressStyle};

    // Suppress progress for JSON/raw formats (machine-readable output)
    let show_progress = !context.quiet
        && context.use_colors()
        && context.output_format != OutputFormat::Json
        && context.output_format != OutputFormat::Raw;

    let provider = if show_progress {
        // Show spinner for model initialization
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message("Initializing FHIR schema...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let provider = EmbeddedModelProvider::new(FhirVersion::R4);

        spinner.finish_with_message("✅ FHIR schema initialized");
        Arc::new(provider)
    } else {
        // Silent initialization for quiet mode or machine-readable formats
        Arc::new(EmbeddedModelProvider::new(FhirVersion::R4))
    };

    Ok(provider)
}

/// Handle evaluate with watch mode - re-evaluate when input file changes
#[cfg(feature = "watch")]
async fn handle_evaluate_watch(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    pretty: bool,
    analyze: bool,
    context: &CliContext,
    model_provider: &Arc<EmbeddedModelProvider>,
) -> anyhow::Result<()> {
    use notify_debouncer_full::{
        DebouncedEvent, Debouncer, FileIdMap, new_debouncer, notify::RecursiveMode,
    };
    use std::path::Path;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    // Watch mode requires an input file
    let input_path = input.ok_or_else(|| {
        anyhow::anyhow!("Watch mode requires an input file. Use -i <file> or --input <file>")
    })?;

    let path = Path::new(input_path);
    if !path.exists() {
        return Err(anyhow::anyhow!("Input file does not exist: {input_path}"));
    }

    // Get absolute path for watching
    let abs_path = path
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to resolve path {input_path}: {e}"))?;

    if !context.quiet {
        println!("👀 Watching {} for changes...", abs_path.display());
        println!("   Press Ctrl+C to stop\n");
    }

    // Initial evaluation
    handlers::handle_evaluate(
        expression,
        Some(input_path),
        variables,
        pretty,
        analyze,
        context,
        model_provider,
    )
    .await;

    // Set up file watcher
    let (tx, rx) = channel();
    let mut debouncer: Debouncer<_, FileIdMap> = new_debouncer(
        Duration::from_millis(500), // 500ms debounce
        None,
        move |result: Result<Vec<DebouncedEvent>, Vec<notify_debouncer_full::notify::Error>>| {
            if let Ok(events) = result {
                for event in events {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
            }
        },
    )?;

    // Watch the file
    debouncer.watch(&abs_path, RecursiveMode::NonRecursive)?;

    // Process file change events
    loop {
        match rx.recv() {
            Ok(_event) => {
                if !context.quiet {
                    println!("\n📝 File changed, re-evaluating...\n");
                }

                // Re-evaluate
                handlers::handle_evaluate(
                    expression,
                    Some(input_path),
                    variables,
                    pretty,
                    analyze,
                    context,
                    model_provider,
                )
                .await;
            }
            Err(e) => {
                eprintln!("Watch error: {e}");
                break;
            }
        }
    }

    Ok(())
}

/// Handle evaluate in batch mode - process multiple files matching a pattern
#[allow(clippy::too_many_arguments)]
async fn handle_evaluate_batch(
    expression: &str,
    pattern: &str,
    variables: &[String],
    pretty: bool,
    analyze: bool,
    _continue_on_error: bool,
    context: &CliContext,
    model_provider: &Arc<EmbeddedModelProvider>,
) -> anyhow::Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    // Find all matching files using glob pattern
    let paths: Vec<_> = glob::glob(pattern)
        .map_err(|e| anyhow::anyhow!("Invalid glob pattern '{pattern}': {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Error reading files: {e}"))?;

    if paths.is_empty() {
        return Err(anyhow::anyhow!(
            "No files found matching pattern: {pattern}"
        ));
    }

    if !context.quiet {
        println!("📁 Processing {} file(s)...\n", paths.len());
    }

    let show_progress = !context.quiet && context.use_colors();
    let progress = if show_progress {
        let pb = ProgressBar::new(paths.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut success_count = 0;

    for (idx, path) in paths.iter().enumerate() {
        let path_str = path.to_string_lossy();

        if let Some(ref pb) = progress {
            pb.set_message(format!("Processing: {path_str}"));
        }

        if !context.quiet && progress.is_none() {
            println!("\n[{}/{}] Processing: {}", idx + 1, paths.len(), path_str);
        }

        // Evaluate expression against this file
        // Note: handle_evaluate prints errors internally
        handlers::handle_evaluate(
            expression,
            Some(&path_str),
            variables,
            pretty,
            analyze,
            context,
            model_provider,
        )
        .await;

        // Track as success (actual evaluation errors are displayed by handle_evaluate)
        success_count += 1;

        if let Some(ref pb) = progress {
            pb.inc(1);
        }
    }

    if let Some(ref pb) = progress {
        pb.finish_with_message(format!("✅ Processed {} file(s)", success_count));
    }

    if !context.quiet {
        println!(
            "\n📊 Batch processing complete: {} file(s) processed",
            success_count
        );
    }

    Ok(())
}

/// Handle REPL command (kept in main.rs as it's specific to CLI setup)
#[cfg(feature = "repl")]
async fn handle_repl(
    input: Option<&str>,
    variables: &[String],
    history_file: Option<&str>,
    history_size: usize,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    use fhirpath_cli::cli::repl::{ReplConfig, start_repl};
    use serde_json::Value as JsonValue;
    use std::path::PathBuf;

    // Parse initial variables
    let mut initial_variables = Vec::new();
    for var in variables {
        if let Some((name, value)) = var.split_once('=') {
            initial_variables.push((name.to_string(), value.to_string()));
        } else {
            eprintln!("Warning: Invalid variable format '{var}'. Expected name=value");
        }
    }

    // Load initial resource if provided
    let initial_resource = if let Some(input_path) = input {
        match std::fs::read_to_string(input_path) {
            Ok(content) => match serde_json::from_str::<JsonValue>(&content) {
                Ok(json) => Some(json),
                Err(e) => {
                    eprintln!("Warning: Failed to parse JSON from '{input_path}': {e}");
                    None
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read file '{input_path}': {e}");
                None
            }
        }
    } else {
        None
    };

    // Create REPL configuration
    let config = ReplConfig {
        prompt: "fhirpath> ".to_string(),
        history_size,
        auto_save_history: true,
        color_output: std::env::var("NO_COLOR").is_err(),
        show_types: false,
        history_file: history_file.map(PathBuf::from),
    };

    // Start REPL
    let model_provider_arc =
        model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>;
    if let Err(e) = start_repl(
        model_provider_arc,
        config,
        initial_resource,
        initial_variables,
    )
    .await
    {
        eprintln!("REPL error: {e}");
        std::process::exit(1);
    }
}

/// Handle server command
#[cfg(feature = "server")]
#[allow(clippy::too_many_arguments)]
async fn handle_server(
    port: u16,
    _storage: std::path::PathBuf,
    host: String,
    cors_all: bool,
    max_body_size: u64,
    _timeout: u64,
    _rate_limit: u32,
) {
    use fhirpath_cli::cli::server::{config::ServerConfig, start_server};

    let config = ServerConfig {
        port,
        host: host
            .parse()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        cors_all,
        max_body_size_mb: max_body_size,
        timeout_seconds: _timeout,
        rate_limit_per_minute: _rate_limit,
        trace_config: fhirpath_cli::cli::server::config::TraceConfig::Server,
    };

    if let Err(e) = start_server(config).await {
        eprintln!("❌ Server error: {e}");
        std::process::exit(1);
    }
}

/// Handle TUI command
#[cfg(feature = "tui")]
#[allow(clippy::too_many_arguments)]
async fn handle_tui(
    input: Option<&str>,
    variables: &[String],
    config_path: Option<&str>,
    theme: &str,
    no_mouse: bool,
    no_syntax_highlighting: bool,
    no_auto_completion: bool,
    performance_monitoring: bool,
    check_terminal: bool,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    use fhirpath_cli::tui::{TuiConfig, check_terminal_capabilities, start_tui};

    // Check terminal capabilities if requested
    if check_terminal {
        match check_terminal_capabilities() {
            Ok(_) => {
                println!("✅ Terminal capabilities check passed");
                println!("   - Minimum size requirement met");
                println!("   - Color support available");
                return;
            }
            Err(e) => {
                eprintln!("❌ Terminal capabilities check failed: {e}");
                eprintln!("   Consider using a larger terminal or different terminal emulator");
                process::exit(1);
            }
        }
    }

    // Load configuration
    let mut config = if let Some(config_path) = config_path {
        match TuiConfig::load_from_file(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to load config from {config_path}: {e}");
                eprintln!("Using default configuration");
                TuiConfig::default()
            }
        }
    } else {
        TuiConfig::load_with_fallbacks().unwrap_or_default()
    };

    // Apply command-line overrides
    if let Err(e) = config.set_theme(theme) {
        eprintln!("Warning: {e}");
        eprintln!("Using default theme");
    }

    if no_mouse {
        config.set_feature("mouse_support", false).ok();
    }

    if no_syntax_highlighting {
        config.set_feature("syntax_highlighting", false).ok();
    }

    if no_auto_completion {
        config.set_feature("auto_completion", false).ok();
    }

    if performance_monitoring {
        config.set_feature("performance_monitoring", true).ok();
    }

    // Parse initial variables
    let mut initial_variables = Vec::new();
    for var in variables {
        if let Some((name, value)) = var.split_once('=') {
            initial_variables.push((name.to_string(), value.to_string()));
        } else {
            eprintln!("Warning: Invalid variable format '{var}', expected 'name=value'");
        }
    }

    // Load initial resource if provided
    let initial_resource = if let Some(input_path) = input {
        match load_resource_from_input(input_path) {
            Ok(resource) => Some(resource),
            Err(e) => {
                eprintln!("Warning: Failed to load resource from '{input_path}': {e}");
                None
            }
        }
    } else {
        None
    };

    // Show startup information
    if !config.ui_preferences.show_performance_info {
        println!(
            "🎨 Starting FHIRPath TUI with {} theme",
            config.theme.metadata.name
        );
        if config.features.syntax_highlighting {
            println!("✨ Syntax highlighting enabled");
        }
        if config.features.auto_completion {
            println!("🔮 Auto-completion enabled");
        }
        if config.features.performance_monitoring {
            println!("📊 Performance monitoring enabled");
        }
        println!("Press F1 for help, Esc to quit\n");
    }

    // Start the TUI
    let model_provider_arc =
        model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>;
    if let Err(e) = start_tui(
        model_provider_arc,
        config,
        initial_resource,
        initial_variables,
    )
    .await
    {
        eprintln!("TUI error: {e}");
        process::exit(1);
    }
}

/// Load a resource from input (file path or JSON string)
fn load_resource_from_input(input: &str) -> anyhow::Result<serde_json::Value> {
    use anyhow::Context;

    if input.starts_with('{') || input.starts_with('[') {
        // Input looks like JSON, try to parse directly
        serde_json::from_str(input).context("Failed to parse input as JSON")
    } else {
        // Input is likely a file path
        let content = std::fs::read_to_string(input).context("Failed to read input file")?;
        serde_json::from_str(&content).context("Failed to parse file content as JSON")
    }
}
