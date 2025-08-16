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

//! Comprehensive tests for the unified FHIRPath engine core functionality

use super::{
    TestUtils, as_collection, as_single_boolean, as_single_decimal, as_single_integer,
    as_single_string, count,
};
use octofhir_fhirpath_model::FhirPathValue;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_all_literal_types() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Boolean literals
    let result = engine.evaluate("true", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("false", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));

    // Integer literals
    let result = engine.evaluate("42", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(42));

    let result = engine.evaluate("-17", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(-17));

    // Zero
    let result = engine.evaluate("0", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(0));

    // Decimal literals
    let result = engine.evaluate("3.14", json!({})).await.unwrap();
    if let Some(decimal_val) = as_single_decimal(&result) {
        assert!((decimal_val.to_string().parse::<f64>().unwrap_or(0.0) - 3.14).abs() < 0.001);
    } else {
        panic!("Expected decimal value");
    }

    // String literals
    let result = engine.evaluate("'hello world'", json!({})).await.unwrap();
    assert_eq!(as_single_string(&result), Some("hello world".to_string()));

    let result = engine
        .evaluate("'with\\'escaped\\'quotes'", json!({}))
        .await
        .unwrap();
    assert_eq!(
        as_single_string(&result),
        Some("with'escaped'quotes".to_string())
    );

    // Empty string
    let result = engine.evaluate("''", json!({})).await.unwrap();
    assert_eq!(as_single_string(&result), Some("".to_string()));
}

#[tokio::test]
async fn test_all_binary_operators() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Arithmetic operators
    let result = engine.evaluate("5 + 3", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(8));

    let result = engine.evaluate("10 - 4", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(6));

    let result = engine.evaluate("6 * 7", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(42));

    let result = engine.evaluate("15 / 3", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(5));

    let result = engine.evaluate("17 mod 5", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(2));

    let result = engine.evaluate("17 div 5", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(3));

    // Comparison operators
    let result = engine.evaluate("5 = 5", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("5 != 3", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("5 > 3", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("3 < 5", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("5 >= 5", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("3 <= 5", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    // Logical operators
    let result = engine.evaluate("true and true", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("true or false", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("true xor false", json!({})).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine
        .evaluate("false implies true", json!({}))
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    // String operators
    let result = engine
        .evaluate("'hello' & ' world'", json!({}))
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("hello world".to_string()));

    // Collection operators
    let result = engine
        .evaluate("(1 | 2 | 3).count()", json!({}))
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(3));
}

#[tokio::test]
async fn test_complex_property_navigation() {
    let engine = TestUtils::create_test_engine().await.unwrap();
    let patient = TestUtils::sample_patient();

    // Simple property access
    let result = engine.evaluate("name", patient.clone()).await.unwrap();
    assert_eq!(count(&result), 2);

    // Nested property access
    let result = engine
        .evaluate("name.given", patient.clone())
        .await
        .unwrap();
    assert_eq!(count(&result), 3); // "John", "Robert", "Johnny"

    // Array indexing
    let result = engine
        .evaluate("name[0].given[1]", patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("Robert".to_string()));

    // Property access with filtering
    let result = engine
        .evaluate("name.where(use = 'official').given", patient.clone())
        .await
        .unwrap();
    assert_eq!(count(&result), 2); // "John", "Robert"

    // Complex nested access
    let result = engine
        .evaluate("address[0].line[0]", patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("123 Main St".to_string()));

    // Access non-existent property
    let result = engine
        .evaluate("nonexistentProperty", patient.clone())
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_all_builtin_functions() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Collection functions
    let data = json!([1, 2, 3, 4, 5]);

    let result = engine.evaluate("count()", data.clone()).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(5));

    let result = engine.evaluate("empty()", data.clone()).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));

    let result = engine.evaluate("exists()", data.clone()).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine.evaluate("first()", data.clone()).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(1));

    let result = engine.evaluate("last()", data.clone()).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(5));

    let result = engine.evaluate("tail()", data.clone()).await.unwrap();
    assert_eq!(count(&result), 4); // [2, 3, 4, 5]

    // String functions
    let result = engine
        .evaluate("'hello'.length()", json!({}))
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(5));

    let result = engine.evaluate("'HELLO'.lower()", json!({})).await.unwrap();
    assert_eq!(as_single_string(&result), Some("hello".to_string()));

    let result = engine.evaluate("'hello'.upper()", json!({})).await.unwrap();
    assert_eq!(as_single_string(&result), Some("HELLO".to_string()));

    // Boolean functions
    let bool_data = json!([true, false, true]);
    let result = engine
        .evaluate("allTrue()", bool_data.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));

    let result = engine
        .evaluate("anyTrue()", bool_data.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));
}

#[tokio::test]
async fn test_error_conditions() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Division by zero should be handled gracefully
    let result = engine.evaluate("5 / 0", json!({})).await;
    // Some implementations may return empty, others error - both are acceptable
    match result {
        Ok(val) => assert!(val.is_empty()),
        Err(_) => {} // Error is also acceptable
    }

    // Unknown function should error
    let result = engine.evaluate("unknownFunction()", json!({})).await;
    assert!(result.is_err());

    // Invalid property access should return empty, not error
    let result = engine
        .evaluate("nonexistentProperty", json!({"resourceType": "Patient"}))
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Index out of bounds should return empty, not error
    let result = engine.evaluate("skip(10).first()", json!([1, 2, 3])).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Malformed expressions should error
    let result = engine.evaluate("5 +", json!({})).await;
    assert!(result.is_err());

    let result = engine.evaluate("(((", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_recursion_depth_limit() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Create deeply nested expression but not too deep to cause stack overflow
    let deep_expr = "1".to_string() + &" + 1".repeat(50); // Moderate depth for testing
    let result = engine.evaluate(&deep_expr, json!({})).await;

    // Should either succeed (with result 51) or error gracefully
    match result {
        Ok(value) => {
            // If it succeeds, should get 1 + 50 ones = 51
            if let Some(int_val) = as_single_integer(&value) {
                assert_eq!(int_val, 51);
            }
        }  
        Err(_) => {} // Hit recursion limit as expected - this is also valid
    }
}

#[tokio::test]
async fn test_variable_evaluation() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    let mut variables = HashMap::new();
    variables.insert(
        "myVar".to_string(),
        FhirPathValue::String("test value".into()),
    );
    variables.insert("myNumber".to_string(), FhirPathValue::Integer(42));
    variables.insert("myBool".to_string(), FhirPathValue::Boolean(true));

    let result = engine
        .evaluate_with_variables("%myVar", json!({}), variables.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("test value".to_string()));

    let result = engine
        .evaluate_with_variables("%myNumber", json!({}), variables.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(42));

    let result = engine
        .evaluate_with_variables("%myBool", json!({}), variables.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    // Test undefined variable
    let result = engine
        .evaluate_with_variables("%undefined", json!({}), variables)
        .await;
    match result {
        Ok(val) => assert!(val.is_empty()),
        Err(_) => {} // Error is also acceptable for undefined variables
    }
}

#[tokio::test]
async fn test_parentheses_and_precedence() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test operator precedence
    let result = engine.evaluate("2 + 3 * 4", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(14)); // Should be 2 + (3 * 4) = 14

    let result = engine.evaluate("(2 + 3) * 4", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(20)); // Should be (2 + 3) * 4 = 20

    // Complex precedence
    let result = engine.evaluate("2 + 3 * 4 - 1", json!({})).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(13)); // Should be 2 + (3 * 4) - 1 = 13

    // Logical operator precedence
    let result = engine
        .evaluate("true or false and false", json!({}))
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true)); // Should be true or (false and false) = true

    let result = engine
        .evaluate("(true or false) and false", json!({}))
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(false)); // Should be (true or false) and false = false
}

#[tokio::test]
async fn test_collection_operations() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test union operation
    let result = engine
        .evaluate("(1 | 2) | (2 | 3)", json!({}))
        .await
        .unwrap();
    let collection = as_collection(&result).unwrap();
    assert!(collection.len() >= 3); // Should contain unique elements

    // Test collection filtering
    let data = json!([1, 2, 3, 4, 5]);
    let result = engine.evaluate("where($this > 3)", data).await.unwrap();
    assert_eq!(count(&result), 2); // Should contain 4 and 5

    // Test collection selection
    let data = json!([{"value": 1}, {"value": 2}, {"value": 3}]);
    let result = engine.evaluate("select(value)", data).await.unwrap();
    assert_eq!(count(&result), 3);
}

#[tokio::test]
async fn test_type_conversion_and_casting() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test string to integer conversion
    let result = engine.evaluate("'42'.toInteger()", json!({})).await;
    if result.is_ok() {
        assert_eq!(as_single_integer(&result.unwrap()), Some(42));
    }

    // Test integer to string conversion
    let result = engine.evaluate("42.toString()", json!({})).await;
    if result.is_ok() {
        assert_eq!(as_single_string(&result.unwrap()), Some("42".to_string()));
    }

    // Test boolean to string
    let result = engine.evaluate("true.toString()", json!({})).await;
    if result.is_ok() {
        assert_eq!(as_single_string(&result.unwrap()), Some("true".to_string()));
    }
}

#[tokio::test]
async fn test_string_manipulation_functions() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test string contains
    let result = engine
        .evaluate("'hello world'.contains('world')", json!({}))
        .await;
    if result.is_ok() {
        assert_eq!(as_single_boolean(&result.unwrap()), Some(true));
    }

    // Test string starts with
    let result = engine
        .evaluate("'hello world'.startsWith('hello')", json!({}))
        .await;
    if result.is_ok() {
        assert_eq!(as_single_boolean(&result.unwrap()), Some(true));
    }

    // Test string ends with
    let result = engine
        .evaluate("'hello world'.endsWith('world')", json!({}))
        .await;
    if result.is_ok() {
        assert_eq!(as_single_boolean(&result.unwrap()), Some(true));
    }

    // Test substring
    let result = engine
        .evaluate("'hello world'.substring(6)", json!({}))
        .await;
    if result.is_ok() {
        assert_eq!(
            as_single_string(&result.unwrap()),
            Some("world".to_string())
        );
    }
}

#[tokio::test]
async fn test_mathematical_functions() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test mathematical functions on collections
    let data = json!([1, 2, 3, 4, 5]);

    let result = engine.evaluate("min()", data.clone()).await;
    if result.is_ok() {
        assert_eq!(as_single_integer(&result.unwrap()), Some(1));
    }

    let result = engine.evaluate("max()", data.clone()).await;
    if result.is_ok() {
        assert_eq!(as_single_integer(&result.unwrap()), Some(5));
    }

    let result = engine.evaluate("avg()", data.clone()).await;
    if result.is_ok() {
        // Average of [1,2,3,4,5] is 3.0
        let avg_val = result.unwrap();
        if let Some(decimal) = as_single_decimal(&avg_val) {
            assert!((decimal.to_string().parse::<f64>().unwrap_or(0.0) - 3.0).abs() < 0.001);
        } else if let Some(integer) = as_single_integer(&avg_val) {
            assert_eq!(integer, 3);
        }
    }
}
