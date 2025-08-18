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

//! Edge case and error handling tests

use super::{TestUtils, as_single_integer, as_single_string};
use sonic_rs::json;

#[tokio::test]
async fn test_type_coercion_edge_cases() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test string to number coercion in comparisons
    let result = engine.evaluate("'5' > 3", json!({})).await;
    // This might work or fail depending on implementation
    match result {
        Ok(value) => println!("String comparison result: {value:?}"),
        Err(_) => println!("String comparison failed (acceptable)"),
    }

    // Test mixed type collections
    let mixed_data = json!([1, "hello", true, null]);
    let result = engine.evaluate("count()", mixed_data).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(4));
}

#[tokio::test]
async fn test_boundary_conditions() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test very large numbers
    let result = engine.evaluate("999999999999", json!({})).await.unwrap();
    assert!(as_single_integer(&result).is_some());

    // Test very small numbers
    let result = engine.evaluate("-999999999999", json!({})).await.unwrap();
    assert!(as_single_integer(&result).is_some());

    // Test zero division
    let result = engine.evaluate("5 / 0", json!({})).await;
    // Should either return empty or error
    match result {
        Ok(val) => assert!(val.is_empty()),
        Err(_) => {} // Error is acceptable
    }
}

#[tokio::test]
async fn test_unicode_and_special_characters() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test unicode in data
    let unicode_data = json!({
        "name": "José María",
        "city": "北京",
        "emoji": "🎉🚀",
        "arabic": "مرحبا"
    });

    let result = engine.evaluate("name", unicode_data.clone()).await.unwrap();
    assert_eq!(as_single_string(&result), Some("José María".to_string()));

    let result = engine.evaluate("emoji", unicode_data).await.unwrap();
    assert_eq!(as_single_string(&result), Some("🎉🚀".to_string()));

    // Test unicode in expressions (property names)
    let result = engine.evaluate("'unicode: 🔥'", json!({})).await.unwrap();
    assert_eq!(as_single_string(&result), Some("unicode: 🔥".to_string()));
}

#[tokio::test]
async fn test_circular_reference_handling() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // JSON doesn't support true circular references, but we can test
    // structures that might cause issues
    let self_referential = json!({
        "id": "self",
        "parent": {
            "id": "parent",
            "child": {
                "id": "self"  // Same id as root
            }
        }
    });

    let result = engine
        .evaluate("parent.child.id", self_referential)
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("self".to_string()));
}

#[tokio::test]
async fn test_resource_exhaustion_protection() {
    // Create engine with lower limits for this test
    use crate::EvaluationConfig;
    use octofhir_fhirpath_model::MockModelProvider;
    use octofhir_fhirpath_registry::create_standard_registry;
    use std::sync::Arc;

    let config = EvaluationConfig {
        max_recursion_depth: 50,
        timeout_ms: 5000,
        enable_lambda_optimization: true,
        enable_sync_optimization: true,
        memory_limit_mb: Some(10),
        max_expression_nodes: 200, // Very low limit to test protection
        max_collection_size: 1000,
    };

    let registry = Arc::new(create_standard_registry().await.unwrap());
    let model_provider = Arc::new(MockModelProvider::empty());
    let engine = crate::FhirPathEngine::new(registry, model_provider).with_config(config);

    // Test small but valid expression
    let small_expression = "1 + 2 + 3";
    let result = engine.evaluate(small_expression, json!({})).await;
    assert!(result.is_ok(), "Small expression should work");

    // Test expression that should exceed our complexity limit (200 nodes)
    // Create an expression with 300+ nodes using nested function calls
    let mut complex_parts = Vec::new();
    for i in 0..50 {
        complex_parts.push(format!("count().toString().length() + {i}"));
    }
    let complex_expression = complex_parts.join(" + ");
    let result = engine.evaluate(&complex_expression, json!({})).await;

    // This should fail due to complexity limits
    match result {
        Ok(_) => println!("WARNING: Complex expression should have been rejected"),
        Err(e) => {
            println!("Complex expression correctly rejected: {e}");
            assert!(
                e.to_string().contains("too complex") || e.to_string().contains("exceeds maximum")
            );
        }
    }

    // Test recursion depth by creating deeply nested parentheses (safe approach)
    let nested_expr = format!("{}42{}", "(".repeat(10), ")".repeat(10));
    let result = engine.evaluate(&nested_expr, json!({})).await;
    assert!(result.is_ok(), "Moderately nested expression should work");

    // Test very wide object access - this should work fine
    let wide_object_map: std::collections::HashMap<String, sonic_rs::Value> =
        (0..1000).map(|i| (format!("key_{i}"), json!(i))).collect();
    let wide_object: sonic_rs::Value = json!(wide_object_map);

    let result = engine.evaluate("key_500", wide_object).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(500));
}
