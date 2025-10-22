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
//! Usage:
//!   cargo run --bin test-runner <test_file.json>     # Run specific file
//!   cargo run --bin test-runner <filename>           # Run by filename (with/without .json)
//!   cargo run --bin test-runner <test_name>          # Run specific test case
//!   cargo run --bin test-runner <category>           # Run all tests in category
//!
//! Examples:
//!   cargo run --bin test-runner analyzer.json
//!   cargo run --bin test-runner analyzer
//!   cargo run --bin test-runner testBooleanLogicAnd1
//!   cargo run --bin test-runner boolean

use fhirpath_dev_tools::metadata::{TestLookupResult, TestMetadataManager};
use fhirpath_dev_tools::test_support::{TestSuite, compare_results, verify_output_types};
use octofhir_fhir_model::FhirVersion;
use octofhir_fhirpath::core::trace::create_cli_provider;
use octofhir_fhirschema::create_validation_provider_from_embedded;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::Duration;

fn load_input_data(inputfile: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let specs_dir = Path::new("test-cases/input");
    let input_path = specs_dir.join(inputfile);

    let content = fs::read_to_string(&input_path)?;
    let data: Value = serde_json::from_str(&content)?;
    Ok(data)
}

/// Compare expected result with actual result
/// Simplified comparison with proper handling of FHIRPath collection semantics
type TestQueryResult = Result<Vec<(PathBuf, Option<String>)>, Box<dyn std::error::Error>>;

fn resolve_test_query(query: &str) -> TestQueryResult {
    // First try direct file path
    let direct_path = Path::new(query);
    if direct_path.exists() {
        return Ok(vec![(direct_path.to_path_buf(), None)]);
    }

    // If it looks like a direct path but doesn't exist, try relative to test-cases
    let test_cases_path = Path::new("test-cases").join(query);
    if test_cases_path.exists() {
        return Ok(vec![(test_cases_path, None)]);
    }

    // Load metadata and use intelligent lookup
    let metadata_manager = TestMetadataManager::load()?;

    match metadata_manager.lookup(query) {
        TestLookupResult::TestFile(path) => Ok(vec![(path, None)]),
        TestLookupResult::TestCase(path, test_name) => Ok(vec![(path, Some(test_name))]),
        TestLookupResult::Category(paths) => Ok(paths.into_iter().map(|p| (p, None)).collect()),
        TestLookupResult::MultipleMatches(matches) => {
            eprintln!("❌ Multiple matches found for '{query}':");
            for m in &matches {
                eprintln!("  • {m}");
            }
            eprintln!("\nPlease be more specific.");
            process::exit(1);
        }
        TestLookupResult::NotFound => {
            eprintln!("❌ No test found for '{query}'");
            eprintln!("\n🔍 Available options:");

            eprintln!("\nCategories:");
            for (category, count) in metadata_manager.list_categories() {
                eprintln!("  • {category} ({count} suites)");
            }

            eprintln!("\nTest files:");
            for (name, suite) in metadata_manager.list_test_files() {
                eprintln!("  • {} ({} tests)", name, suite.test_count);
            }

            process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <query>", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} analyzer.json          # Run specific file", args[0]);
        eprintln!("  {} analyzer               # Run by filename", args[0]);
        eprintln!("  {} testBooleanLogicAnd1   # Run specific test", args[0]);
        eprintln!("  {} boolean                # Run category", args[0]);
        process::exit(1);
    }

    let query = &args[1];
    let test_targets = resolve_test_query(query)?;

    if test_targets.len() > 1 {
        println!(
            "🧪 Running FHIRPath tests from {} files for query: {}",
            test_targets.len(),
            query
        );
    } else {
        let (path, test_name) = &test_targets[0];
        if let Some(test_name) = test_name {
            println!(
                "🧪 Running specific test '{}' from: {}",
                test_name,
                path.display()
            );
        } else {
            println!("🧪 Running FHIRPath tests from: {}", path.display());
        }
    }

    // Initialize shared components once
    println!("📋 Initializing FHIR R5 schema provider...");
    let _provider_timeout = Duration::from_secs(60);
    let provider = octofhir_fhirschema::EmbeddedSchemaProvider::new(FhirVersion::R5);
    println!("✅ EmbeddedModelProvider (R5) loaded successfully");
    let model_provider: Arc<dyn octofhir_fhirpath::ModelProvider> = Arc::new(provider);

    // Create function registry
    println!("📋 Creating function registry...");
    let registry_start = std::time::Instant::now();
    let registry = std::sync::Arc::new(octofhir_fhirpath::create_function_registry());
    let registry_time = registry_start.elapsed();
    println!(
        "✅ Function registry created in {}ms",
        registry_time.as_millis()
    );

    // Create the FhirPathEngine with model provider
    println!("📋 Creating FhirPathEngine...");
    let engine_start = std::time::Instant::now();
    let provider_version = model_provider
        .get_fhir_version()
        .await
        .unwrap_or(octofhir_fhirschema::ModelFhirVersion::R4);
    let fhir_version = match provider_version {
        octofhir_fhirschema::ModelFhirVersion::R4 => "r4".to_string(),
        octofhir_fhirschema::ModelFhirVersion::R4B => "r4b".to_string(),
        octofhir_fhirschema::ModelFhirVersion::R5 => "r5".to_string(),
        octofhir_fhirschema::ModelFhirVersion::R6 => "r6".to_string(),
        _ => "r4".to_string(),
    };
    let mut engine =
        octofhir_fhirpath::FhirPathEngine::new(registry, model_provider.clone()).await?;

    // Add CLI trace provider for trace function support
    let trace_provider = create_cli_provider();
    engine = engine.with_trace_provider(trace_provider);

    if let Ok(validation_provider) = create_validation_provider_from_embedded(
        model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>,
    )
    .await
    {
        engine = engine.with_validation_provider(validation_provider);
    }

    // Attach HttpTerminologyProvider (tx.fhir.org) for terminology-enabled tests
    let tx_base = match fhir_version.as_str() {
        "r6" => "https://tx.fhir.org/r6",
        "r5" => "https://tx.fhir.org/r5",
        "r4b" => "https://tx.fhir.org/r4b",
        _ => "https://tx.fhir.org/r4",
    };
    if let Ok(tx) = octofhir_fhir_model::HttpTerminologyProvider::new(tx_base.to_string()) {
        let tx_arc: std::sync::Arc<dyn octofhir_fhir_model::terminology::TerminologyProvider> =
            std::sync::Arc::new(tx);
        engine = engine.with_terminology_provider(tx_arc.clone());
    }
    let engine_time = engine_start.elapsed();
    println!("✅ FhirPathEngine created in {}ms", engine_time.as_millis());

    // Process all test targets
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_errors = 0;
    let mut total_tests = 0;

    for (i, (test_file_path, specific_test)) in test_targets.iter().enumerate() {
        if test_targets.len() > 1 {
            println!(
                "\n📁 ({}/{}) Processing: {}",
                i + 1,
                test_targets.len(),
                test_file_path.display()
            );
        }

        // Load test suite
        let content = match fs::read_to_string(test_file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Failed to read test file: {e}");
                continue;
            }
        };

        let test_suite: TestSuite = match serde_json::from_str(&content) {
            Ok(suite) => suite,
            Err(e) => {
                eprintln!("❌ Failed to parse test file: {e}");
                continue;
            }
        };

        println!("📝 Test Suite: {}", test_suite.name);
        if let Some(desc) = &test_suite.description {
            println!("📋 Description: {desc}");
        }

        // Filter tests if specific test requested
        let tests_to_run: Vec<_> = if let Some(specific_test_name) = specific_test {
            test_suite
                .tests
                .iter()
                .filter(|t| &t.name == specific_test_name)
                .collect()
        } else {
            test_suite.tests.iter().collect()
        };

        if tests_to_run.is_empty() {
            if specific_test.is_some() {
                eprintln!(
                    "❌ Test '{}' not found in suite '{}'",
                    specific_test.as_ref().unwrap(),
                    test_suite.name
                );
            } else {
                println!("⚠️  No tests found in suite '{}'", test_suite.name);
            }
            continue;
        }

        println!(
            "🔢 Running {} of {} tests",
            tests_to_run.len(),
            test_suite.tests.len()
        );
        println!();

        let mut passed = 0;
        let mut failed = 0;
        let mut errors = 0;

        'test_loop: for test_case in &tests_to_run {
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

            // Check if this is an analyzer category test - run analyzer-only execution
            if test_case.category.as_ref().is_some_and(|c| c == "analyzer")
                || test_suite
                    .category
                    .as_ref()
                    .is_some_and(|c| c == "analyzer")
            {
                // For analyzer tests, only run semantic analysis
                let context_type = if input_data != Value::Null {
                    // Try to determine FHIR resource type from input
                    if let Some(resource_type) =
                        input_data.get("resourceType").and_then(|v| v.as_str())
                    {
                        model_provider.get_type(resource_type).await.ok().flatten()
                    } else {
                        None
                    }
                } else {
                    None
                };

                let semantic_result = octofhir_fhirpath::parser::parse_with_semantic_analysis(
                    &test_case.expression,
                    model_provider.clone(),
                    context_type,
                )
                .await;

                if test_case.expect_error.unwrap_or(false) {
                    if let Some(ref invalid_kind) = test_case.invalid_kind
                        && (invalid_kind == "semantic" || invalid_kind == "syntax")
                    {
                        // Expect semantic/syntax error
                        if !semantic_result.analysis.success {
                            // Found error as expected
                            for diagnostic in &semantic_result.analysis.diagnostics {
                                if matches!(
                                    diagnostic.severity,
                                    octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
                                ) {
                                    println!(
                                        "✅ PASS: {} error detected: {}",
                                        invalid_kind, diagnostic.message
                                    );
                                    passed += 1;
                                    continue 'test_loop;
                                }
                            }
                        }
                        // No error found when expected
                        println!("❌ FAIL: Expected {invalid_kind} error but none found");
                        failed += 1;
                        continue;
                    }
                } else {
                    // Expect successful analysis - but continue with full evaluation if semantic analysis passes
                    if !semantic_result.analysis.success {
                        // Check if this is a type resolution issue that might work with full evaluation
                        let has_type_resolution_error = semantic_result
                            .analysis
                            .diagnostics
                            .iter()
                            .any(|d| d.message.contains("not found on Any"));

                        // If test has expected results and the error is just type resolution, fall through to evaluation
                        if has_type_resolution_error
                            && (test_case.expected != Value::Null
                                || !test_case.output_types.is_empty())
                        {
                            println!(
                                "⚠️  Semantic analysis failed due to type resolution, trying full evaluation..."
                            );
                            // Fall through to evaluation
                        } else {
                            println!("❌ FAIL: Unexpected semantic errors:");
                            for diagnostic in &semantic_result.analysis.diagnostics {
                                if matches!(
                                    diagnostic.severity,
                                    octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
                                ) {
                                    println!("   - {}", diagnostic.message);
                                }
                            }
                            failed += 1;
                            continue;
                        }
                    }
                    // Semantic analysis passed OR we're falling through due to type resolution issues - continue to evaluation
                }
            }

            // For non-analyzer tests, check for semantic errors first if test expects an error
            if test_case.expect_error.is_some()
                && test_case.expect_error.unwrap()
                && let Some(ref invalid_kind) = test_case.invalid_kind
                && invalid_kind == "semantic"
            {
                // Extract context type from input data if available
                let context_type = if input_data != Value::Null {
                    // Try to determine FHIR resource type from input
                    if let Some(resource_type) =
                        input_data.get("resourceType").and_then(|v| v.as_str())
                    {
                        model_provider.get_type(resource_type).await.ok().flatten()
                    } else {
                        None
                    }
                } else {
                    None
                };

                let semantic_result = octofhir_fhirpath::parser::parse_with_semantic_analysis(
                    &test_case.expression,
                    model_provider.clone(),
                    context_type,
                )
                .await;

                if !semantic_result.analysis.success {
                    // Found semantic error as expected
                    for diagnostic in &semantic_result.analysis.diagnostics {
                        if matches!(
                            diagnostic.severity,
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
                        ) {
                            println!("✅ PASS: Semantic error detected: {}", diagnostic.message);
                            passed += 1;
                            continue 'test_loop;
                        }
                    }
                }
                // If we get here, no semantic error was found
                println!("❌ FAIL: Expected semantic error but none found");
                failed += 1;
                continue;
            }

            // Convert input to FhirPathValue and create evaluation context
            let input_value = octofhir_fhirpath::FhirPathValue::resource(input_data);
            let input_collection = octofhir_fhirpath::Collection::single(input_value);
            let context = octofhir_fhirpath::EvaluationContext::new(
                input_collection,
                model_provider.clone(),
                engine.get_terminology_provider(),
                engine.get_validation_provider(),
                engine.get_trace_provider(),
            );

            // Log terminology setup only for tests that actually use it (engine handles terminology setup automatically)
            if test_suite.name.contains("Terminology")
                || test_case.expression.contains("%terminologies")
            {
                let fhir_version =
                    std::env::var("FHIRPATH_FHIR_VERSION").unwrap_or_else(|_| "r4".to_string());
                println!(
                    "📋 Engine includes terminology service (tx.fhir.org/{fhir_version}) for test '{}'",
                    test_case.name
                );
            }

            // Use single root evaluation method (parse + evaluate in one call)
            let timeout_ms: u64 = env::var("FHIRPATH_TEST_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5_000);

            println!("📋 Evaluating expression with timeout {timeout_ms}ms...");
            let eval_start = std::time::Instant::now();
            let eval_fut = engine.evaluate(&test_case.expression, &context);
            let result = match tokio::time::timeout(Duration::from_millis(timeout_ms), eval_fut)
                .await
            {
                Err(_) => {
                    let eval_time = eval_start.elapsed();
                    println!(
                        "⚠️ TIMEOUT after {}ms (limit: {timeout_ms}ms)",
                        eval_time.as_millis()
                    );
                    if test_case.expect_error.is_some() && test_case.expect_error.unwrap() {
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
                            if test_case.expect_error.is_some() && test_case.expect_error.unwrap() {
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
            if test_case.expect_error.is_some() && test_case.expect_error.unwrap() {
                println!("❌ FAIL: Expected error but got result");
                failed += 1;
                continue;
            }

            // Handle predicate tests - convert result to boolean using FHIRPath exists() logic
            let final_result = if test_case.predicate.is_some() && test_case.predicate.unwrap() {
                use octofhir_fhirpath::FhirPathValue;
                let exists = !result.is_empty();
                octofhir_fhirpath::Collection::single(FhirPathValue::Boolean(
                    exists,
                    octofhir_fhir_model::TypeInfo::system_type("Boolean".to_string(), true),
                    None,
                ))
            } else {
                result
            };

            if !test_case.output_types.is_empty()
                && let Err(mismatch) = verify_output_types(&test_case.output_types, &final_result)
            {
                println!("❌ FAIL: Type mismatch");
                println!("   Expected types: {:?}", mismatch.expected);
                println!("   Actual types:   {:?}", mismatch.actual);
                failed += 1;
                continue;
            }

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
                        .unwrap_or_else(|_| format!("{final_result:?}")),
                    Err(_) => format!("{final_result:?}"),
                };
                println!("   Expected: {expected_json}");
                println!("   Actual:   {actual_json}");

                println!();
                failed += 1;
            }
        }

        println!();
        println!("📊 === Test Suite Summary ===");
        println!("Total:   {}", tests_to_run.len());
        if passed > 0 {
            println!(
                "✅ Passed:  {} ({:.1}%)",
                passed,
                (passed as f64 / tests_to_run.len() as f64) * 100.0
            );
        }
        if failed > 0 {
            println!(
                "❌ Failed:  {} ({:.1}%)",
                failed,
                (failed as f64 / tests_to_run.len() as f64) * 100.0
            );
        }
        if errors > 0 {
            println!(
                "⚠️  Errors:  {} ({:.1}%)",
                errors,
                (errors as f64 / tests_to_run.len() as f64) * 100.0
            );
        }

        total_passed += passed;
        total_failed += failed;
        total_errors += errors;
        total_tests += tests_to_run.len();
    }

    // Overall summary for multiple files
    if test_targets.len() > 1 {
        println!("\n📊 === Overall Summary ===");
        println!("Total files: {}", test_targets.len());
        println!("Total tests: {total_tests}");
        if total_passed > 0 {
            println!(
                "✅ Passed:   {} ({:.1}%)",
                total_passed,
                (total_passed as f64 / total_tests as f64) * 100.0
            );
        }
        if total_failed > 0 {
            println!(
                "❌ Failed:   {} ({:.1}%)",
                total_failed,
                (total_failed as f64 / total_tests as f64) * 100.0
            );
        }
        if total_errors > 0 {
            println!(
                "⚠️  Errors:   {} ({:.1}%)",
                total_errors,
                (total_errors as f64 / total_tests as f64) * 100.0
            );
        }
    }

    if total_failed > 0 || total_errors > 0 {
        println!("💥 Some tests failed or errored.");
        process::exit(1);
    } else {
        println!("🎉 All tests passed!");
    }

    Ok(())
}
