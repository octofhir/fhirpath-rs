//! Test runner binary for running individual FHIRPath test files
//!
//! Usage: cargo run --bin test_runner <test_file.json>

use octofhir_fhirpath::FhirPathValue;
use octofhir_fhirpath::engine::FhirPathEngine;
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

fn compare_results(expected: &Value, actual: &FhirPathValue) -> bool {
    match (expected, actual) {
        (Value::Array(expected_arr), FhirPathValue::Collection(actual_collection)) => {
            if expected_arr.len() != actual_collection.len() {
                return false;
            }
            for (exp, act) in expected_arr.iter().zip(actual_collection.iter()) {
                if !compare_results(exp, act) {
                    return false;
                }
            }
            true
        }
        // Handle case where expected is [true] but actual is Boolean(true)
        (Value::Array(expected_arr), actual_single) => {
            if expected_arr.is_empty() {
                matches!(actual_single, FhirPathValue::Empty)
            } else if expected_arr.len() == 1 {
                compare_results(&expected_arr[0], actual_single)
            } else {
                false
            }
        }
        (Value::Bool(exp), FhirPathValue::Boolean(act)) => exp == act,
        (Value::Number(exp), FhirPathValue::Integer(act)) => exp.as_i64() == Some(*act),
        (Value::Number(exp), FhirPathValue::Decimal(act)) => {
            use rust_decimal::prelude::ToPrimitive;
            exp.as_f64()
                .is_some_and(|e| (e - act.to_f64().unwrap_or(0.0)).abs() < 1e-10)
        }
        (Value::String(exp), FhirPathValue::String(act)) => exp == act,
        (Value::Null, FhirPathValue::Empty) => true,
        _ => {
            // Try JSON conversion for complex comparisons
            match serde_json::to_value(actual) {
                Ok(actual_json) => expected == &actual_json,
                Err(_) => false,
            }
        }
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

    let mut engine = FhirPathEngine::new();
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
            println!(
                "   Expected: {}",
                serde_json::to_string_pretty(&test_case.expected).unwrap_or_default()
            );
            println!("   Actual:   {result:?}");
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
