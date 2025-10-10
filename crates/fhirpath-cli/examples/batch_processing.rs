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

//! Batch processing example
//!
//! This example demonstrates how to:
//! - Process multiple FHIR resources
//! - Use glob patterns to find files
//! - Collect and aggregate results
//! - Handle errors gracefully

use fhirpath_cli::{
    EmbeddedModelProvider, cli::context::CliContext, cli::handlers, cli::output::OutputFormat,
};
use octofhir_fhir_model::provider::FhirVersion;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("FHIRPath Batch Processing Example");
    println!("==================================\n");

    // Create CLI context with JSON output for easier parsing
    let context = CliContext::new(
        OutputFormat::Json, // Use JSON for programmatic processing
        false,
        true, // quiet mode to reduce noise
        false,
        "r4".to_string(),
        vec![],
        false,
    );

    // Initialize model provider
    let model_provider = Arc::new(EmbeddedModelProvider::new(FhirVersion::R4));

    // Create temporary directory with test files
    let temp_dir = tempfile::TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create multiple patient files
    println!("Creating test patient files...");

    let patients = vec![
        r#"{"resourceType": "Patient", "id": "1", "active": true, "name": [{"family": "Smith", "given": ["Alice"]}]}"#,
        r#"{"resourceType": "Patient", "id": "2", "active": false, "name": [{"family": "Jones", "given": ["Bob"]}]}"#,
        r#"{"resourceType": "Patient", "id": "3", "active": true, "name": [{"family": "Brown", "given": ["Charlie"]}]}"#,
        r#"{"resourceType": "Patient", "id": "4", "active": true, "name": [{"family": "Davis", "given": ["Diana"]}]}"#,
        r#"{"resourceType": "Patient", "id": "5", "active": false, "name": [{"family": "Wilson", "given": ["Eve"]}]}"#,
    ];

    for (i, patient) in patients.iter().enumerate() {
        let file_path = temp_path.join(format!("patient_{}.json", i + 1));
        std::fs::write(&file_path, patient)?;
    }

    println!("✅ Created {} patient files\n", patients.len());

    // Example 1: Process all files to check active status
    println!("Example 1: Check active status for all patients");
    println!("------------------------------------------------");

    let pattern = temp_path.join("*.json").to_string_lossy().to_string();
    println!("Pattern: {}\n", pattern);

    // Note: In actual CLI usage, you would use --batch flag
    // Here we demonstrate processing individual files
    for i in 1..=patients.len() {
        let file_path = temp_path.join(format!("patient_{}.json", i));
        print!("Patient {}: ", i);
        handlers::handle_evaluate(
            "Patient.active",
            Some(file_path.to_str().unwrap()),
            &[],
            false,
            false,
            &context,
            &model_provider,
        )
        .await;
    }

    println!("\n");

    // Example 2: Extract all family names
    println!("Example 2: Extract family names from all patients");
    println!("-------------------------------------------------");

    for i in 1..=patients.len() {
        let file_path = temp_path.join(format!("patient_{}.json", i));
        print!("Patient {}: ", i);
        handlers::handle_evaluate(
            "Patient.name.family",
            Some(file_path.to_str().unwrap()),
            &[],
            false,
            false,
            &context,
            &model_provider,
        )
        .await;
    }

    println!("\n");

    // Example 3: Count names in each patient record
    println!("Example 3: Count names in each patient record");
    println!("----------------------------------------------");

    for i in 1..=patients.len() {
        let file_path = temp_path.join(format!("patient_{}.json", i));
        print!("Patient {}: ", i);
        handlers::handle_evaluate(
            "Patient.name.count()",
            Some(file_path.to_str().unwrap()),
            &[],
            false,
            false,
            &context,
            &model_provider,
        )
        .await;
    }

    println!("\n");

    // Example 4: Complex aggregation - collect all given names
    println!("Example 4: Collect all given names across patients");
    println!("---------------------------------------------------");

    for i in 1..=patients.len() {
        let file_path = temp_path.join(format!("patient_{}.json", i));
        print!("Patient {}: ", i);
        handlers::handle_evaluate(
            "Patient.name.given",
            Some(file_path.to_str().unwrap()),
            &[],
            false,
            false,
            &context,
            &model_provider,
        )
        .await;
    }

    println!("\n");

    // Example 5: Filter and process
    println!("Example 5: Find active patients only");
    println!("-------------------------------------");

    for i in 1..=patients.len() {
        let file_path = temp_path.join(format!("patient_{}.json", i));

        // First check if active
        // In real usage, you'd parse the JSON output to make decisions
        print!("Checking patient {}: ", i);
        handlers::handle_evaluate(
            "Patient.active",
            Some(file_path.to_str().unwrap()),
            &[],
            false,
            false,
            &context,
            &model_provider,
        )
        .await;
    }

    println!("\n");

    // Example 6: Validate all files
    println!("Example 6: Validate required fields in all patients");
    println!("----------------------------------------------------");

    for i in 1..=patients.len() {
        let file_path = temp_path.join(format!("patient_{}.json", i));
        print!("Patient {}: ", i);
        handlers::handle_evaluate(
            "Patient.name.exists() and Patient.id.exists()",
            Some(file_path.to_str().unwrap()),
            &[],
            false,
            false,
            &context,
            &model_provider,
        )
        .await;
    }

    println!("\n");

    println!("✅ Batch processing examples completed!");
    println!("\n💡 Tip: Use the CLI directly with --batch flag for glob patterns:");
    println!("   octofhir-fhirpath evaluate \"Patient.active\" --batch \"patients/*.json\"");

    Ok(())
}
