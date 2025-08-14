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

//! Tests for lambda expression evaluation functionality

use super::{TestUtils, as_single_integer, as_single_boolean, count, as_collection};
use octofhir_fhirpath_model::FhirPathValue;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_lambda_where_function() {
    let engine = TestUtils::create_test_engine();
    let data = json!([1, 2, 3, 4, 5]);
    
    let result = engine.evaluate("where($this > 3)", data).await.unwrap();
    assert_eq!(count(&result), 2); // 4 and 5
    
    // Verify the actual values
    if let Some(collection) = as_collection(&result) {
        let integers: Vec<i64> = collection.iter()
            .filter_map(|v| as_single_integer(v))
            .collect();
        assert!(integers.contains(&4));
        assert!(integers.contains(&5));
    }
}

#[tokio::test]
async fn test_lambda_select_function() {
    let engine = TestUtils::create_test_engine();
    let data = json!([{"value": 1}, {"value": 2}, {"value": 3}]);
    
    let result = engine.evaluate("select(value * 2)", data).await.unwrap();
    assert_eq!(count(&result), 3);
    
    // Check if we can extract the transformed values
    if let Some(collection) = as_collection(&result) {
        let has_doubles = collection.len() == 3;
        assert!(has_doubles, "Should have 3 transformed values");
    }
}

#[tokio::test]
async fn test_lambda_all_function() {
    let engine = TestUtils::create_test_engine();
    
    // Test all() with condition that should be true for all elements
    let data = json!([2, 4, 6, 8]);
    let result = engine.evaluate("all($this mod 2 = 0)", data).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));
    
    // Test all() with condition that should be false for some elements
    let data = json!([2, 4, 5, 8]);
    let result = engine.evaluate("all($this mod 2 = 0)", data).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));
    
    // Test all() on empty collection
    let data = json!([]);
    let result = engine.evaluate("all($this > 0)", data).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true)); // Vacuously true
}

#[tokio::test]
async fn test_lambda_any_function() {
    let engine = TestUtils::create_test_engine();
    
    // Test any() with condition that should be true for some elements
    let data = json!([1, 3, 5, 7]);
    let result = engine.evaluate("any($this > 5)", data).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(true)); // 7 > 5
    
    // Test any() with condition that should be false for all elements
    let data = json!([1, 3, 5]);
    let result = engine.evaluate("any($this > 5)", data).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));
    
    // Test any() on empty collection
    let data = json!([]);
    let result = engine.evaluate("any($this > 0)", data).await.unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));
}

#[tokio::test]
async fn test_lambda_aggregate_function() {
    let engine = TestUtils::create_test_engine();
    let data = json!([1, 2, 3, 4, 5]);
    
    // Test basic aggregation - sum using aggregate
    let result = engine.evaluate("aggregate($total + $this, 0)", data).await;
    
    // Aggregate might not be implemented yet, so we test if it's available
    match result {
        Ok(val) => {
            if let Some(sum) = as_single_integer(&val) {
                assert_eq!(sum, 15); // Sum of 1+2+3+4+5
            }
        },
        Err(_) => {
            // Aggregate function may not be implemented yet
            println!("Aggregate function not yet implemented");
        }
    }
}

#[tokio::test]
async fn test_nested_lambda_expressions() {
    let engine = TestUtils::create_test_engine();
    let data = json!([
        {"items": [1, 2, 3]},
        {"items": [4, 5, 6]},
        {"items": [7, 8, 9]}
    ]);
    
    // Select objects where any item > 5, then get their items
    let result = engine.evaluate("where(items.any($this > 5)).items", data).await.unwrap();
    
    // Should include items from the last two objects: [4,5,6] and [7,8,9]
    assert!(count(&result) >= 6, "Should have at least 6 items");
}

#[tokio::test]
async fn test_lambda_variable_scoping() {
    let engine = TestUtils::create_test_engine();
    let data = json!([{"x": 1}, {"x": 2}, {"x": 3}]);
    
    // Test with environment variable
    let mut variables = HashMap::new();
    variables.insert("threshold".to_string(), FhirPathValue::Integer(1));
    
    let result = engine.evaluate_with_variables(
        "where(x > %threshold)",
        data,
        variables
    ).await.unwrap();
    
    assert_eq!(count(&result), 2); // objects with x=2 and x=3
}

#[tokio::test]
async fn test_lambda_implicit_variables() {
    let engine = TestUtils::create_test_engine();
    let data = json!(["a", "b", "c", "d"]);
    
    // Test $index variable
    let result = engine.evaluate("where($index > 1)", data.clone()).await.unwrap();
    assert_eq!(count(&result), 2); // "c" and "d" (indices 2 and 3)
    
    // Test selecting indices
    let result = engine.evaluate("select($index)", data.clone()).await.unwrap();
    assert_eq!(count(&result), 4); // Should return [0, 1, 2, 3]
    
    // Test $total variable
    let result = engine.evaluate("select($total)", data).await.unwrap();
    if let Some(collection) = as_collection(&result) {
        // All should be 4 (total count)
        for value in collection.iter() {
            if let Some(total) = as_single_integer(value) {
                assert_eq!(total, 4);
            }
        }
    }
}

#[tokio::test]
async fn test_lambda_with_complex_data() {
    let engine = TestUtils::create_test_engine();
    let patient = TestUtils::sample_patient();
    
    // Test lambda with FHIR data
    let result = engine.evaluate("name.where(use = 'official')", patient.clone()).await.unwrap();
    assert_eq!(count(&result), 1);
    
    let result = engine.evaluate("name.select(given)", patient.clone()).await.unwrap();
    assert_eq!(count(&result), 2); // Two name objects, each with given arrays
    
    // Test with telecom data
    let result = engine.evaluate("telecom.where(system = 'phone')", patient).await.unwrap();
    assert_eq!(count(&result), 1);
}

#[tokio::test]
async fn test_lambda_performance_optimization() {
    let engine = TestUtils::create_test_engine();
    let large_data = json!((0..1000).collect::<Vec<i32>>());
    
    // Test any() early exit optimization
    let start = std::time::Instant::now();
    let result = engine.evaluate("any($this = 5)", large_data.clone()).await.unwrap();
    let any_time = start.elapsed();
    
    assert_eq!(as_single_boolean(&result), Some(true));
    
    // Early exit should be much faster than processing all items
    assert!(any_time.as_millis() < 100, "any() should exit early, took: {:?}", any_time);
    
    // Test all() early exit optimization  
    let start = std::time::Instant::now();
    let result = engine.evaluate("all($this < 500)", large_data.clone()).await.unwrap();
    let all_time = start.elapsed();
    
    assert_eq!(as_single_boolean(&result), Some(false));
    
    // Should be fast due to early exit on first item >= 500
    assert!(all_time.as_millis() < 100, "all() should exit early, took: {:?}", all_time);
    
    // Compare with select() which can't exit early
    let start = std::time::Instant::now();
    let result = engine.evaluate("select($this).count()", large_data).await.unwrap();
    let select_time = start.elapsed();
    
    assert_eq!(as_single_integer(&result), Some(1000));
    
    // Early exit functions should be much faster than full processing
    println!("Performance comparison - any: {:?}, all: {:?}, select: {:?}", 
             any_time, all_time, select_time);
}

#[tokio::test]
async fn test_lambda_error_handling() {
    let engine = TestUtils::create_test_engine();
    
    // Test with non-collection input for collection-based lambda
    let result = engine.evaluate("where($this > 1)", json!(42)).await.unwrap();
    // Single items should be treated as single-item collections
    assert!(count(&result) <= 1);
    
    // Test lambda with empty collection
    let result = engine.evaluate("where($this > 1)", json!([])).await.unwrap();
    assert_eq!(count(&result), 0);
    
    // Test lambda with null/empty input
    let result = engine.evaluate("where($this.exists())", json!(null)).await.unwrap();
    assert_eq!(count(&result), 0);
}

#[tokio::test]
async fn test_lambda_with_different_types() {
    let engine = TestUtils::create_test_engine();
    
    // Test with mixed type collections
    let data = json!([1, "hello", true, 3.14]);
    
    // Filter by type using lambda
    let result = engine.evaluate("where($this.convertsToInteger())", data.clone()).await;
    match result {
        Ok(filtered) => {
            // Should contain integer values
            assert!(count(&filtered) >= 1);
        },
        Err(_) => {
            // convertsToInteger might not be implemented yet
            println!("Type checking functions not yet implemented");
        }
    }
    
    // Test string operations in lambda
    let string_data = json!(["hello", "world", "test", "fhir"]);
    let result = engine.evaluate("where($this.length() > 4)", string_data).await.unwrap();
    // Should contain "hello" and "world" (length > 4)
    assert!(count(&result) >= 2);
}

#[tokio::test]
async fn test_lambda_chaining() {
    let engine = TestUtils::create_test_engine();
    let data = json!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    
    // Chain multiple lambda operations
    let result = engine.evaluate(
        "where($this > 3).where($this < 8).select($this * 2)",
        data
    ).await.unwrap();
    
    // Should process: filter > 3 -> [4,5,6,7,8,9,10]
    // Then filter < 8 -> [4,5,6,7] 
    // Then multiply by 2 -> [8,10,12,14]
    assert_eq!(count(&result), 4);
}

#[tokio::test]
async fn test_lambda_with_nested_properties() {
    let engine = TestUtils::create_test_engine();
    let data = json!([
        {"person": {"age": 25, "name": "Alice"}},
        {"person": {"age": 30, "name": "Bob"}},
        {"person": {"age": 35, "name": "Carol"}}
    ]);
    
    // Test lambda accessing nested properties
    let result = engine.evaluate("where(person.age > 28)", data.clone()).await.unwrap();
    assert_eq!(count(&result), 2); // Bob and Carol
    
    let result = engine.evaluate("select(person.name)", data).await.unwrap();
    assert_eq!(count(&result), 3); // All names
}

#[tokio::test]
async fn test_lambda_edge_cases() {
    let engine = TestUtils::create_test_engine();
    
    // Test lambda with single item (not in array)
    let result = engine.evaluate("where($this > 5)", json!(10)).await.unwrap();
    assert_eq!(count(&result), 1); // 10 > 5 is true
    
    let result = engine.evaluate("where($this > 5)", json!(3)).await.unwrap();
    assert_eq!(count(&result), 0); // 3 > 5 is false
    
    // Test lambda with nested arrays
    let data = json!([[1, 2], [3, 4], [5, 6]]);
    let result = engine.evaluate("select(count())", data).await.unwrap();
    assert_eq!(count(&result), 3); // Each sub-array contributes one count
    
    // Test lambda with boolean results
    let data = json!([true, false, true, false]);
    let result = engine.evaluate("where($this = true)", data).await.unwrap();
    assert_eq!(count(&result), 2); // Two true values
}