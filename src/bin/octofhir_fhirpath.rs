//! Simple CLI for FHIRPath evaluation
//!
//! A command-line interface for evaluating FHIRPath expressions against FHIR resources.

use clap::{Parser, Subcommand};
use octofhir_fhirpath::model::{FhirPathValue, FhirResource};
use octofhir_fhirpath::{FhirPathEngine, parse};
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::io::{self, Read};
use std::process;

#[derive(Parser)]
#[command(name = "octofhir-fhirpath")]
#[command(about = "Simple FHIRPath CLI for evaluating expressions against FHIR resources")]
#[command(version)]
#[command(author = "OctoFHIR Team <funyloony@gmail.com>")]
struct Cli {
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

fn main() {
    // Setup human-panic for better error messages
    human_panic::setup_panic!();

    let cli = Cli::parse();

    match cli.command {
        Commands::Evaluate {
            expression,
            input,
            pretty,
            quiet,
        } => {
            handle_evaluate(&expression, input.as_deref(), pretty, quiet);
        }
        Commands::Parse { expression, quiet } => {
            handle_parse(&expression, quiet);
        }
        Commands::Validate { expression, quiet } => {
            handle_validate(&expression, quiet);
        }
    }
}

fn handle_evaluate(expression: &str, input: Option<&str>, pretty: bool, quiet: bool) {
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
        // Read from stdin
        let mut buffer = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Error reading from stdin: {e}");
            process::exit(1);
        }
        buffer
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

    // Parse the expression first
    let ast = match parse(expression) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error parsing expression: {e}");
            process::exit(1);
        }
    };

    // Convert JSON to FhirPathValue through FhirResource
    let fhir_resource = FhirResource::from_json(resource);
    let fhir_value = FhirPathValue::Resource(fhir_resource);

    // Create FHIRPath engine and evaluate
    let engine = FhirPathEngine::new();

    match engine.evaluate(&ast, fhir_value) {
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
