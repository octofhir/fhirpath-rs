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
//! Usage: cargo run --bin test-runner <test_file.json>

use octofhir_fhirpath::FhirPathValue;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::time::Duration;

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
    #[serde(default, alias = "expectError", alias = "expecterror")]
    pub expecterror: Option<bool>,
    #[serde(default)]
    pub predicate: Option<bool>,
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
    let specs_dir = Path::new("test-cases/input");
    let input_path = specs_dir.join(inputfile);

    let content = fs::read_to_string(&input_path)?;
    let data: Value = serde_json::from_str(&content)?;
    Ok(data)
}

/// Compare expected result with actual result
/// Simplified comparison with proper handling of FHIRPath collection semantics
fn compare_results(expected: &Value, actual: &octofhir_fhirpath::Collection) -> bool {
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
        (expected_single, actual_json) if actual_json.is_array() => {
            if let Some(actual_arr) = actual_json.as_array() {
                if actual_arr.len() == 1 {
                    expected_single == &actual_arr[0]
                } else {
                    false
                }
            } else {
                false
            }
        }
        // Test expects array, actual is single value (shouldn't happen with new spec compliance but handle it)
        (expected, actual_single) if expected.is_array() => {
            if let Some(expected_arr) = expected.as_array() {
                if expected_arr.len() == 1 {
                    &expected_arr[0] == actual_single
                } else {
                    expected == actual_single
                }
            } else {
                false
            }
        }
        // Both empty
        (expected, actual_json) if expected.is_array() && actual_json.is_null() => {
            if let Some(expected_arr) = expected.as_array() {
                expected_arr.is_empty()
            } else {
                false
            }
        }
        (expected, actual_json) if expected.is_null() && actual_json.is_array() => {
            if let Some(actual_arr) = actual_json.as_array() {
                actual_arr.is_empty()
            } else {
                false
            }
        }
        // Test expects array with single item, actual is single primitive
        (expected, actual_single) if expected.is_array() => {
            if let Some(expected_arr) = expected.as_array() {
                if expected_arr.len() == 1 {
                    &expected_arr[0] == actual_single
                } else {
                    false
                }
            } else {
                false
            }
        }
        // Default: no match
        _ => false,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create FHIR schema provider (R4) to match CLI behavior
    println!("📋 Initializing FHIR R4 schema provider...");
    let provider_timeout = Duration::from_secs(60);
    let model_provider: Arc<dyn octofhir_fhirpath::ModelProvider> = match tokio::time::timeout(
        provider_timeout,
        octofhir_fhirschema::provider::EmbeddedModelProvider::r4(),
    )
    .await
    {
        Ok(Ok(provider)) => {
            println!("✅ EmbeddedModelProvider (R4) loaded successfully");
            Arc::new(provider)
        }
        Ok(Err(e)) => {
            eprintln!("❌ Failed to initialize EmbeddedModelProvider (R4): {e}");
            eprintln!("💡 Ensure FHIR schema packages are available");
            process::exit(1);
        }
        Err(_) => {
            eprintln!(
                "❌ EmbeddedModelProvider (R4) initialization timed out ({}s)",
                provider_timeout.as_secs()
            );
            eprintln!("💡 Check network connectivity");
            process::exit(1);
        }
    };

    // Create function registry
    println!("📋 Creating function registry...");
    let registry_start = std::time::Instant::now();
    let registry = std::sync::Arc::new(octofhir_fhirpath::create_standard_registry().await);
    let registry_time = registry_start.elapsed();
    println!(
        "✅ Function registry created in {}ms",
        registry_time.as_millis()
    );

    // Create the FhirPathEngine with model provider
    println!("📋 Creating FhirPathEngine...");
    let engine_start = std::time::Instant::now();
    let fhir_version = std::env::var("FHIRPATH_FHIR_VERSION").unwrap_or_else(|_| "r4".to_string());
    let mut engine = octofhir_fhirpath::FhirPathEngine::new_with_fhir_version(
        registry,
        model_provider.clone(),
        &fhir_version,
    )
    .await?;
    let engine_time = engine_start.elapsed();
    println!("✅ FhirPathEngine created in {}ms", engine_time.as_millis());
    let mut passed = 0;
    let mut failed = 0;
    let mut errors = 0;

    for test_case in &test_suite.tests {
        print!("Running {} ... ", test_case.name);

        // (Debug block removed; keeping runner output lean for CI)

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

        // Convert input to FhirPathValue and create evaluation context
        let input_value = octofhir_fhirpath::FhirPathValue::resource(input_data);
        let mut context = octofhir_fhirpath::EvaluationContext::from_value(input_value);

        // Log terminology setup only for tests that actually use it (engine handles terminology setup automatically)
        if test_suite.name.contains("Terminology")
            || test_case.expression.contains("%terminologies")
        {
            let fhir_version =
                std::env::var("FHIRPATH_FHIR_VERSION").unwrap_or_else(|_| "r4".to_string());
            println!(
                "📋 Engine includes terminology service (tx.fhir.org/{}) for test '{}'",
                fhir_version, test_case.name
            );
        }

        // Use single root evaluation method (parse + evaluate in one call)
        let timeout_ms: u64 = env::var("FHIRPATH_TEST_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5_000);

        println!("📋 Evaluating expression with timeout {}ms...", timeout_ms);
        let eval_start = std::time::Instant::now();
        let eval_fut = engine.evaluate(&test_case.expression, &context);
        let result = match tokio::time::timeout(Duration::from_millis(timeout_ms), eval_fut).await {
            Err(_) => {
                let eval_time = eval_start.elapsed();
                println!(
                    "⚠️ TIMEOUT after {}ms (limit: {}ms)",
                    eval_time.as_millis(),
                    timeout_ms
                );
                if test_case.expecterror.is_some() && test_case.expecterror.unwrap() {
                    println!("✅ PASS");
                    passed += 1;
                    continue;
                }
                errors += 1;
                continue;
            }
            Ok(inner) => {
                let eval_time = eval_start.elapsed();
                println!("✅ Expression evaluated in {}ms", eval_time.as_millis());
                match inner {
                    Ok(eval_result) => eval_result.value, // Extract FhirPathValue from EvaluationResult
                    Err(e) => {
                        if test_case.expecterror.is_some() && test_case.expecterror.unwrap() {
                            println!("✅ PASS");
                            passed += 1;
                            continue;
                        }
                        println!("⚠️ ERROR: {e}");
                        errors += 1;
                        continue;
                    }
                }
            }
        };

        // Check if test expects an error but we got a result
        if test_case.expecterror.is_some() && test_case.expecterror.unwrap() {
            println!("❌ FAIL: Expected error but got result");
            failed += 1;
            continue;
        }

        // Handle predicate tests - convert result to boolean using FHIRPath exists() logic
        let final_result = if test_case.predicate.is_some() && test_case.predicate.unwrap() {
            use octofhir_fhirpath::FhirPathValue;
            let exists = !result.is_empty();
            octofhir_fhirpath::Collection::single(FhirPathValue::Boolean(exists))
        } else {
            result
        };

        // Compare results
        if compare_results(&test_case.expected, &final_result) {
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
            let actual_json = match serde_json::to_value(&final_result) {
                Ok(json) => serde_json::to_string_pretty(&json)
                    .unwrap_or_else(|_| format!("{:?}", final_result)),
                Err(_) => format!("{:?}", final_result),
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

    Ok(())
}
