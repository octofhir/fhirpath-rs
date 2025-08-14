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

//! Compatibility tests to ensure unified engine matches existing API behavior

use super::{TestUtils, as_single_boolean, as_single_string, count, as_collection};
use serde_json::json;

#[tokio::test]
async fn test_api_compatibility() {
    let unified_engine = TestUtils::create_test_engine();
    
    // Test that basic API methods work as expected
    let patient = TestUtils::sample_patient();
    
    // Basic evaluation
    let result = unified_engine.evaluate("name.given", patient.clone()).await;
    assert!(result.is_ok(), "Basic evaluation should work");
    
    // Evaluation with variables
    let mut variables = std::collections::HashMap::new();
    variables.insert("testVar".to_string(), octofhir_fhirpath_model::FhirPathValue::String("test".into()));
    
    let result = unified_engine.evaluate_with_variables("%testVar", json!({}), variables).await;
    assert!(result.is_ok(), "Variable evaluation should work");
}

#[tokio::test]
async fn test_error_compatibility() {
    let engine = TestUtils::create_test_engine();
    
    // Test that errors are handled consistently
    let error_cases = vec![
        "unknownFunction()",
        "5 +", 
        "(((",
        "[1,2,3][",
    ];
    
    for expression in error_cases {
        let result = engine.evaluate(expression, json!({})).await;
        assert!(result.is_err(), "Expression '{}' should produce error", expression);
    }
}

#[tokio::test]
async fn test_result_format_compatibility() {
    let engine = TestUtils::create_test_engine();
    
    // Test that results are in expected format
    let patient = TestUtils::sample_patient();
    
    // Single value result
    let result = engine.evaluate("gender", patient.clone()).await.unwrap();
    assert!(as_single_string(&result).is_some(), "Single string should be accessible");
    
    // Collection result
    let result = engine.evaluate("name.given", patient.clone()).await.unwrap();
    assert!(as_collection(&result).is_some(), "Collection should be accessible");
    assert!(count(&result) > 0, "Collection should have items");
    
    // Boolean result
    let result = engine.evaluate("name.exists()", patient).await.unwrap();
    assert!(as_single_boolean(&result).is_some(), "Boolean should be accessible");
}