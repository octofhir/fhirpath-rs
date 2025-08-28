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

//! Simple CLI for FHIRPath evaluation
//!
//! A command-line interface for evaluating FHIRPath expressions against FHIR resources.

use clap::Parser;
use octofhir_fhirpath::cli::output::{
    AnalysisOutput, EvaluationOutput, FormatterFactory, OutputMetadata, ParseOutput,
};
use octofhir_fhirpath::cli::{Cli, Commands};
use octofhir_fhirpath::model::provider::PackageSpec;
use octofhir_fhirpath::parse;
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() {
    // Setup human-panic for better error messages
    human_panic::setup_panic!();

    let cli = Cli::parse();

    // Create formatter factory
    let formatter_factory = FormatterFactory::new(cli.no_color);
    let formatter = formatter_factory.create_formatter(cli.output_format.clone());

    match cli.command {
        Commands::Evaluate {
            ref expression,
            ref input,
            ref variables,
            pretty,
        } => {
            handle_evaluate(
                expression,
                input.as_deref(),
                variables,
                pretty,
                &cli,
                &*formatter,
            )
            .await;
        }
        Commands::Parse { ref expression } => {
            handle_parse(expression, &cli, &*formatter);
        }
        Commands::Validate { ref expression } => {
            handle_validate(expression, &cli, &*formatter);
        }
        Commands::Analyze {
            ref expression,
            ref variables,
            validate_only,
            no_inference,
        } => {
            handle_analyze(
                expression,
                variables,
                validate_only,
                no_inference,
                &cli,
                &*formatter,
            )
            .await;
        }
        Commands::Repl {
            ref input,
            ref variables,
            ref history_file,
            history_size,
        } => {
            handle_repl(
                input.as_deref(),
                variables,
                history_file.as_deref(),
                history_size,
                &cli,
            )
            .await;
        }
        Commands::Server {
            port,
            ref storage,
            ref host,
            cors_all,
        } => {
            handle_server(port, storage.clone(), host.clone(), cors_all, &cli).await;
        }
    }
}

async fn handle_evaluate(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    _pretty: bool,
    cli: &Cli,
    formatter: &dyn octofhir_fhirpath::cli::output::OutputFormatter,
) {
    let start_time = Instant::now();

    // Get resource data
    let resource_data = if let Some(input_str) = input {
        // Check if input is a file path or JSON string
        if input_str.starts_with('{') || input_str.starts_with('[') || input_str.trim().is_empty() {
            // Treat as JSON string directly
            input_str.to_string()
        } else {
            // Treat as file path
            match fs::read_to_string(input_str) {
                Ok(content) => content,
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("Error reading file '{input_str}': {e}");
                    }
                    process::exit(1);
                }
            }
        }
    } else {
        // No input provided - read from stdin
        if !cli.quiet {
            eprintln!("Reading FHIR resource from stdin...");
        }

        use std::io::{self, Read};
        let mut stdin_content = String::new();
        match io::stdin().read_to_string(&mut stdin_content) {
            Ok(_) => stdin_content,
            Err(e) => {
                if !cli.quiet {
                    eprintln!("Error reading from stdin: {e}");
                }
                process::exit(1);
            }
        }
    };

    // Handle empty input case
    let resource: JsonValue = if resource_data.trim().is_empty() {
        // Use empty object for empty input
        parse_json("{}").unwrap_or_default()
    } else {
        // Parse JSON resource
        match parse_json(&resource_data) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error parsing JSON resource: {e}");
                process::exit(1);
            }
        }
    };

    // Create FHIRPath engine with specified FHIR version schema provider
    let model_provider: std::sync::Arc<dyn octofhir_fhirpath::model::provider::ModelProvider> = {
        use octofhir_fhirpath::model::{
            fhirschema_provider::FhirSchemaModelProvider, provider::FhirVersion,
        };

        let fhir_version = match cli.fhir_version.to_lowercase().as_str() {
            "r4" => FhirVersion::R4,
            "r4b" => FhirVersion::R4B,
            "r5" => FhirVersion::R5,
            _ => {
                eprintln!(
                    "⚠️ Invalid FHIR version '{}', defaulting to R4",
                    cli.fhir_version
                );
                FhirVersion::R4
            }
        };

        let mut additional_packages = Vec::new();
        for package_spec in &cli.packages {
            if let Some((name, version)) = package_spec.split_once('@') {
                additional_packages.push(PackageSpec::registry(name, version));
            } else {
                eprintln!("⚠️ Invalid package format '{package_spec}', expected 'package@version'");
            }
        }

        let config = octofhir_fhirpath_model::provider::FhirSchemaConfig {
            fhir_version,
            additional_packages,
            ..Default::default()
        };

        match FhirSchemaModelProvider::with_config(config).await {
            Ok(provider) => {
                if !cli.quiet {
                    eprintln!(
                        "✅ Initialized FHIR {} schema provider",
                        match fhir_version {
                            FhirVersion::R4 => "R4",
                            FhirVersion::R4B => "R4B",
                            FhirVersion::R5 => "R5",
                        }
                    );
                }
                std::sync::Arc::new(provider)
            }
            Err(e) => {
                if !cli.quiet {
                    eprintln!("⚠️ Failed to initialize FHIR schema provider: {e}");
                    eprintln!("🔄 Falling back to mock provider...");
                }
                std::sync::Arc::new(octofhir_fhirpath::model::MockModelProvider::empty())
            }
        }
    };
    // Create registries for the SendSafe engine using standard registries
    let registry = octofhir_fhirpath_registry::create_standard_registry().await;

    // Use the unified FhirPathEngine as default (thread-safe by design)
    let engine =
        octofhir_fhirpath_evaluator::FhirPathEngine::new(Arc::new(registry), model_provider);

    // Parse initial variables from command line
    let mut initial_variables = std::collections::HashMap::new();
    for var_spec in variables {
        if let Some((name, value_str)) = var_spec.split_once('=') {
            // Try to parse value as JSON first
            let value = match parse_json::<JsonValue>(value_str) {
                Ok(json_value) => octofhir_fhirpath::FhirPathValue::from(json_value),
                Err(_) => {
                    // If JSON parsing fails, treat as string
                    octofhir_fhirpath::FhirPathValue::String(value_str.to_string().into())
                }
            };
            initial_variables.insert(name.to_string(), value);
            if !cli.quiet {
                eprintln!("Variable set: {name} = {value_str}");
            }
        } else {
            eprintln!("⚠️ Invalid variable format '{var_spec}', expected 'name=value'");
        }
    }

    // Convert variables to correct HashMap type
    let variables: std::collections::HashMap<String, octofhir_fhirpath::FhirPathValue> =
        initial_variables.into_iter().collect();

    // Use the appropriate evaluation method based on whether variables are provided
    let result = if variables.is_empty() {
        engine.evaluate(expression, resource).await
    } else {
        engine
            .evaluate_with_variables(expression, resource, variables)
            .await
    };

    let execution_time = start_time.elapsed();

    let output = match result {
        Ok(result_value) => EvaluationOutput {
            success: true,
            result: Some(result_value),
            error: None,
            expression: expression.to_string(),
            execution_time,
            metadata: OutputMetadata {
                cache_hits: 0,  // TODO: Track cache hits from engine
                ast_nodes: 0,   // TODO: Track AST nodes
                memory_used: 0, // TODO: Track memory usage
            },
        },
        Err(e) => EvaluationOutput {
            success: false,
            result: None,
            error: Some((Box::new(e) as Box<dyn std::error::Error>).into()),
            expression: expression.to_string(),
            execution_time,
            metadata: OutputMetadata::default(),
        },
    };

    match formatter.format_evaluation(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error formatting output: {}", e);
            process::exit(1);
        }
    }
}

fn handle_parse(
    expression: &str,
    cli: &Cli,
    formatter: &dyn octofhir_fhirpath::cli::output::OutputFormatter,
) {
    let output = match parse(expression) {
        Ok(ast) => ParseOutput {
            success: true,
            ast: Some(ast),
            error: None,
            expression: expression.to_string(),
            metadata: OutputMetadata {
                cache_hits: 0,
                ast_nodes: 1, // TODO: Count AST nodes properly
                memory_used: 0,
            },
        },
        Err(e) => ParseOutput {
            success: false,
            ast: None,
            error: Some(e.into()),
            expression: expression.to_string(),
            metadata: OutputMetadata::default(),
        },
    };

    match formatter.format_parse(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error formatting output: {}", e);
            process::exit(1);
        }
    }
}

fn handle_validate(
    expression: &str,
    cli: &Cli,
    formatter: &dyn octofhir_fhirpath::cli::output::OutputFormatter,
) {
    // Validate is basically the same as parse but focuses on success/failure
    let output = match parse(expression) {
        Ok(ast) => ParseOutput {
            success: true,
            ast: Some(ast),
            error: None,
            expression: expression.to_string(),
            metadata: OutputMetadata {
                cache_hits: 0,
                ast_nodes: 1,
                memory_used: 0,
            },
        },
        Err(e) => ParseOutput {
            success: false,
            ast: None,
            error: Some(e.into()),
            expression: expression.to_string(),
            metadata: OutputMetadata::default(),
        },
    };

    match formatter.format_parse(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error formatting output: {}", e);
            process::exit(1);
        }
    }
}

async fn handle_analyze(
    expression: &str,
    variables: &[String],
    validate_only: bool,
    _no_inference: bool,
    cli: &Cli,
    formatter: &dyn octofhir_fhirpath::cli::output::OutputFormatter,
) {
    use octofhir_fhirpath::FhirPathEngineWithAnalyzer;
    use octofhir_fhirpath_model::FhirSchemaModelProvider;
    use octofhir_fhirpath_registry::create_standard_registry;
    use std::sync::Arc;

    // Always use FhirSchemaModelProvider for comprehensive field validation
    if !cli.quiet {
        eprintln!("🔧 Initializing FhirSchemaModelProvider for comprehensive field validation...");
    }

    let model_provider: Box<dyn octofhir_fhirpath_model::provider::ModelProvider> =
        match FhirSchemaModelProvider::new().await {
            Ok(provider) => {
                if !cli.quiet {
                    eprintln!("✅ FhirSchemaModelProvider initialized successfully");
                }
                Box::new(provider)
            }
            Err(e) => {
                eprintln!(
                    "❌ CRITICAL: Failed to create FhirSchemaModelProvider: {}",
                    e
                );
                eprintln!("💡 This is required for proper FHIR field validation.");
                eprintln!("🔧 Please ensure FHIR schema data is available and try again.");
                process::exit(1);
            }
        };

    let function_registry = Arc::new(create_standard_registry().await);
    let engine =
        match FhirPathEngineWithAnalyzer::with_full_analysis(model_provider, function_registry)
            .await
        {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("❌ Failed to create analyzer engine: {}", e);
                process::exit(1);
            }
        };

    let output = if validate_only {
        // Validation only
        match engine.validate_expression(expression).await {
            Ok(validation_errors) => AnalysisOutput {
                success: validation_errors.is_empty(),
                analysis: None,
                validation_errors,
                error: None,
                expression: expression.to_string(),
                metadata: OutputMetadata::default(),
            },
            Err(e) => AnalysisOutput {
                success: false,
                analysis: None,
                validation_errors: vec![],
                error: Some((Box::new(e) as Box<dyn std::error::Error>).into()),
                expression: expression.to_string(),
                metadata: OutputMetadata::default(),
            },
        }
    } else {
        // Full analysis
        match engine.analyze_expression(expression).await {
            Ok(Some(analysis)) => AnalysisOutput {
                success: analysis.validation_errors.is_empty(),
                analysis: Some(analysis),
                validation_errors: vec![], // Validation errors are in analysis.validation_errors
                error: None,
                expression: expression.to_string(),
                metadata: OutputMetadata::default(),
            },
            Ok(None) => AnalysisOutput {
                success: false,
                analysis: None,
                validation_errors: vec![],
                error: Some(
                    Box::<dyn std::error::Error>::from("No analyzer available".to_string()).into(),
                ),
                expression: expression.to_string(),
                metadata: OutputMetadata::default(),
            },
            Err(e) => AnalysisOutput {
                success: false,
                analysis: None,
                validation_errors: vec![],
                error: Some((Box::new(e) as Box<dyn std::error::Error>).into()),
                expression: expression.to_string(),
                metadata: OutputMetadata::default(),
            },
        }
    };

    match formatter.format_analysis(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error formatting output: {}", e);
            process::exit(1);
        }
    }
}

async fn handle_repl(
    input: Option<&str>,
    variables: &[String],
    history_file: Option<&str>,
    history_size: usize,
    cli: &Cli,
) {
    use octofhir_fhirpath::cli::repl::{ReplConfig, start_repl};
    use octofhir_fhirpath::model::{
        fhirschema_provider::FhirSchemaModelProvider,
        provider::{FhirVersion, ModelProvider},
    };
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    // Parse initial variables
    let initial_variables: Vec<(String, String)> = variables
        .iter()
        .filter_map(|var| {
            if let Some(eq_pos) = var.find('=') {
                let name = var[..eq_pos].to_string();
                let value = var[eq_pos + 1..].to_string();
                Some((name, value))
            } else {
                eprintln!(
                    "Warning: Invalid variable format '{}', expected 'name=value'",
                    var
                );
                None
            }
        })
        .collect();

    // Load initial resource if provided
    let initial_resource = if let Some(input_path) = input {
        match fs::read_to_string(input_path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => Some(json),
                Err(e) => {
                    eprintln!("Error parsing initial resource '{}': {}", input_path, e);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading initial resource '{}': {}", input_path, e);
                process::exit(1);
            }
        }
    } else {
        None
    };

    // Create model provider with specified FHIR version
    let fhir_version = match cli.fhir_version.to_lowercase().as_str() {
        "r4" => FhirVersion::R4,
        "r4b" => FhirVersion::R4B,
        "r5" => FhirVersion::R5,
        _ => {
            eprintln!(
                "⚠️ Invalid FHIR version '{}', defaulting to R4",
                cli.fhir_version
            );
            FhirVersion::R4
        }
    };

    let model_provider = match fhir_version {
        FhirVersion::R4 => match FhirSchemaModelProvider::r4().await {
            Ok(provider) => std::sync::Arc::new(provider) as Arc<dyn ModelProvider>,
            Err(e) => {
                eprintln!(
                    "Failed to create model provider for FHIR {:?}: {}",
                    fhir_version, e
                );
                process::exit(1);
            }
        },
        FhirVersion::R4B => match FhirSchemaModelProvider::r4b().await {
            Ok(provider) => std::sync::Arc::new(provider) as Arc<dyn ModelProvider>,
            Err(e) => {
                eprintln!(
                    "Failed to create model provider for FHIR {:?}: {}",
                    fhir_version, e
                );
                process::exit(1);
            }
        },
        FhirVersion::R5 => match FhirSchemaModelProvider::r5().await {
            Ok(provider) => std::sync::Arc::new(provider) as Arc<dyn ModelProvider>,
            Err(e) => {
                eprintln!(
                    "Failed to create model provider for FHIR {:?}: {}",
                    fhir_version, e
                );
                process::exit(1);
            }
        },
    };

    // Create REPL configuration
    let mut repl_config = ReplConfig {
        color_output: !cli.no_color,
        show_types: cli.verbose,
        history_size,
        ..Default::default()
    };

    // Set history file
    if let Some(history_path) = history_file {
        repl_config.history_file = Some(PathBuf::from(history_path));
    } else {
        // Use default history file location
        if let Some(home_dir) = dirs::home_dir() {
            repl_config.history_file = Some(home_dir.join(".fhirpath_history"));
        }
    }

    // Start REPL
    if let Err(e) = start_repl(
        model_provider,
        repl_config,
        initial_resource,
        initial_variables,
    )
    .await
    {
        eprintln!("REPL error: {}", e);
        process::exit(1);
    }
}

async fn handle_server(port: u16, storage: PathBuf, host: String, cors_all: bool, cli: &Cli) {
    use octofhir_fhirpath::cli::server::{config::ServerConfig, start_server};

    // Initialize tracing for server logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    if !cli.quiet {
        println!("🚀 Starting FHIRPath HTTP server...");
        println!("📍 Host: {}", host);
        println!("🔌 Port: {}", port);
        println!("📁 Storage: {}", storage.display());
        if cors_all {
            println!("🌐 CORS: Enabled for all origins (development mode)");
        }
    }

    // Create server configuration
    let config = ServerConfig::new(port, host, storage, cors_all);

    // Ensure storage directory exists
    if let Err(e) = config.ensure_storage_dir().await {
        eprintln!("❌ Failed to create storage directory: {}", e);
        process::exit(1);
    }

    // Start the server
    if let Err(e) = start_server(config).await {
        eprintln!("❌ Server error: {}", e);
        process::exit(1);
    }
}
