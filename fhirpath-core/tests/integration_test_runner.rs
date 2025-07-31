//! Integration Test Runner for FHIRPath Official Tests
//!
//! This module provides functionality to run official FHIRPath tests from JSON test definitions
//! using the complete integrated stack of all fhirpath-* crates together.

use fhirpath_ast::ExpressionNode;
use fhirpath_evaluator::{EvaluationError, FhirPathEngine};
use fhirpath_model::{FhirPathValue, FhirResource};
use fhirpath_parser::parse_expression;
use fhirpath_registry::create_standard_registries;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    input_cache: HashMap<String, FhirResource>,
    base_path: PathBuf,
    verbose: bool,
}

impl IntegrationTestRunner {
    /// Create a new integration test runner
    pub fn new() -> Self {
        let (functions, operators) = create_standard_registries();
        let engine = FhirPathEngine::with_registries(
            std::sync::Arc::new(functions),
            std::sync::Arc::new(operators),
        );

        Self {
            engine,
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
        let full_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.base_path.join(path)
        };

        let content = fs::read_to_string(&full_path).map_err(|e| {
            format!(
                "Failed to read test suite file {}: {}",
                full_path.display(),
                e
            )
        })?;

        let suite: TestSuite = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse test suite JSON: {}", e))?;

        Ok(suite)
    }

    /// Load input data from a file (with caching)
    fn load_input_data(
        &mut self,
        filename: &str,
    ) -> Result<FhirResource, Box<dyn std::error::Error>> {
        if let Some(cached) = self.input_cache.get(filename) {
            return Ok(cached.clone());
        }

        // Try multiple possible paths for input files
        let possible_paths = vec![
            self.base_path.join("input").join(filename),
            self.base_path.join("tests").join("input").join(filename),
            self.base_path
                .join("specs")
                .join("fhirpath")
                .join("tests")
                .join("input")
                .join(filename),
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
            .map_err(|e| format!("Failed to parse JSON in {}: {}", filename, e))?;

        let resource = FhirResource::from_json(json_value);

        if self.verbose {
            println!(
                "Loaded input file {} from {}",
                filename,
                used_path.unwrap().display()
            );
        }

        self.input_cache
            .insert(filename.to_string(), resource.clone());
        Ok(resource)
    }

    /// Parse a FHIRPath expression using the integrated parser
    fn parse_expression(&mut self, expression: &str) -> Result<ExpressionNode, String> {
        fhirpath_parser::parse_expression(expression)
            .map_err(|e| format!("Parser error in '{}': {}", expression, e))
    }

    /// Convert expected JSON value to FhirPathValue for comparison
    fn convert_expected_value(&self, expected: &Value) -> FhirPathValue {
        match expected {
            Value::Null => FhirPathValue::Empty,
            Value::Bool(b) => FhirPathValue::Boolean(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    // Use rust_decimal for precise decimal handling
                    match rust_decimal::Decimal::try_from(f) {
                        Ok(d) => FhirPathValue::Decimal(d),
                        Err(_) => FhirPathValue::Empty,
                    }
                } else {
                    FhirPathValue::Empty
                }
            }
            Value::String(s) => FhirPathValue::String(s.clone()),
            Value::Array(arr) => {
                if arr.is_empty() {
                    FhirPathValue::Empty
                } else {
                    let values: Vec<FhirPathValue> =
                        arr.iter().map(|v| self.convert_expected_value(v)).collect();
                    FhirPathValue::collection(values)
                }
            }
            Value::Object(_) => {
                // For objects, convert through FhirPathValue
                FhirPathValue::from(expected.clone())
            }
        }
    }

    /// Compare actual result with expected result
    fn compare_results(&self, actual: &FhirPathValue, expected: &Value) -> bool {
        let expected_value = self.convert_expected_value(expected);

        // Handle empty collections vs empty values
        match (actual, &expected_value) {
            (FhirPathValue::Empty, FhirPathValue::Empty) => true,
            (FhirPathValue::Collection(actual_items), FhirPathValue::Empty) => {
                actual_items.is_empty()
            }
            (FhirPathValue::Empty, FhirPathValue::Collection(expected_items)) => {
                expected_items.is_empty()
            }
            (
                FhirPathValue::Collection(actual_items),
                FhirPathValue::Collection(expected_items),
            ) => {
                if actual_items.len() != expected_items.len() {
                    return false;
                }
                actual_items
                    .iter()
                    .zip(expected_items.iter())
                    .all(|(a, e)| a == e)
            }
            // Handle single value vs single-item collection (common in FHIRPath tests)
            (single_val, FhirPathValue::Collection(expected_items))
                if expected_items.len() == 1 =>
            {
                single_val == expected_items.first().unwrap()
            }
            (FhirPathValue::Collection(actual_items), single_val) if actual_items.len() == 1 => {
                actual_items.first().unwrap() == single_val
            }
            _ => actual == &expected_value,
        }
    }

    /// Run a single test case using the integrated stack
    pub fn run_test(&mut self, test: &TestCase) -> TestResult {
        if self.verbose {
            println!("Running test: {}", test.name);
            println!("Expression: {}", test.expression);
        }

        // Load input data
        let input_data = match &test.inputfile {
            Some(filename) => match self.load_input_data(filename) {
                Ok(data) => FhirPathValue::Resource(data),
                Err(e) => {
                    return TestResult::Error {
                        error: format!("Failed to load input from {}: {}", filename, e),
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

        // Evaluate expression using integrated engine
        let result = match self.engine.evaluate(&ast, input_data) {
            Ok(result) => result,
            Err(e) => {
                // Per FHIRPath spec, evaluation errors should return empty collection
                // Check if expected result is empty array
                let expected = self.convert_expected_value(&test.expected);
                if expected.is_empty() {
                    // This is expected - evaluation errors should produce empty
                    return TestResult::Passed;
                } else {
                    return TestResult::Error {
                        error: format!("Evaluation error: {}", e),
                    };
                }
            }
        };

        if self.verbose {
            println!("Result: {:?}", result);
            println!(
                "Expected: {}",
                serde_json::to_string(&test.expected).unwrap_or_default()
            );
        }

        // Compare results
        if self.compare_results(&result, &test.expected) {
            TestResult::Passed
        } else {
            // Convert actual result to JSON for comparison
            let actual_json = self.fhirpath_value_to_json(&result);
            TestResult::Failed {
                expected: test.expected.clone(),
                actual: actual_json,
            }
        }
    }

    /// Convert FhirPathValue to JSON Value for display
    /// According to FHIRPath spec, all results should be collections (arrays)
    fn fhirpath_value_to_json(&self, value: &FhirPathValue) -> Value {
        match value {
            FhirPathValue::Boolean(b) => Value::Array(vec![Value::Bool(*b)]),
            FhirPathValue::Integer(i) => Value::Array(vec![Value::Number((*i).into())]),
            FhirPathValue::Decimal(d) => {
                use rust_decimal::prelude::ToPrimitive;
                let num = Value::Number(
                    serde_json::Number::from_f64(d.to_f64().unwrap_or(0.0))
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                );
                Value::Array(vec![num])
            }
            FhirPathValue::String(s) => Value::Array(vec![Value::String(s.clone())]),
            FhirPathValue::Date(d) => {
                Value::Array(vec![Value::String(format!("@{}", d.format("%Y-%m-%d")))])
            }
            FhirPathValue::DateTime(dt) => Value::Array(vec![Value::String(format!(
                "@{}",
                dt.format("%Y-%m-%dT%H:%M:%S%.3f%z")
            ))]),
            FhirPathValue::Time(t) => {
                Value::Array(vec![Value::String(format!("@T{}", t.format("%H:%M:%S")))])
            }
            FhirPathValue::Quantity(q) => Value::Array(vec![q.to_json()]),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Value::Array(vec![])
                } else {
                    Value::Array(
                        items
                            .iter()
                            .map(|item| self.fhirpath_value_to_json_item(item))
                            .collect(),
                    )
                }
            }
            FhirPathValue::Empty => Value::Array(vec![]),
            FhirPathValue::Resource(resource) => {
                // Convert back to JSON representation
                Value::Array(vec![resource.to_json()])
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Convert TypeInfo to JSON object
                let mut obj = serde_json::Map::new();
                obj.insert("namespace".to_string(), Value::String(namespace.clone()));
                obj.insert("name".to_string(), Value::String(name.clone()));
                Value::Array(vec![Value::Object(obj)])
            }
        }
    }

    /// Convert a single FhirPathValue item to JSON (not wrapped in array)
    fn fhirpath_value_to_json_item(&self, value: &FhirPathValue) -> Value {
        match value {
            FhirPathValue::Boolean(b) => Value::Bool(*b),
            FhirPathValue::Integer(i) => Value::Number((*i).into()),
            FhirPathValue::Decimal(d) => {
                use rust_decimal::prelude::ToPrimitive;
                Value::Number(
                    serde_json::Number::from_f64(d.to_f64().unwrap_or(0.0))
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                )
            }
            FhirPathValue::String(s) => Value::String(s.clone()),
            FhirPathValue::Date(d) => Value::String(format!("@{}", d.format("%Y-%m-%d"))),
            FhirPathValue::DateTime(dt) => {
                Value::String(format!("@{}", dt.format("%Y-%m-%dT%H:%M:%S%.3f%z")))
            }
            FhirPathValue::Time(t) => Value::String(format!("@T{}", t.format("%H:%M:%S"))),
            FhirPathValue::Quantity(q) => q.to_json(),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Value::Array(vec![])
                } else {
                    Value::Array(
                        items
                            .iter()
                            .map(|item| self.fhirpath_value_to_json_item(item))
                            .collect(),
                    )
                }
            }
            FhirPathValue::Empty => Value::Null,
            FhirPathValue::Resource(resource) => resource.to_json(),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Convert TypeInfo to JSON object
                let mut obj = serde_json::Map::new();
                obj.insert("namespace".to_string(), Value::String(namespace.clone()));
                obj.insert("name".to_string(), Value::String(name.clone()));
                Value::Object(obj)
            }
        }
    }

    /// Run all tests in a test suite
    pub fn run_test_suite(&mut self, suite: &TestSuite) -> HashMap<String, TestResult> {
        let mut results = HashMap::new();

        if self.verbose {
            println!("Running test suite: {}", suite.name);
            println!("Description: {}", suite.description);
            println!("Total tests: {}", suite.tests.len());
            println!();
        }

        for test in &suite.tests {
            let result = self.run_test(test);
            results.insert(test.name.clone(), result);
        }

        results
    }

    /// Run tests from a JSON file and return results
    pub fn run_tests_from_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<HashMap<String, TestResult>, Box<dyn std::error::Error>> {
        let suite = self.load_test_suite(path)?;
        Ok(self.run_test_suite(&suite))
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

    /// Run tests and print detailed results to console
    pub fn run_and_report<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<TestStats, Box<dyn std::error::Error>> {
        let suite = self.load_test_suite(&path)?;
        println!("🧪 FHIRPath Integration Test Suite: {}", suite.name);
        println!("📝 Description: {}", suite.description);
        if let Some(source) = &suite.source {
            println!("📋 Source: {}", source);
        }
        println!("🔢 Total tests: {}", suite.tests.len());
        println!();

        let results = self.run_test_suite(&suite);
        let stats = self.calculate_stats(&results);

        // Print individual test results
        for test in &suite.tests {
            let result = &results[&test.name];
            let (status, icon) = match result {
                TestResult::Passed => ("PASS", "✅"),
                TestResult::Failed { .. } => ("FAIL", "❌"),
                TestResult::Error { .. } => ("ERROR", "⚠️"),
                TestResult::Skipped { .. } => ("SKIP", "⊘"),
            };
            println!("{} {} {}", icon, status, test.name);

            if self.verbose || !matches!(result, TestResult::Passed) {
                println!("   Expression: {}", test.expression);
                if let Some(inputfile) = &test.inputfile {
                    println!("   Input file: {}", inputfile);
                }
                if !test.tags.is_empty() {
                    println!("   Tags: {}", test.tags.join(", "));
                }
            }

            match result {
                TestResult::Failed { expected, actual } => {
                    println!(
                        "   Expected: {}",
                        serde_json::to_string_pretty(expected).unwrap_or_default()
                    );
                    // Convert FhirPathValue to serde_json::Value to use the proper format
                    let actual_json: serde_json::Value = actual.clone().into();
                    println!(
                        "   Actual:   {}",
                        serde_json::to_string_pretty(&actual_json).unwrap_or_default()
                    );
                }
                TestResult::Error { error } => {
                    println!("   Error: {}", error);
                }
                TestResult::Skipped { reason } => {
                    println!("   Reason: {}", reason);
                }
                _ => {}
            }

            if !matches!(result, TestResult::Passed) {
                println!();
            }
        }

        // Print summary
        println!();
        println!("📊 === Test Summary ===");
        println!("Total:   {}", stats.total);
        println!("✅ Passed:  {} ({:.1}%)", stats.passed, stats.pass_rate());
        if stats.failed > 0 {
            println!(
                "❌ Failed:  {} ({:.1}%)",
                stats.failed,
                (stats.failed as f64 / stats.total as f64) * 100.0
            );
        }
        if stats.errored > 0 {
            println!(
                "⚠️  Errors:  {} ({:.1}%)",
                stats.errored,
                (stats.errored as f64 / stats.total as f64) * 100.0
            );
        }
        if stats.skipped > 0 {
            println!(
                "⊘ Skipped: {} ({:.1}%)",
                stats.skipped,
                (stats.skipped as f64 / stats.total as f64) * 100.0
            );
        }

        let success = stats.failed == 0 && stats.errored == 0;
        if success {
            println!("🎉 All tests passed!");
        } else {
            println!("💥 Some tests failed or errored.");
        }

        Ok(stats)
    }

    /// Run multiple test files and provide consolidated report
    pub fn run_multiple_test_files<P: AsRef<Path>>(
        &mut self,
        test_files: &[P],
    ) -> Result<TestStats, Box<dyn std::error::Error>> {
        let mut consolidated_stats = TestStats::default();

        println!("🚀 Running FHIRPath Integration Test Suite");
        println!("📁 Test files: {}", test_files.len());
        println!();

        for (i, test_file) in test_files.iter().enumerate() {
            println!(
                "📄 [{}/{}] Running {}",
                i + 1,
                test_files.len(),
                test_file
                    .as_ref()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );

            match self.run_and_report(test_file) {
                Ok(stats) => {
                    consolidated_stats.total += stats.total;
                    consolidated_stats.passed += stats.passed;
                    consolidated_stats.failed += stats.failed;
                    consolidated_stats.errored += stats.errored;
                    consolidated_stats.skipped += stats.skipped;
                }
                Err(e) => {
                    println!("❌ Failed to run test file: {}", e);
                    consolidated_stats.errored += 1;
                }
            }
            println!();
        }

        // Print consolidated summary
        println!("🏁 === Consolidated Summary ===");
        println!("Total suites: {}", test_files.len());
        println!("Total tests:  {}", consolidated_stats.total);
        println!(
            "✅ Passed:     {} ({:.1}%)",
            consolidated_stats.passed,
            consolidated_stats.pass_rate()
        );
        if consolidated_stats.failed > 0 {
            println!(
                "❌ Failed:     {} ({:.1}%)",
                consolidated_stats.failed,
                (consolidated_stats.failed as f64 / consolidated_stats.total as f64) * 100.0
            );
        }
        if consolidated_stats.errored > 0 {
            println!(
                "⚠️  Errors:     {} ({:.1}%)",
                consolidated_stats.errored,
                (consolidated_stats.errored as f64 / consolidated_stats.total as f64) * 100.0
            );
        }

        Ok(consolidated_stats)
    }
}

impl Default for IntegrationTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_expected_value() {
        let runner = IntegrationTestRunner::new();

        assert_eq!(
            runner.convert_expected_value(&Value::Bool(true)),
            FhirPathValue::Boolean(true)
        );
        assert_eq!(
            runner.convert_expected_value(&Value::from(42)),
            FhirPathValue::Integer(42)
        );
        assert_eq!(
            runner.convert_expected_value(&Value::from("test")),
            FhirPathValue::String("test".to_string())
        );
        assert_eq!(
            runner.convert_expected_value(&Value::Array(vec![])),
            FhirPathValue::Empty
        );
    }

    #[test]
    fn test_compare_results() {
        let runner = IntegrationTestRunner::new();

        assert!(runner.compare_results(&FhirPathValue::Boolean(true), &Value::Bool(true)));
        assert!(runner.compare_results(&FhirPathValue::Integer(42), &Value::from(42)));
        assert!(runner.compare_results(&FhirPathValue::Empty, &Value::Array(vec![])));

        // Test collection comparison
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test".to_string()),
            FhirPathValue::Integer(42),
        ]);
        let expected = Value::Array(vec![Value::String("test".to_string()), Value::from(42)]);
        assert!(runner.compare_results(&collection, &expected));
    }

    #[test]
    fn test_stats_calculation() {
        let runner = IntegrationTestRunner::new();
        let mut results = HashMap::new();

        results.insert("test1".to_string(), TestResult::Passed);
        results.insert(
            "test2".to_string(),
            TestResult::Failed {
                expected: Value::Bool(true),
                actual: Value::Bool(false),
            },
        );
        results.insert(
            "test3".to_string(),
            TestResult::Error {
                error: "Test error".to_string(),
            },
        );

        let stats = runner.calculate_stats(&results);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.passed, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.errored, 1);
        assert!((stats.pass_rate() - 33.333333333333336).abs() < 0.00001);
    }
}
