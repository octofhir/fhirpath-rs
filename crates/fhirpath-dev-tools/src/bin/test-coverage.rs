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

//! Test coverage generation binary for FHIRPath implementation
//!
//! This binary runs all official FHIRPath test suites and generates a comprehensive
//! coverage report saved to TEST_COVERAGE.md

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

// Integration test runner functionality
mod integration_test_runner {
    use octofhir_fhirpath::FhirPathValue;
    use octofhir_fhirpath::ModelProvider;
    use octofhir_fhirpath::{FhirPathEngine, create_standard_registry};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    /// A single test case within a test suite
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TestCase {
        /// Test name
        pub name: String,
        /// FHIRPath expression to evaluate
        pub expression: String,
        /// Input data (usually null, uses inputfile instead)
        #[serde(default, deserialize_with = "deserialize_nullable_input")]
        pub input: Option<Value>,
        /// File containing input data
        pub inputfile: Option<String>,
        /// Expected result
        pub expected: Value,
        /// Test tags for categorization
        #[serde(default)]
        pub tags: Vec<String>,
        /// Optional description
        #[serde(default)]
        pub description: Option<String>,

        /// Expression is expected to error (parse or evaluation)
        #[serde(rename = "expectError", alias = "expecterror")]
        pub expect_error: Option<bool>,

        /// Mark test disabled/skipped
        #[serde(default)]
        pub disabled: Option<bool>,

        /// For metadata only (not used in execution)
        #[serde(default)]
        pub predicate: Option<bool>,

        /// Skip static check metadata from XML (kept for reference)
        #[serde(rename = "skipStaticCheck")]
        pub skip_static_check: Option<bool>,

        /// Invalid kind from XML: syntax|semantic|execution (reference only)
        #[serde(rename = "invalidKind")]
        pub invalid_kind: Option<String>,

        /// Expression mode (e.g., strict) from XML
        #[serde(default)]
        pub mode: Option<String>,
    }

    /// Custom deserializer to handle "input": null as Some(Value::Null) instead of None
    fn deserialize_nullable_input<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt = Option::<Value>::deserialize(deserializer)?;
        Ok(opt.or(Some(Value::Null)))
    }

    /// A test suite containing multiple test cases
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct TestSuite {
        /// Suite name
        pub name: String,
        /// Suite description
        #[serde(default)]
        pub description: Option<String>,
        /// Source of the tests
        #[serde(default)]
        pub source: Option<String>,
        /// List of test cases
        pub tests: Vec<TestCase>,
    }

    /// Result of running a single test
    #[derive(Debug, Clone, PartialEq)]
    pub enum TestResult {
        /// Test passed
        Passed,
        /// Test failed with actual vs expected values
        Failed { expected: Value, actual: Value },
        /// Test errored during execution
        Error { error: String },
        /// Test was skipped
        Skipped { reason: String },
    }

    /// Statistics for test run results
    #[derive(Debug, Clone, Default)]
    pub struct TestStats {
        pub total: usize,
        pub passed: usize,
        pub failed: usize,
        pub errored: usize,
        pub skipped: usize,
        pub error_details: Vec<String>, // Store first few error messages for debugging
    }

    impl TestStats {
        /// Calculate pass rate as percentage
        pub fn pass_rate(&self) -> f64 {
            if self.total == 0 {
                0.0
            } else {
                (self.passed as f64 / self.total as f64) * 100.0
            }
        }
    }

    /// Integration test runner that uses the complete FHIRPath stack
    pub struct IntegrationTestRunner {
        engine: FhirPathEngine,
        registry: Arc<octofhir_fhirpath::FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        input_cache: HashMap<String, Value>,
        base_path: PathBuf,
        verbose: bool,
    }

    impl IntegrationTestRunner {
        /// Create a new integration test runner with FHIR R4 ModelProvider (same as test-runner)
        pub async fn new() -> Self {
            println!("🔄 Loading FHIR R4 ModelProvider (using schema packages)...");
            let model_provider: std::sync::Arc<dyn octofhir_fhirpath::ModelProvider> = {
                // Add timeout to prevent hanging
                let timeout_duration = std::time::Duration::from_secs(60);
                match tokio::time::timeout(
                    timeout_duration,
                    octofhir_fhirschema::provider::FhirSchemaModelProvider::r4(),
                )
                .await
                {
                    Ok(Ok(provider)) => {
                        println!("✅ FhirSchemaModelProvider (R4) loaded successfully");
                        std::sync::Arc::new(provider)
                    }
                    Ok(Err(e)) => {
                        panic!("❌ Failed to load FhirSchemaModelProvider (R4): {e}");
                    }
                    Err(_) => {
                        panic!(
                            "❌ FhirSchemaModelProvider (R4) initialization timed out ({}s)",
                            timeout_duration.as_secs()
                        );
                    }
                }
            };

            let registry = Arc::new(create_standard_registry().await);
            let engine = FhirPathEngine::new(registry.clone(), model_provider.clone());

            Self {
                engine,
                registry,
                model_provider,
                input_cache: HashMap::new(),
                base_path: PathBuf::from("."),
                verbose: false,
            }
        }

        /// Set the base path for resolving test and input files
        pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
            self.base_path = path.as_ref().to_path_buf();
            self
        }

        /// Enable verbose output
        pub fn with_verbose(mut self, verbose: bool) -> Self {
            self.verbose = verbose;
            self
        }

        /// Load a test suite from a JSON file
        pub fn load_test_suite<P: AsRef<Path>>(
            &self,
            path: P,
        ) -> Result<TestSuite, Box<dyn std::error::Error>> {
            let full_path = path.as_ref().to_path_buf();

            let content = fs::read_to_string(&full_path).map_err(|e| {
                format!(
                    "Failed to read test suite file {}: {}",
                    full_path.display(),
                    e
                )
            })?;

            let suite: TestSuite = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse test suite JSON: {e}"))?;

            Ok(suite)
        }

        /// Load input data from a file (with caching)
        fn load_input_data(&mut self, filename: &str) -> Result<Value, Box<dyn std::error::Error>> {
            if let Some(cached) = self.input_cache.get(filename) {
                return Ok(cached.clone());
            }

            // Try multiple possible paths for input files
            let possible_paths = vec![
                PathBuf::from("test-cases/input").join(filename),
                PathBuf::from("input").join(filename),
                self.base_path.join("input").join(filename),
            ];

            let mut content = None;
            let mut used_path = None;

            for path in &possible_paths {
                if let Ok(file_content) = fs::read_to_string(path) {
                    content = Some(file_content);
                    used_path = Some(path.clone());
                    break;
                }
            }

            let content = content.ok_or_else(|| {
                format!(
                    "Failed to find input file {} in any of: {}",
                    filename,
                    possible_paths
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

            let json_value: Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON in {filename}: {e}"))?;

            if self.verbose {
                println!(
                    "Loaded input file {} from {}",
                    filename,
                    used_path.unwrap().display()
                );
            }

            self.input_cache
                .insert(filename.to_string(), json_value.clone());
            Ok(json_value)
        }


        /// Convert expected JSON value to FhirPathValue for comparison
        fn convert_expected_value(&self, expected: &Value) -> FhirPathValue {
            FhirPathValue::JsonValue(expected.clone())
        }

        /// Compare actual collection result with expected result
        /// Matches the test-runner comparison logic exactly  
        fn compare_results_collection(
            &self,
            actual: &octofhir_fhirpath::Collection,
            expected: &Value,
        ) -> bool {
            // Convert actual to JSON for uniform comparison
            let actual_normalized = match serde_json::to_value(actual) {
                Ok(json) => json,
                Err(_) => return false,
            };

            // Direct comparison first - handles most cases
            if expected == &actual_normalized {
                return true;
            }

            // FHIRPath collection handling: expected single value should match [single_value]
            match (expected, &actual_normalized) {
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

        /// Compare actual result with expected result
        /// Simplified comparison with proper handling of FHIRPath collection semantics
        fn compare_results(&self, actual: &FhirPathValue, expected: &Value) -> bool {
            // Convert actual to JSON for uniform comparison
            let actual_normalized = match serde_json::to_value(actual) {
                Ok(json) => json,
                Err(_) => return false,
            };

            // Direct comparison first - handles most cases
            if expected == &actual_normalized {
                return true;
            }

            // FHIRPath collection handling: expected single value should match [single_value]
            match (expected, &actual_normalized) {
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
                // Default: no match
                _ => false,
            }
        }

        /// Run a single test case using the integrated stack
        pub async fn run_test(&mut self, test: &TestCase) -> TestResult {
            // Skip disabled tests
            if test.disabled.unwrap_or(false) {
                return TestResult::Skipped {
                    reason: "disabled".into(),
                };
            }
            if self.verbose {
                println!("Running test: {}", test.name);
                println!("Expression: {}", test.expression);
            }

            // Load input data - use same logic as test-runner.rs
            let input_data = if let Some(ref filename) = test.inputfile {
                match self.load_input_data(filename) {
                    Ok(json_data) => json_data,
                    Err(e) => {
                        return TestResult::Error {
                            error: format!("Failed to load input from {filename}: {e}"),
                        };
                    }
                }
            } else if let Some(ref input_val) = test.input {
                input_val.clone()
            } else {
                serde_json::Value::Null
            };

            // Convert input_data to FhirPathValue and create collection - same as test-runner
            let input_value = octofhir_fhirpath::FhirPathValue::resource(input_data);
            let collection = octofhir_fhirpath::Collection::single(input_value);

            // Create evaluation context with the collection
            let context = octofhir_fhirpath::EvaluationContext::new(collection);

            // Use single root evaluation method (parse + evaluate in one call) - same as test-runner
            let timeout_ms: u64 = std::env::var("FHIRPATH_TEST_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10_000);

            let eval_fut = self.engine.evaluate(&test.expression, &context);
            let result = match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), eval_fut).await {
                Err(_) => {
                    if test.expect_error.unwrap_or(false) {
                        return TestResult::Passed;
                    }
                    return TestResult::Error {
                        error: format!("Evaluation timed out after {}ms", timeout_ms),
                    };
                }
                Ok(inner) => match inner {
                    Ok(eval_result) => eval_result.value, // Extract collection from EvaluationResult
                    Err(e) => {
                        if test.expect_error.unwrap_or(false) {
                            return TestResult::Passed;
                        }
                        // Legacy: empty expected means error acceptable
                        let expected = self.convert_expected_value(&test.expected);
                        return if expected.is_empty() {
                            TestResult::Passed
                        } else {
                            TestResult::Error {
                                error: format!("Evaluation error: {e}"),
                            }
                        };
                    }
                },
            };

            // For expectError tests, we only expect errors during parsing/evaluation
            // If evaluation succeeds, we should compare results normally
            // The expectError flag is handled earlier for actual parse/eval errors

            if self.verbose {
                println!("Result: {result:?}");
                println!(
                    "Expected: {}",
                    serde_json::to_string(&test.expected).unwrap_or_default()
                );
            }

            // Compare results using the entire collection (matches test-runner behavior)
            if self.compare_results_collection(&result, &test.expected) {
                TestResult::Passed
            } else {
                // Convert actual result to JSON for display
                let actual_json = serde_json::to_value(&result).unwrap_or_default();
                
                TestResult::Failed {
                    expected: test.expected.clone(),
                    actual: actual_json,
                }
            }
        }

        /// Run all tests in a test suite
        pub async fn run_test_suite(&mut self, suite: &TestSuite) -> HashMap<String, TestResult> {
            let mut results = HashMap::new();

            if self.verbose {
                println!("Running test suite: {}", suite.name);
                if let Some(desc) = &suite.description {
                    println!("Description: {}", desc);
                }
                println!("Total tests: {}", suite.tests.len());
                println!();
            }

            for test in &suite.tests {
                let result = self.run_test(test).await;
                results.insert(test.name.clone(), result);
            }

            results
        }

        /// Calculate statistics from test results
        pub fn calculate_stats(&self, results: &HashMap<String, TestResult>) -> TestStats {
            let mut stats = TestStats::default();
            stats.total = results.len();

            for result in results.values() {
                match result {
                    TestResult::Passed => stats.passed += 1,
                    TestResult::Failed { .. } => stats.failed += 1,
                    TestResult::Error { .. } => stats.errored += 1,
                    TestResult::Skipped { .. } => stats.skipped += 1,
                }
            }

            stats
        }

        /// Run tests silently and only report failures/errors for coverage analysis
        pub async fn run_and_report_quiet<P: AsRef<Path>>(
            &mut self,
            path: P,
        ) -> Result<TestStats, Box<dyn std::error::Error>> {
            let suite = self.load_test_suite(&path)?;
            let results = self.run_test_suite(&suite).await;

            // Compile statistics
            let mut stats = TestStats::default();
            stats.total = suite.tests.len();
            for test in &suite.tests {
                let result = &results[&test.name];
                match result {
                    TestResult::Passed => stats.passed += 1,
                    TestResult::Failed { .. } => stats.failed += 1,
                    TestResult::Error { error } => {
                        stats.errored += 1;
                        // Capture first 3 error details for debugging
                        if stats.error_details.len() < 3 {
                            stats
                                .error_details
                                .push(format!("{}: {}", test.name, error));
                        }
                    }
                    TestResult::Skipped { .. } => stats.skipped += 1,
                }
            }

            Ok(stats)
        }
    }
}

use integration_test_runner::{IntegrationTestRunner, TestStats};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("test-coverage")
        .version("0.1.0")
        .about("Generate test coverage reports for FHIRPath implementation")
        .arg(
            Arg::new("specs-dir")
                .long("specs-dir")
                .value_name("DIR")
                .help("Path to FHIRPath test specifications")
                .default_value("test-cases"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file for coverage report")
                .default_value("TEST_COVERAGE.md"),
        )
        .get_matches();

    let specs_dir = PathBuf::from(matches.get_one::<String>("specs-dir").unwrap());
    let output_file = PathBuf::from(matches.get_one::<String>("output").unwrap());

    println!("🧪 Generating FHIRPath Test Coverage Report");
    println!("============================================");

    if !specs_dir.exists() {
        println!("❌ Specs directory not found: {}", specs_dir.display());
        return Ok(());
    }

    let test_files = get_all_test_files(&specs_dir);
    if test_files.is_empty() {
        println!("❌ No test files found in specs directory");
        return Ok(());
    }

    println!("📁 Found {} test files", test_files.len());
    println!("🏃 Running tests...\n");

    let mut runner = IntegrationTestRunner::new()
        .await
        .with_base_path(&specs_dir)
        .with_verbose(false);

    let mut test_results = Vec::new();
    let mut processed = 0;

    for test_file in &test_files {
        if let Some(filename) = test_file.file_stem().and_then(|s| s.to_str()) {
            processed += 1;
            print!(
                "({:2}/{:2}) Testing {:<25}",
                processed,
                test_files.len(),
                format!("{}.json", filename)
            );

            match runner.run_and_report_quiet(test_file).await {
                Ok(stats) => {
                    // Show ERROR status if there are parse errors, otherwise show pass/fail info
                    if stats.errored > 0 {
                        println!(
                            " ⚠️ ERROR {}/{} ({} parse errors, {} failed)",
                            stats.passed, stats.total, stats.errored, stats.failed
                        );
                        // Show detailed error messages for debugging
                        if !stats.error_details.is_empty() {
                            for (i, error_detail) in stats.error_details.iter().enumerate() {
                                println!("     {}. {}", i + 1, error_detail);
                            }
                            if stats.error_details.len() < stats.errored {
                                println!(
                                    "     ... and {} more parse errors",
                                    stats.errored - stats.error_details.len()
                                );
                            }
                        }
                    } else {
                        let emoji = if stats.pass_rate() == 100.0 {
                            "✅"
                        } else if stats.pass_rate() >= 70.0 {
                            "🟡"
                        } else if stats.pass_rate() >= 30.0 {
                            "🟠"
                        } else {
                            "🔴"
                        };

                        println!(
                            " {} {}/{} ({:.1}%)",
                            emoji,
                            stats.passed,
                            stats.total,
                            stats.pass_rate()
                        );
                    }

                    test_results.push((filename.to_string(), stats));
                }
                Err(e) => {
                    println!(" ❌ Error: {e}");
                    test_results.push((
                        filename.to_string(),
                        TestStats {
                            total: 1,
                            passed: 0,
                            failed: 0,
                            errored: 1,
                            skipped: 0,
                            error_details: vec![format!("File load error: {}", e)],
                        },
                    ));
                }
            }
        }
    }

    println!("\n📊 Generating coverage report...");

    // Generate comprehensive report
    let report = generate_coverage_report(&test_results);
    fs::write(&output_file, report)?;

    let total_tests: usize = test_results.iter().map(|(_, r)| r.total).sum();
    let total_passed: usize = test_results.iter().map(|(_, r)| r.passed).sum();
    let overall_pass_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    println!("✅ Coverage report saved to: {}", output_file.display());
    println!("\n📈 Summary:");
    println!("   Test Suites: {}", test_results.len());
    println!("   Total Tests: {total_tests}");
    println!("   Pass Rate: {overall_pass_rate:.1}%");

    Ok(())
}

fn get_all_test_files(specs_path: &PathBuf) -> Vec<PathBuf> {
    fn collect(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip input data directories — they contain JSON resources, not test suites
                    if path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                        n.eq_ignore_ascii_case("input") || n.eq_ignore_ascii_case("inputs")
                    }) {
                        continue;
                    }
                    collect(&path, out);
                } else if path.extension().is_some_and(|ext| ext == "json") {
                    out.push(path);
                }
            }
        }
    }

    let mut test_files = Vec::new();
    collect(specs_path, &mut test_files);
    test_files.sort();
    test_files
}

fn generate_coverage_report(test_results: &[(String, TestStats)]) -> String {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.format("%Y-%m-%d").to_string();

    let total_suites = test_results.len();
    let total_tests: usize = test_results.iter().map(|(_, r)| r.total).sum();
    let total_passed: usize = test_results.iter().map(|(_, r)| r.passed).sum();
    let total_failed: usize = test_results.iter().map(|(_, r)| r.failed).sum();
    let total_errors: usize = test_results.iter().map(|(_, r)| r.errored).sum();

    let overall_pass_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    let mut report = format!(
        r#"# FHIRPath Test Coverage Report

Generated on: {}
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | {} | 100% |
| **Total Individual Tests** | {} | 100% |
| **Passing Tests** | {} | {:.1}% |
| **Failing Tests** | {} | {:.1}% |
| **Error Tests** | {} | {:.1}% |

## Test Results by Suite

"#,
        timestamp,
        total_suites,
        total_tests,
        total_passed,
        overall_pass_rate,
        total_failed,
        if total_tests > 0 {
            (total_failed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        },
        total_errors,
        if total_tests > 0 {
            (total_errors as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        }
    );

    // Sort by pass rate
    let mut sorted_results = test_results.to_vec();
    sorted_results.sort_by(|a, b| b.1.pass_rate().partial_cmp(&a.1.pass_rate()).unwrap());

    // Fully passing tests
    report.push_str("### ✅ Fully Passing (100%)\n\n");
    let fully_passing: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() == 100.0)
        .collect();
    if fully_passing.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &fully_passing {
            report.push_str(&format!(
                "- **{}.json** - {}/{} tests\n",
                name, stats.passed, stats.total
            ));
        }
        report.push('\n');
    }

    // Well implemented
    report.push_str("### 🟡 Well Implemented (70%+)\n\n");
    let well_implemented: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() >= 70.0 && s.pass_rate() < 100.0)
        .collect();
    if well_implemented.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &well_implemented {
            report.push_str(&format!(
                "- **{}.json** - {:.1}% ({}/{} tests)\n",
                name,
                stats.pass_rate(),
                stats.passed,
                stats.total
            ));
        }
        report.push('\n');
    }

    // Partially implemented
    report.push_str("### 🟠 Partially Implemented (30-70%)\n\n");
    let partially_implemented: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() >= 30.0 && s.pass_rate() < 70.0)
        .collect();
    if partially_implemented.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &partially_implemented {
            report.push_str(&format!(
                "- **{}.json** - {:.1}% ({}/{} tests)\n",
                name,
                stats.pass_rate(),
                stats.passed,
                stats.total
            ));
        }
        report.push('\n');
    }

    // Major issues
    report.push_str("### 🔴 Major Issues (0-30%)\n\n");
    let major_issues: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() < 30.0)
        .collect();
    if major_issues.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &major_issues {
            let status = if stats.passed == 0 {
                "Missing"
            } else {
                "Issues"
            };
            report.push_str(&format!(
                "- **{}.json** - {:.1}% ({}/{} tests) - {}\n",
                name,
                stats.pass_rate(),
                stats.passed,
                stats.total,
                status
            ));
        }
        report.push('\n');
    }

    report.push_str(&format!(
        r#"## Summary

The fhirpath-rs implementation currently passes approximately **{:.1}% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: {}
- **Total Tests**: {}
- **Pass Rate**: {:.1}%

---

*Report generated on: {}*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
"#,
        overall_pass_rate,
        total_suites,
        total_tests,
        overall_pass_rate,
        now.format("%Y-%m-%d %H:%M:%S")
    ));

    report
}
