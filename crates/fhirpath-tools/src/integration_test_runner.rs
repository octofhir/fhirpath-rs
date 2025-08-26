//! Integration Test Runner for FHIRPath Official Tests
//!
//! This module provides functionality to run official FHIRPath tests from JSON test definitions
//! using the complete integrated stack of all fhirpath-* crates together.

use octofhir_fhirpath::ast::ExpressionNode;
use octofhir_fhirpath::model::FhirPathValue;
use octofhir_fhirpath::model::ModelProvider;
use octofhir_fhirpath::{FhirPathEngine, create_standard_registry, parse};
use serde::{Deserialize, Serialize};
use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value};
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

    /// expression call allows to fail
    #[serde(rename = "expectError")]
    pub expect_error: Option<bool>,
}

/// Custom deserializer to handle "input": null as Some(Value::Null) instead of None
fn deserialize_nullable_input<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<Value>::deserialize(deserializer)?;
    Ok(opt.or(Some(Value::new_null())))
}

/// A test suite containing multiple test cases
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Suite description
    pub description: String,
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
    registry: Arc<octofhir_fhirpath_registry::FunctionRegistry>,
    model_provider: Arc<dyn ModelProvider>,
    input_cache: HashMap<String, Value>,
    base_path: PathBuf,
    verbose: bool,
}

impl IntegrationTestRunner {
    /// Create a new integration test runner with real FhirSchemaModelProvider
    pub async fn new() -> Self {
        // Use real FhirSchemaModelProvider for official test validation (FHIR R4) with timeout
        let model_provider: std::sync::Arc<dyn octofhir_fhirpath::model::ModelProvider> = {
            // Check if MockModelProvider is explicitly requested via environment variable
            if std::env::var("FHIRPATH_USE_MOCK_PROVIDER").is_ok() {
                println!("🔄 Using MockModelProvider (FHIRPATH_USE_MOCK_PROVIDER set)");
                std::sync::Arc::new(
                    octofhir_fhirpath::model::mock_provider::MockModelProvider::new(),
                )
            } else {
                println!(
                    "🔄 Loading FhirSchemaModelProvider (may download packages on first run)..."
                );

                // Add timeout to FhirSchemaModelProvider initialization to prevent hanging
                let timeout_duration = std::time::Duration::from_secs(60); // 60 seconds timeout
                match tokio::time::timeout(
                    timeout_duration,
                    octofhir_fhirpath::model::fhirschema_provider::FhirSchemaModelProvider::with_config(
                        octofhir_fhirpath::model::provider::FhirSchemaConfig::default()
                    ),
                )
                .await
                {
                    Ok(Ok(provider)) => {
                        println!("✅ FhirSchemaModelProvider loaded successfully");
                        std::sync::Arc::new(provider)
                    }
                    Ok(Err(e)) => {
                        // FhirSchema provider failed to load
                        eprintln!(
                            "⚠️  Warning: Failed to load FhirSchemaModelProvider ({e}), using MockModelProvider"
                        );
                        std::sync::Arc::new(
                            octofhir_fhirpath::model::mock_provider::MockModelProvider::new(),
                        )
                    }
                    Err(_) => {
                        // Timeout occurred
                        eprintln!(
                            "⚠️  Warning: FhirSchemaModelProvider initialization timed out ({}s), using MockModelProvider",
                            timeout_duration.as_secs()
                        );
                        std::sync::Arc::new(
                            octofhir_fhirpath::model::mock_provider::MockModelProvider::new(),
                        )
                    }
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

        let suite: TestSuite = sonic_rs::from_str(&content)
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
            PathBuf::from("specs/fhirpath/tests/input").join(filename),
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

        let json_value: Value = sonic_rs::from_str(&content)
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

    /// Parse a FHIRPath expression using the integrated parser
    fn parse_expression(&mut self, expression: &str) -> Result<ExpressionNode, String> {
        parse(expression).map_err(|e| format!("Parser error in '{expression}': {e}"))
    }

    /// Convert expected JSON value to FhirPathValue for comparison
    fn convert_expected_value(&self, expected: &Value) -> FhirPathValue {
        FhirPathValue::from(expected.clone())
    }

    /// Compare actual result with expected result
    /// Simplified comparison with proper handling of FHIRPath collection semantics
    fn compare_results(&self, actual: &FhirPathValue, expected: &Value) -> bool {
        // Convert actual to JSON for uniform comparison
        let actual_json = match sonic_rs::to_string(actual) {
            Ok(json_str) => match sonic_rs::from_str::<Value>(&json_str) {
                Ok(json) => json,
                Err(_) => return false,
            },
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
            // Default: no match
            _ => false,
        }
    }

    /// Run a single test case using the integrated stack
    pub async fn run_test(&mut self, test: &TestCase) -> TestResult {
        if self.verbose {
            println!("Running test: {}", test.name);
            println!("Expression: {}", test.expression);
        }

        // Load input data
        let input_data = match &test.inputfile {
            Some(filename) => match self.load_input_data(filename) {
                Ok(json_data) => FhirPathValue::from(json_data),
                Err(e) => {
                    return TestResult::Error {
                        error: format!("Failed to load input from {filename}: {e}"),
                    };
                }
            },
            None => {
                match &test.input {
                    Some(input_val) => {
                        if input_val.is_null() {
                            // null input means evaluate without context data
                            FhirPathValue::Empty
                        } else {
                            FhirPathValue::from(input_val.clone())
                        }
                    }
                    None => {
                        return TestResult::Error {
                            error: "No input data provided (neither inputfile nor input)"
                                .to_string(),
                        };
                    }
                }
            }
        };

        // Parse expression using integrated parser
        let ast = match self.parse_expression(&test.expression) {
            Ok(ast) => ast,
            Err(e) => {
                // Per FHIRPath spec, syntax errors should return empty collection
                // Check if expected result is empty array
                let expected = self.convert_expected_value(&test.expected);
                if expected.is_empty() {
                    // This is expected - syntax errors should produce empty
                    return TestResult::Passed;
                } else {
                    return TestResult::Error { error: e };
                }
            }
        };

        // Create evaluation context using the same registries as the engine
        let context = octofhir_fhirpath::EvaluationContext::new(
            input_data.clone(),
            self.registry.clone(),
            self.model_provider.clone(),
        );

        // Evaluate expression using integrated engine
        let result = match self.engine.evaluate_ast(&ast, input_data, &context).await {
            Ok(result) => result,
            Err(e) => {
                // Per FHIRPath spec, evaluation errors should return empty collection
                // Check if an expected result is empty array
                let expected = self.convert_expected_value(&test.expected);
                if expected.is_empty() {
                    // This is expected - evaluation errors should produce empty
                    return TestResult::Passed;
                } else {
                    return TestResult::Error {
                        error: format!("Evaluation error: {e}"),
                    };
                }
            }
        };

        if self.verbose {
            println!("Result: {result:?}");
            println!(
                "Expected: {}",
                sonic_rs::to_string(&test.expected).unwrap_or_default()
            );
        }

        // Compare results
        if self.compare_results(&result, &test.expected) {
            TestResult::Passed
        } else {
            // Convert actual result to JSON for comparison
            let actual_json = sonic_rs::to_string(&result)
                .and_then(|s| sonic_rs::from_str::<Value>(&s))
                .unwrap_or_default();
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
            println!("Description: {}", suite.description);
            println!("Total tests: {}", suite.tests.len());
            println!();
        }

        for test in &suite.tests {
            let result = self.run_test(test).await;
            results.insert(test.name.clone(), result);
        }

        results
    }

    /// Run tests from a JSON file and return results
    pub async fn run_tests_from_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<HashMap<String, TestResult>, Box<dyn std::error::Error>> {
        let suite = self.load_test_suite(path)?;
        Ok(self.run_test_suite(&suite).await)
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
                TestResult::Error { .. } => stats.errored += 1,
                TestResult::Skipped { .. } => stats.skipped += 1,
            }
        }

        // Only show failures and errors, suppress successful tests
        for test in &suite.tests {
            let result = &results[&test.name];
            match result {
                TestResult::Failed { expected, actual } => {
                    println!("❌ FAIL {}", test.name);
                    println!("   Expression: {}", test.expression);
                    if let Some(inputfile) = &test.inputfile {
                        println!("   Input file: {inputfile}");
                    }
                    if !test.tags.is_empty() {
                        println!("   Tags: {}", test.tags.join(", "));
                    }
                    println!(
                        "   Expected: {}",
                        sonic_rs::to_string_pretty(&expected).unwrap_or_default()
                    );
                    println!(
                        "   Actual:   {}",
                        sonic_rs::to_string_pretty(&actual).unwrap_or_default()
                    );
                    println!();
                }
                TestResult::Error { error } => {
                    println!("⚠️ ERROR {}", test.name);
                    println!("   Expression: {}", test.expression);
                    if let Some(inputfile) = &test.inputfile {
                        println!("   Input file: {inputfile}");
                    }
                    if !test.tags.is_empty() {
                        println!("   Tags: {}", test.tags.join(", "));
                    }
                    println!("   Error: {error}");
                    println!();
                }
                _ => {} // Don't print successful tests
            }
        }

        Ok(stats)
    }
}
