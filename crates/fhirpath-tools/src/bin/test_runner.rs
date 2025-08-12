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

//! Test runner binary for running individual FHIRPath test files
//!
//! Usage: cargo run --bin test_runner <test_file.json>

use octofhir_fhirpath::FhirPathValue;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

/// A single test case within a test suite
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct TestCase {
    pub name: String,
    pub expression: String,
    #[serde(default)]
    pub input: Option<Value>,
    pub inputfile: Option<String>,
    pub expected: Value,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A test suite containing multiple test cases
#[derive(serde::Deserialize)]
struct TestSuite {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    pub tests: Vec<TestCase>,
}

fn load_input_data(inputfile: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let specs_dir = Path::new("specs/fhirpath/tests/input");
    let input_path = specs_dir.join(inputfile);

    let content = fs::read_to_string(&input_path)?;
    let data: Value = serde_json::from_str(&content)?;
    Ok(data)
}

/// Compare expected result with actual result
/// Simplified comparison with proper handling of FHIRPath collection semantics
fn compare_results(expected: &Value, actual: &FhirPathValue) -> bool {
    // Convert actual to JSON for uniform comparison
    let actual_json = match serde_json::to_value(actual) {
        Ok(json) => json,
        Err(_) => return false,
    };

    // Direct comparison first - handles most cases
    if expected == &actual_json {
        return true;
    }

    // FHIRPath collection handling: expected single value should match [single_value]
    match (expected, &actual_json) {
        // Test expects single value, actual is collection with one item
        (expected_single, Value::Array(actual_arr)) if actual_arr.len() == 1 => {
            expected_single == &actual_arr[0]
        }
        // Test expects array, actual is single value (shouldn't happen with new spec compliance but handle it)
        (Value::Array(expected_arr), actual_single) if expected_arr.len() == 1 => {
            &expected_arr[0] == actual_single
        }
        // Both empty
        (Value::Array(expected_arr), Value::Null) if expected_arr.is_empty() => true,
        (Value::Null, Value::Array(actual_arr)) if actual_arr.is_empty() => true,
        // Default: no match
        _ => false,
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <test_file.json>", args[0]);
        process::exit(1);
    }

    let test_file = &args[1];
    println!("🧪 Running FHIRPath tests from: {test_file}");

    // Load test suite
    let content = match fs::read_to_string(test_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read test file: {e}");
            process::exit(1);
        }
    };

    let test_suite: TestSuite = match serde_json::from_str(&content) {
        Ok(suite) => suite,
        Err(e) => {
            eprintln!("❌ Failed to parse test file: {e}");
            process::exit(1);
        }
    };

    println!("📝 Test Suite: {}", test_suite.name);
    if let Some(desc) = &test_suite.description {
        println!("📋 Description: {desc}");
    }
    if let Some(source) = &test_suite.source {
        println!("📋 Source: {source}");
    }
    println!("🔢 Total tests: {}", test_suite.tests.len());
    println!();

    // Create FHIR R5 schema provider for accurate type checking and conformance validation
    println!("📋 Initializing FHIR R5 schema provider...");
    let model_provider: std::sync::Arc<dyn octofhir_fhirpath::model::provider::ModelProvider> =
        match octofhir_fhirpath::model::fhirschema_provider::FhirSchemaModelProvider::r5().await {
            Ok(provider) => {
                println!("✅ FHIR R5 schema provider initialized successfully!");
                std::sync::Arc::new(provider)
            }
            Err(e) => {
                eprintln!("⚠️ Failed to initialize FHIR R5 schema provider: {e}");
                eprintln!("🔄 Falling back to mock provider...");
                std::sync::Arc::new(
                    octofhir_fhirpath::model::mock_provider::MockModelProvider::new(),
                )
            }
        };
    let mut engine =
        octofhir_fhirpath::engine::IntegratedFhirPathEngine::new(model_provider.clone());
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = 0;

    for test_case in &test_suite.tests {
        print!("Running {} ... ", test_case.name);

        // Load input data
        let input_data = if let Some(ref inputfile) = test_case.inputfile {
            match load_input_data(inputfile) {
                Ok(data) => data,
                Err(e) => {
                    println!("⚠️ ERROR: Failed to load input file {inputfile}: {e}");
                    errors += 1;
                    continue;
                }
            }
        } else if let Some(ref input) = test_case.input {
            input.clone()
        } else {
            Value::Null
        };

        // Evaluate expression
        let result = match engine.evaluate(&test_case.expression, input_data).await {
            Ok(result) => result,
            Err(e) => {
                println!("⚠️ ERROR: {e}");
                errors += 1;
                continue;
            }
        };

        // Compare results
        if compare_results(&test_case.expected, &result) {
            println!("✅ PASS");
            passed += 1;
        } else {
            println!("❌ FAIL");
            println!("   Expression: {}", test_case.expression);
            if let Some(inputfile) = &test_case.inputfile {
                println!("   Input file: {inputfile}");
            }
            let expected_json =
                serde_json::to_string_pretty(&test_case.expected).unwrap_or_default();
            let actual_json = match serde_json::to_value(&result) {
                Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| format!("{result:?}")),
                Err(_) => format!("{result:?}"),
            };
            println!("   Expected: {expected_json}");
            println!("   Actual:   {actual_json}");

            println!();
            failed += 1;
        }
    }

    println!();
    println!("📊 === Test Summary ===");
    println!("Total:   {}", test_suite.tests.len());
    if passed > 0 {
        println!(
            "✅ Passed:  {} ({:.1}%)",
            passed,
            (passed as f64 / test_suite.tests.len() as f64) * 100.0
        );
    }
    if failed > 0 {
        println!(
            "❌ Failed:  {} ({:.1}%)",
            failed,
            (failed as f64 / test_suite.tests.len() as f64) * 100.0
        );
    }
    if errors > 0 {
        println!(
            "⚠️  Errors:  {} ({:.1}%)",
            errors,
            (errors as f64 / test_suite.tests.len() as f64) * 100.0
        );
    }

    if failed > 0 || errors > 0 {
        println!("💥 Some tests failed or errored.");
        process::exit(1);
    } else {
        println!("🎉 All tests passed!");
    }
}
