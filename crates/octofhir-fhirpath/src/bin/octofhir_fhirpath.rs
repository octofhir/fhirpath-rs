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
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::process;

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
        JsonValue::Object(serde_json::Map::new())
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

        let config = octofhir_fhirpath::model::fhirschema_provider::FhirSchemaConfig {
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
    let (functions, operators) = octofhir_fhirpath_registry::create_standard_registries();
    
    // Use the unified FhirPathEngine as default (thread-safe by design)
    let engine = octofhir_fhirpath_evaluator::FhirPathEngine::new(
        std::sync::Arc::new(functions),
        std::sync::Arc::new(operators),
        model_provider,
    );

    // Parse initial variables from command line
    let mut initial_variables = std::collections::HashMap::new();
    for var_spec in variables {
        if let Some((name, value_str)) = var_spec.split_once('=') {
            // Try to parse value as JSON first
            let value = match serde_json::from_str::<serde_json::Value>(value_str) {
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
        engine.evaluate_with_variables(expression, resource, variables).await
    };

    match result {
        Ok(result) => {
            if !quiet {
                eprintln!("Expression: {expression}");
                eprintln!("Result:");
            }

            let output = if pretty {
                match serde_json::to_string_pretty(&result) {
                    Ok(json) => json,
                    Err(_) => format!("{result:?}"),
                }
            } else {
                match serde_json::to_string(&result) {
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
