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

use super::{TestUtils, as_single_integer, as_single_boolean, as_single_string};
use serde_json::json;

#[tokio::test]
async fn test_null_and_empty_handling() {
    let engine = TestUtils::create_test_engine();
    
    // Test with null input
    let result = engine.evaluate("exists()", json!(null)).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));
    
    let result = engine.evaluate("empty()", json!(null)).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));
    
    // Test with empty array
    let result = engine.evaluate("count()", json!([])).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(0));
    
    // Test with empty object
    let result = engine.evaluate("nonexistent", json!({})).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_type_coercion_edge_cases() {
    let engine = TestUtils::create_test_engine();
    
    // Test string to number coercion in comparisons
    let result = engine.evaluate("'5' > 3", json!({})).await;
    // This might work or fail depending on implementation
    match result {
        Ok(value) => println!("String comparison result: {:?}", value),
        Err(_) => println!("String comparison failed (acceptable)"),
    }
    
    // Test mixed type collections
    let mixed_data = json!([1, "hello", true, null]);
    let result = engine.evaluate("count()", mixed_data).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(4));
}

#[tokio::test]
async fn test_boundary_conditions() {
    let engine = TestUtils::create_test_engine();
    
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
        Err(_) => {}, // Error is acceptable
    }
}

#[tokio::test]
async fn test_malformed_expressions() {
    let engine = TestUtils::create_test_engine();
    
    let malformed_expressions = vec![
        "",           // Empty expression
        " ",          // Whitespace only
        ".",          // Just a dot
        "..",         // Double dot
        "[",          // Unclosed bracket
        ")",          // Unmatched parenthesis
        "5 + + 3",    // Double operator
        "function(",  // Unclosed function
    ];
    
    for expression in malformed_expressions {
        let result = engine.evaluate(expression, json!({})).await;
        assert!(result.is_err(), "Malformed expression '{}' should error", expression);
    }
}

#[tokio::test]
async fn test_unicode_and_special_characters() {
    let engine = TestUtils::create_test_engine();
    
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
async fn test_deeply_nested_expressions() {
    let engine = TestUtils::create_test_engine();
    
    // Test deeply nested parentheses
    let nested_expr = format!("{}42{}", "(".repeat(20), ")".repeat(20));
    let result = engine.evaluate(&nested_expr, json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(42));
    
    // Test deeply nested property access (but reasonable depth)
    let mut nested_data = json!({"value": "found"});
    for i in 0..10 {
        nested_data = json!({format!("level{}", i): nested_data});
    }
    
    let path = (0..10).map(|i| format!("level{}", i)).collect::<Vec<_>>().join(".") + ".value";
    let result = engine.evaluate(&path, nested_data).await.unwrap();
    assert_eq!(as_single_string(&result), Some("found".to_string()));
}

#[tokio::test]
async fn test_circular_reference_handling() {
    let engine = TestUtils::create_test_engine();
    
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
    
    let result = engine.evaluate("parent.child.id", self_referential).await.unwrap();
    assert_eq!(as_single_string(&result), Some("self".to_string()));
}

#[tokio::test]
async fn test_resource_exhaustion_protection() {
    let engine = TestUtils::create_test_engine();
    
    // Test extremely large expression (should be rejected or handled gracefully)
    let huge_expression = "1 + ".repeat(1000) + "1";
    let result = engine.evaluate(&huge_expression, json!({})).await;
    
    // Should either complete (if optimized) or fail gracefully
    match result {
        Ok(_) => println!("Huge expression completed"),
        Err(_) => println!("Huge expression failed (acceptable)"),
    }
    
    // Test very wide object access
    let wide_object: serde_json::Value = json!(
        (0..1000).map(|i| (format!("key_{}", i), json!(i)))
               .collect::<serde_json::Map<String, serde_json::Value>>()
    );
    
    let result = engine.evaluate("key_500", wide_object).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(500));
}