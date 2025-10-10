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

//! Basic FHIRPath evaluation example
//!
//! This example demonstrates how to:
//! - Create a CLI context
//! - Load a FHIR resource
//! - Evaluate a FHIRPath expression
//! - Handle the output programmatically

use fhirpath_cli::{
    EmbeddedModelProvider, cli::context::CliContext, cli::handlers, cli::output::OutputFormat,
};
use octofhir_fhir_model::provider::FhirVersion;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("FHIRPath Basic Evaluation Example");
    println!("==================================\n");

    // Create CLI context with default settings
    let context = CliContext::new(
        OutputFormat::Pretty, // Use pretty output format
        false,                // no_color = false (enable colors)
        false,                // quiet = false (show messages)
        false,                // verbose = false (normal verbosity)
        "r4".to_string(),     // Use FHIR R4
        vec![],               // No additional packages
        false,                // profile = false (no profiling)
    );

    // Initialize model provider
    println!("Initializing FHIR R4 model provider...");
    let model_provider = Arc::new(EmbeddedModelProvider::new(FhirVersion::R4));
    println!("✅ Model provider initialized\n");

    // Example 1: Simple field access
    println!("Example 1: Extract patient active status");
    println!("-----------------------------------------");
    let patient_json = r#"
    {
        "resourceType": "Patient",
        "id": "example",
        "active": true,
        "name": [{"family": "Doe", "given": ["John"]}]
    }
    "#;

    // Create a temporary file with the resource
    let temp_file = tempfile::NamedTempFile::new()?;
    std::fs::write(temp_file.path(), patient_json)?;

    handlers::handle_evaluate(
        "Patient.active",
        Some(temp_file.path().to_str().unwrap()),
        &[],
        false,
        false,
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    // Example 2: Extract nested field
    println!("Example 2: Extract patient family name");
    println!("---------------------------------------");
    handlers::handle_evaluate(
        "Patient.name.family",
        Some(temp_file.path().to_str().unwrap()),
        &[],
        false,
        false,
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    // Example 3: Using where() function
    println!("Example 3: Filter with where() function");
    println!("---------------------------------------");
    let patient_with_multiple_names = r#"
    {
        "resourceType": "Patient",
        "name": [
            {"use": "official", "family": "Doe", "given": ["John"]},
            {"use": "nickname", "given": ["Johnny"]}
        ]
    }
    "#;

    let temp_file2 = tempfile::NamedTempFile::new()?;
    std::fs::write(temp_file2.path(), patient_with_multiple_names)?;

    handlers::handle_evaluate(
        "Patient.name.where(use = 'official').family",
        Some(temp_file2.path().to_str().unwrap()),
        &[],
        false,
        false,
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    // Example 4: Using count() function
    println!("Example 4: Count items with count() function");
    println!("--------------------------------------------");
    handlers::handle_evaluate(
        "Patient.name.count()",
        Some(temp_file2.path().to_str().unwrap()),
        &[],
        false,
        false,
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    // Example 5: Complex expression
    println!("Example 5: Complex expression - concatenate names");
    println!("-------------------------------------------------");
    handlers::handle_evaluate(
        "Patient.name.given.first() + ' ' + Patient.name.family.first()",
        Some(temp_file.path().to_str().unwrap()),
        &[],
        false,
        false,
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    // Example 6: Using variables
    println!("Example 6: Using variables in expressions");
    println!("-----------------------------------------");
    handlers::handle_evaluate(
        "Patient.name.where(use = %nameUse).family",
        Some(temp_file2.path().to_str().unwrap()),
        &["nameUse=official".to_string()],
        false,
        false,
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    // Example 7: With static analysis
    println!("Example 7: Evaluation with static analysis");
    println!("------------------------------------------");
    handlers::handle_evaluate(
        "Patient.name.family",
        Some(temp_file.path().to_str().unwrap()),
        &[],
        false,
        true, // Enable static analysis
        &context,
        &model_provider,
    )
    .await;

    println!("\n");

    println!("✅ All examples completed successfully!");

    Ok(())
}
