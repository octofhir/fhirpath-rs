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

use super::{TestUtils, as_single_integer};
use serde_json::json;

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
