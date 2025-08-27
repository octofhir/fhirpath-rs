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

use clap::{Parser, Subcommand};
use octofhir_fhirpath::model::provider::PackageSpec;
use octofhir_fhirpath::parse;
use sonic_rs::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::process;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "octofhir-fhirpath")]
#[command(about = "Simple FHIRPath CLI for evaluating expressions against FHIR resources")]
#[command(version)]
#[command(author = "OctoFHIR Team <funyloony@gmail.com>")]
struct Cli {
    /// FHIR version to use (r4, r4b, r5)
    #[arg(long, value_name = "VERSION", default_value = "r4")]
    fhir_version: String,
    /// Additional FHIR packages to load (format: package@version)
    #[arg(long = "package", value_name = "PACKAGE")]
    packages: Vec<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate FHIRPath expression against a FHIR resource
    Evaluate {
        /// FHIRPath expression to evaluate
        expression: String,
        /// JSON file containing FHIR resource, or JSON string directly (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        /// Initial variables to set in format var=value (can be used multiple times)
        #[arg(short, long = "variable")]
        variables: Vec<String>,
        /// Pretty-print JSON output
        #[arg(short, long)]
        pretty: bool,
        /// Suppress informational messages
        #[arg(short, long)]
        quiet: bool,
    },
    /// Parse and validate FHIRPath expression syntax
    Parse {
        /// FHIRPath expression to parse
        expression: String,
        /// Suppress informational messages
        #[arg(short, long)]
        quiet: bool,
    },
    /// Validate FHIRPath expression syntax (alias for parse)
    Validate {
        /// FHIRPath expression to validate
        expression: String,
        /// Suppress informational messages
        #[arg(short, long)]
        quiet: bool,
    },
    /// Analyze FHIRPath expressions with comprehensive FHIR field validation
    Analyze {
        /// FHIRPath expression to analyze
        expression: String,
        /// Only validate, don't analyze types
        #[arg(long)]
        validate_only: bool,
        /// Disable type inference
        #[arg(long)]
        no_inference: bool,
        /// Suppress informational messages
        #[arg(short, long)]
        quiet: bool,
    },
}

#[tokio::main]
async fn main() {
    // Setup human-panic for better error messages
    human_panic::setup_panic!();

    let cli = Cli::parse();

    match cli.command {
        Commands::Evaluate {
            ref expression,
            ref input,
            ref variables,
            pretty,
            quiet,
        } => {
            handle_evaluate(expression, input.as_deref(), variables, pretty, quiet, &cli).await;
        }
        Commands::Parse { expression, quiet } => {
            handle_parse(&expression, quiet);
        }
        Commands::Validate { expression, quiet } => {
            handle_validate(&expression, quiet);
        }
        Commands::Analyze {
            ref expression,
            validate_only,
            no_inference,
            quiet,
        } => {
            handle_analyze(expression, validate_only, no_inference, quiet).await;
        }
    }
}

async fn handle_evaluate(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    pretty: bool,
    quiet: bool,
    cli: &Cli,
) {
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
                    eprintln!("Error reading file '{input_str}': {e}");
                    process::exit(1);
                }
            }
        }
    } else {
        // No input provided - read from stdin
        if !quiet {
            eprintln!("Reading FHIR resource from stdin...");
        }

        use std::io::{self, Read};
        let mut stdin_content = String::new();
        match io::stdin().read_to_string(&mut stdin_content) {
            Ok(_) => stdin_content,
            Err(e) => {
                eprintln!("Error reading from stdin: {e}");
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
                if !quiet {
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
                if !quiet {
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
            if !quiet {
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

    match result {
        Ok(result) => {
            if !quiet {
                eprintln!("Expression: {expression}");
                eprintln!("Result:");
            }

            let output = if pretty {
                match sonic_rs::to_string_pretty(&result) {
                    Ok(json) => json,
                    Err(_) => format!("{result:?}"),
                }
            } else {
                match sonic_rs::to_string(&result) {
                    Ok(json) => json,
                    Err(_) => format!("{result:?}"),
                }
            };

            println!("{output}");
        }
        Err(e) => {
            eprintln!("Error evaluating expression: {e}");
            process::exit(1);
        }
    }
}

fn handle_parse(expression: &str, quiet: bool) {
    match parse(expression) {
        Ok(ast) => {
            if !quiet {
                println!("✓ Expression parsed successfully");
                println!("Expression: {expression}");
                println!("AST: {ast:?}");
            } else {
                println!("OK");
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {e}");
            process::exit(1);
        }
    }
}

fn handle_validate(expression: &str, quiet: bool) {
    match parse(expression) {
        Ok(_) => {
            if !quiet {
                println!("✓ Expression is valid");
                println!("Expression: {expression}");
            } else {
                println!("VALID");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("✗ Invalid expression: {e}");
                eprintln!("Expression: {expression}");
            } else {
                eprintln!("INVALID");
            }
            process::exit(1);
        }
    }
}

async fn handle_analyze(
    expression: &str,
    validate_only: bool,
    no_inference: bool,
    quiet: bool,
) {
    use octofhir_fhirpath::FhirPathEngineWithAnalyzer;
    use octofhir_fhirpath_model::FhirSchemaModelProvider;
    use octofhir_fhirpath_registry::create_standard_registry;
    use std::sync::Arc;

    // Always use FhirSchemaModelProvider for comprehensive field validation
    if !quiet {
        println!("🔧 Initializing FhirSchemaModelProvider for comprehensive field validation...");
    }

    let model_provider: Box<dyn octofhir_fhirpath_model::provider::ModelProvider> =
        match FhirSchemaModelProvider::new().await {
            Ok(provider) => {
                if !quiet {
                    println!("✅ FhirSchemaModelProvider initialized successfully");
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
    let engine = match FhirPathEngineWithAnalyzer::with_full_analysis(model_provider, function_registry).await {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("❌ Failed to create analyzer engine: {}", e);
            process::exit(1);
        }
    };

    if validate_only {
        // Validation only
        let errors = match engine.validate_expression(expression).await {
            Ok(errors) => errors,
            Err(e) => {
                eprintln!("❌ Error during validation: {}", e);
                process::exit(1);
            }
        };

        if errors.is_empty() {
            if !quiet {
                println!("✅ Expression is valid");
            } else {
                println!("VALID");
            }
        } else {
            if !quiet {
                println!("❌ Validation errors:");
                for error in errors {
                    println!("  - {} (type: {:?})", error.message, error.error_type);
                    if !error.suggestions.is_empty() {
                        println!("    Suggestions: {}", error.suggestions.join(", "));
                    }
                }
            } else {
                println!("INVALID");
            }
            process::exit(1);
        }
    } else {
        // Full analysis
        let analysis = match engine.analyze_expression(expression).await {
            Ok(Some(analysis)) => analysis,
            Ok(None) => {
                if !quiet {
                    println!("⚠️  No analyzer available");
                } else {
                    println!("NO_ANALYZER");
                }
                process::exit(1);
            }
            Err(e) => {
                eprintln!("❌ Error during analysis: {}", e);
                process::exit(1);
            }
        };

        if !quiet {
            println!("📊 Analysis Results for: {expression}");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        if !analysis.validation_errors.is_empty() {
            if !quiet {
                println!("\n❌ Validation Errors:");
                for error in analysis.validation_errors {
                    let icon = match error.error_type {
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidField => "🔍",
                        octofhir_fhirpath_analyzer::ValidationErrorType::DeprecatedField => "⚠️",
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidResourceType => {
                            "🏥"
                        }
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidFunction => "🔧",
                        _ => "❗",
                    };
                    println!("  {} {}", icon, error.message);
                    if !error.suggestions.is_empty() {
                        println!("    💡 Suggestions: {}", error.suggestions.join(", "));
                    }
                }
            }
            process::exit(1);
        }

        if !no_inference && !analysis.type_annotations.is_empty() && !quiet {
            println!("\n🔍 Type Annotations:");
            for (node_id, semantic_info) in analysis.type_annotations {
                println!("  Node {node_id}: ");
                if let Some(fhir_type) = semantic_info.fhir_path_type {
                    println!("    FHIRPath Type: {fhir_type}");
                }
                if let Some(model_type) = semantic_info.model_type {
                    println!("    FHIR Model Type: {model_type}");
                }
                println!("    Cardinality: {:?}", semantic_info.cardinality);
                println!("    Confidence: {:?}", semantic_info.confidence);
            }
        }

        if !analysis.function_calls.is_empty() && !quiet {
            println!("\n🔧 Function Calls:");
            for func_analysis in analysis.function_calls {
                println!(
                    "  - {} ({})",
                    func_analysis.function_name, func_analysis.signature.description
                );
                if !func_analysis.validation_errors.is_empty() {
                    for error in func_analysis.validation_errors {
                        println!("    ⚠️  {}", error.message);
                    }
                }
            }
        }

        if !quiet {
            println!("\n✅ Analysis complete");
        } else {
            println!("OK");
        }
    }
}
