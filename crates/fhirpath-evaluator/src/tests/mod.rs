//! Comprehensive test suite for unified FHIRPath engine

pub mod engine_tests;              // Existing basic engine tests
pub mod unified_engine_tests;      // Core engine functionality
pub mod lambda_tests;             // Lambda evaluation tests  
pub mod performance_tests;        // Performance benchmarks
pub mod integration_tests;        // Real FHIR resource tests
pub mod compatibility_tests;      // Compatibility with existing API
pub mod stress_tests;            // Memory and load tests
pub mod regression_tests;        // Regression prevention
pub mod edge_case_tests;         // Edge cases and error handling
pub mod official_tests;          // Official FHIRPath test suite
pub mod validation_pipeline;     // Automated validation

use super::engine::{FhirPathEngine, EvaluationConfig};
use octofhir_fhirpath_model::FhirPathValue;
use serde_json::json;
use std::time::Instant;

/// Test helper utilities
pub struct TestUtils;

impl TestUtils {
    /// Create engine with test configuration
    pub fn create_test_engine() -> FhirPathEngine {
        use octofhir_fhirpath_registry::create_standard_registries;
        use octofhir_fhirpath_model::MockModelProvider;
        use std::sync::Arc;
        
        let config = EvaluationConfig {
            max_recursion_depth: 100, // Lower for tests
            timeout_ms: 5000,         // 5 second timeout
            enable_lambda_optimization: true,
            memory_limit_mb: Some(50), // 50MB limit for tests
        };
        
        let (functions, operators) = create_standard_registries();
        let model_provider = Arc::new(MockModelProvider::empty());
        
        FhirPathEngine::new_with_config(
            Arc::new(functions),
            Arc::new(operators), 
            model_provider,
            config,
        )
    }
    
    /// Create engine with custom model provider
    pub fn create_engine_with_provider(provider: Box<dyn octofhir_fhirpath_model::ModelProvider>) -> FhirPathEngine {
        use octofhir_fhirpath_registry::create_standard_registries;
        let (functions, operators) = create_standard_registries();
        FhirPathEngine::new(
            std::sync::Arc::new(functions),
            std::sync::Arc::new(operators),
            provider.into()
        )
    }
    
    /// Benchmark expression evaluation
    pub async fn benchmark_expression(
        engine: &FhirPathEngine,
        expression: &str,
        data: serde_json::Value,
        iterations: usize,
    ) -> (std::time::Duration, Result<FhirPathValue, Box<dyn std::error::Error + Send + Sync>>) {
        let start = Instant::now();
        let mut result = None;
        
        for _ in 0..iterations {
            match engine.evaluate(expression, data.clone()).await {
                Ok(r) => result = Some(Ok(r)),
                Err(e) => return (start.elapsed(), Err(Box::new(e))),
            }
        }
        
        (start.elapsed(), result.unwrap())
    }
    
    /// Create sample FHIR Patient resource
    pub fn sample_patient() -> serde_json::Value {
        json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [
                {
                    "use": "official",
                    "family": "Doe",
                    "given": ["John", "Robert"]
                },
                {
                    "use": "nickname",
                    "given": ["Johnny"]
                }
            ],
            "gender": "male",
            "birthDate": "1974-12-25",
            "address": [
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "city": "Anytown",
                    "state": "CA",
                    "postalCode": "12345"
                }
            ],
            "telecom": [
                {
                    "system": "phone",
                    "value": "555-1234",
                    "use": "home"
                },
                {
                    "system": "email",
                    "value": "john.doe@example.com",
                    "use": "work"
                }
            ]
        })
    }
    
    /// Create sample FHIR Bundle resource
    pub fn sample_bundle() -> serde_json::Value {
        json!({
            "resourceType": "Bundle",
            "id": "example",
            "type": "collection",
            "entry": [
                {
                    "resource": Self::sample_patient()
                },
                {
                    "resource": {
                        "resourceType": "Observation",
                        "id": "obs1",
                        "status": "final",
                        "code": {
                            "coding": [{"system": "http://loinc.org", "code": "29463-7"}]
                        },
                        "subject": {"reference": "Patient/example"},
                        "valueQuantity": {"value": 185, "unit": "cm"}
                    }
                }
            ]
        })
    }
    
    /// Create sample numeric data for mathematical tests
    pub fn numeric_test_data() -> serde_json::Value {
        json!([1, 2, 3, 4, 5, -1, -2, 0, 100, 1000])
    }
    
    /// Create sample string data for string function tests
    pub fn string_test_data() -> serde_json::Value {
        json!(["hello", "world", "test", "UPPERCASE", "lowercase", "", "with spaces", "123"])
    }
    
    /// Create sample boolean data
    pub fn boolean_test_data() -> serde_json::Value {
        json!([true, false, true, true, false])
    }
    
    /// Create complex nested data structure
    pub fn complex_nested_data() -> serde_json::Value {
        json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "value": "deep",
                            "array": [1, 2, 3],
                            "nested_objects": [
                                {"id": 1, "name": "first"},
                                {"id": 2, "name": "second"}
                            ]
                        }
                    }
                }
            },
            "simple_array": [10, 20, 30],
            "mixed_data": {
                "numbers": [1.5, 2.5, 3.14],
                "strings": ["a", "b", "c"],
                "booleans": [true, false, true]
            }
        })
    }
    
}

/// Get integer value (single or first from collection)
pub fn as_single_integer(value: &FhirPathValue) -> Option<i64> {
    match value {
        FhirPathValue::Integer(i) => Some(*i),
        FhirPathValue::Collection(items) => {
            items.first().and_then(|v| v.as_integer())
        }
        _ => None,
    }
}

/// Get string value (single or first from collection)
pub fn as_single_string(value: &FhirPathValue) -> Option<String> {
    match value {
        FhirPathValue::String(s) => Some(s.to_string()),
        FhirPathValue::Collection(items) => {
            items.first().and_then(|v| v.as_string().map(|s| s.to_string()))
        }
        _ => None,
    }
}

/// Get boolean value (single or first from collection)
pub fn as_single_boolean(value: &FhirPathValue) -> Option<bool> {
    match value {
        FhirPathValue::Boolean(b) => Some(*b),
        FhirPathValue::Collection(items) => {
            items.first().and_then(|v| v.as_boolean())
        }
        _ => None,
    }
}

/// Get decimal value (single or first from collection)
pub fn as_single_decimal(value: &FhirPathValue) -> Option<rust_decimal::Decimal> {
    match value {
        FhirPathValue::Decimal(d) => Some(*d),
        FhirPathValue::Collection(items) => {
            items.first().and_then(|v| v.as_decimal().copied())
        }
        _ => None,
    }
}

/// Get collection reference
pub fn as_collection(value: &FhirPathValue) -> Option<&octofhir_fhirpath_model::Collection> {
    match value {
        FhirPathValue::Collection(c) => Some(c),
        _ => None,
    }
}

/// Get collection as vector of strings
pub fn as_collection_strings(value: &FhirPathValue) -> Option<Vec<String>> {
    match value {
        FhirPathValue::Collection(items) => {
            Some(items.iter()
                .filter_map(|v| v.as_string().map(|s| s.to_string()))
                .collect())
        }
        FhirPathValue::String(s) => Some(vec![s.to_string()]),
        _ => None,
    }
}

/// Get collection as vector of integers
pub fn as_collection_integers(value: &FhirPathValue) -> Option<Vec<i64>> {
    match value {
        FhirPathValue::Collection(items) => {
            Some(items.iter()
                .filter_map(|v| v.as_integer())
                .collect())
        }
        FhirPathValue::Integer(i) => Some(vec![*i]),
        _ => None,
    }
}

/// Get count of items in value
pub fn count(value: &FhirPathValue) -> usize {
    value.len()
}

/// Compare FhirPathValue results for testing
pub fn values_equal(actual: &FhirPathValue, expected: &FhirPathValue) -> bool {
    match (actual, expected) {
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(e)) => a == e,
        (FhirPathValue::Integer(a), FhirPathValue::Integer(e)) => a == e,
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(e)) => a == e,
        (FhirPathValue::String(a), FhirPathValue::String(e)) => a == e,
        (FhirPathValue::Collection(a), FhirPathValue::Collection(e)) => {
            a.len() == e.len() && a.iter().zip(e.iter()).all(|(av, ev)| values_equal(av, ev))
        },
        (FhirPathValue::Empty, FhirPathValue::Empty) => true,
        _ => false,
    }
}

/// Assert that two values are equal with helpful error message
pub fn assert_values_equal(actual: &FhirPathValue, expected: &FhirPathValue, context: &str) {
    if !values_equal(actual, expected) {
        panic!("Values not equal in {}: expected {:?}, got {:?}", context, expected, actual);
    }
}

impl TestUtils {
}

#[cfg(test)]
mod test_utils_tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = TestUtils::create_test_engine();
        
        // Test basic functionality
        let result = engine.evaluate("42", json!({})).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_sample_data_creation() {
        let patient = TestUtils::sample_patient();
        assert_eq!(patient["resourceType"], "Patient");
        assert_eq!(patient["id"], "example");
        
        let bundle = TestUtils::sample_bundle();
        assert_eq!(bundle["resourceType"], "Bundle");
        assert_eq!(bundle["type"], "collection");
        
        let numeric = TestUtils::numeric_test_data();
        assert!(numeric.is_array());
        
        let strings = TestUtils::string_test_data();
        assert!(strings.is_array());
    }

    #[test]
    fn test_value_comparison() {
        let val1 = FhirPathValue::Integer(42);
        let val2 = FhirPathValue::Integer(42);
        let val3 = FhirPathValue::Integer(43);
        
        assert!(super::values_equal(&val1, &val2));
        assert!(!super::values_equal(&val1, &val3));
        
        let collection1 = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1), 
            FhirPathValue::Integer(2)
        ]);
        let collection2 = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1), 
            FhirPathValue::Integer(2)
        ]);
        
        assert!(super::values_equal(&collection1, &collection2));
    }
}