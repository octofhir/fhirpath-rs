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

//! Custom output formatter example
//!
//! This example demonstrates how to:
//! - Use different output formats (Pretty, JSON, Raw)
//! - Create custom formatting logic
//! - Handle output programmatically

use fhirpath_cli::{
    EmbeddedModelProvider,
    cli::context::CliContext,
    cli::output::{EvaluationOutput, FormatterFactory, OutputFormat, OutputMetadata},
};
use octofhir_fhir_model::provider::FhirVersion;
use octofhir_fhirpath::{Collection, FhirPathValue};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("FHIRPath Custom Formatter Example");
    println!("==================================\n");

    // Initialize model provider
    let model_provider = Arc::new(EmbeddedModelProvider::new(FhirVersion::R4));

    // Create sample evaluation result
    let patient_json = serde_json::json!({
        "resourceType": "Patient",
        "id": "example",
        "active": true,
        "name": [
            {
                "use": "official",
                "family": "Doe",
                "given": ["John", "Robert"]
            }
        ]
    });

    // Example 1: Pretty format (default)
    println!("Example 1: Pretty Format (with colors and diagnostics)");
    println!("-------------------------------------------------------");

    let context = CliContext::new(
        OutputFormat::Pretty,
        false, // Enable colors
        false,
        false,
        "r4".to_string(),
        vec![],
        false,
    );

    let formatter = FormatterFactory::new(false).create_formatter(OutputFormat::Pretty, None);

    let output = create_sample_output(
        "Patient.name.family",
        vec![FhirPathValue::string("Doe".to_string())],
        Duration::from_millis(5),
    );

    match formatter.format_evaluation(&output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    // Example 2: JSON format
    println!("Example 2: JSON Format (structured output)");
    println!("-------------------------------------------");

    let json_formatter = FormatterFactory::new(false).create_formatter(OutputFormat::Json, None);

    match json_formatter.format_evaluation(&output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    // Example 3: Raw format
    println!("Example 3: Raw Format (plain text)");
    println!("-----------------------------------");

    let raw_formatter = FormatterFactory::new(false).create_formatter(OutputFormat::Raw, None);

    match raw_formatter.format_evaluation(&output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    // Example 4: Format with error
    println!("Example 4: Formatting Error Output");
    println!("-----------------------------------");

    let error_output = EvaluationOutput {
        success: false,
        result: None,
        result_with_metadata: None,
        error: Some(octofhir_fhirpath::core::FhirPathError::parse_error(
            octofhir_fhirpath::core::error_code::FP0001,
            "Unexpected token",
            "Patient.(",
            Some(octofhir_fhirpath::core::SourceLocation {
                line: 1,
                column: 9,
                offset: 8,
                length: 1,
            }),
        )),
        expression: "Patient.(".to_string(),
        execution_time: Duration::from_millis(1),
        metadata: OutputMetadata::default(),
    };

    let pretty_formatter =
        FormatterFactory::new(false).create_formatter(OutputFormat::Pretty, None);

    match pretty_formatter.format_evaluation(&error_output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    // Example 5: Format collection of values
    println!("Example 5: Formatting Collection of Values");
    println!("-------------------------------------------");

    let collection_output = create_sample_output(
        "Patient.name.given",
        vec![
            FhirPathValue::string("John".to_string()),
            FhirPathValue::string("Robert".to_string()),
        ],
        Duration::from_millis(3),
    );

    println!("JSON format:");
    match json_formatter.format_evaluation(&collection_output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\nRaw format:");
    match raw_formatter.format_evaluation(&collection_output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    // Example 6: Format boolean result
    println!("Example 6: Formatting Boolean Result");
    println!("-------------------------------------");

    let bool_output = create_sample_output(
        "Patient.active",
        vec![FhirPathValue::boolean(true)],
        Duration::from_millis(2),
    );

    println!("JSON format:");
    match json_formatter.format_evaluation(&bool_output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\nRaw format:");
    match raw_formatter.format_evaluation(&bool_output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    // Example 7: Format empty result
    println!("Example 7: Formatting Empty Result");
    println!("-----------------------------------");

    let empty_output =
        create_sample_output("Patient.invalidField", vec![], Duration::from_millis(1));

    println!("JSON format:");
    match json_formatter.format_evaluation(&empty_output) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n");

    println!("✅ All formatter examples completed!");

    println!("\n💡 Tips:");
    println!("   - Use JSON format for machine-readable output");
    println!("   - Use Raw format for simple text output (e.g., in scripts)");
    println!("   - Use Pretty format for human-readable diagnostics");
    println!("   - Disable colors with --no-color flag or NO_COLOR env var");

    Ok(())
}

/// Helper function to create sample evaluation output
fn create_sample_output(
    expression: &str,
    values: Vec<FhirPathValue>,
    execution_time: Duration,
) -> EvaluationOutput {
    let collection = Collection::from_values(values);

    EvaluationOutput {
        success: true,
        result: Some(collection),
        result_with_metadata: None,
        error: None,
        expression: expression.to_string(),
        execution_time,
        metadata: OutputMetadata {
            cache_hits: 0,
            ast_nodes: 0,
            memory_used: 0,
        },
    }
}
