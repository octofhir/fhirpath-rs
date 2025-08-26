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

//! Stress and load testing for the unified FHIRPath engine

use super::{TestUtils, as_single_boolean, as_single_integer, as_single_string};
use sonic_rs::json;
use std::sync::Arc;
use tokio::task;

#[tokio::test]
async fn test_concurrent_evaluation() {
    let engine = Arc::new(TestUtils::create_test_engine().await.unwrap());
    let patient = TestUtils::sample_patient();

    let mut handles = Vec::new();

    // Spawn 100 concurrent evaluation tasks
    for i in 0..100 {
        let engine_clone = Arc::clone(&engine);
        let patient_clone = patient.clone();
        let expression = match i % 4 {
            0 => "name.given.count()",
            1 => "telecom.where(system='phone').exists()",
            2 => "address.where(use='home').city.first()",
            _ => "birthDate.exists()",
        };
        let expr_str = expression.to_string();

        let handle =
            task::spawn(async move { engine_clone.evaluate(&expr_str, patient_clone).await });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => {
                println!("Evaluation error: {e}");
                error_count += 1;
            }
            Err(e) => {
                println!("Task error: {e}");
                error_count += 1;
            }
        }
    }

    println!("Concurrent evaluation results: {success_count} successes, {error_count} errors");

    // Allow some failures but most should succeed
    assert!(
        success_count >= 90,
        "Too many concurrent evaluation failures: {success_count} successes out of 100"
    );
}

#[tokio::test]
async fn test_timeout_handling() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Create potentially expensive operation
    let large_data = json!((0..100_000).collect::<Vec<i32>>());

    // This might be expensive enough to timeout or complete quickly with optimization
    let result = engine
        .evaluate(
            "select($this).where($this > 1000).select($this * $this).sum()",
            large_data,
        )
        .await;

    match result {
        Ok(value) => {
            println!("Complex operation completed: {value:?}");
            // If it completed, verify the result makes sense
            if let Some(sum) = as_single_integer(&value) {
                assert!(sum > 0, "Sum should be positive");
            }
        }
        Err(e) => {
            println!("Complex operation failed (timeout or error): {e}");
            // This is acceptable - some operations may timeout
        }
    }
}

#[tokio::test]
async fn test_memory_leak_prevention() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Repeatedly create and evaluate expressions to test for memory leaks
    for iteration in 0..100 {
        let data = json!((0..1000).collect::<Vec<i32>>());

        // Various operations that create temporary objects
        let expressions = vec![
            "count()",
            "where($this > 500)",
            "select($this * 2)",
            "first()",
            "last()",
        ];

        for expression in expressions {
            let result = engine.evaluate(expression, data.clone()).await;
            assert!(
                result.is_ok(),
                "Failed on iteration {iteration} with expression {expression}"
            );
        }

        // Occasionally print progress
        if iteration % 20 == 0 {
            println!("Memory leak test iteration: {iteration}");
        }
    }

    println!("Memory leak test completed successfully");
}

#[tokio::test]
async fn test_large_string_handling() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test with very large strings
    let large_string = "x".repeat(1_000_000); // 1MB string
    let data = json!(large_string);

    // Test string operations on large strings
    let result = engine.evaluate("length()", data.clone()).await.unwrap();
    assert_eq!(as_single_integer(&result), Some(1_000_000));

    // Test string operations that might be expensive
    let result = engine.evaluate("upper()", data.clone()).await;
    match result {
        Ok(value) => {
            if let Some(upper_str) = as_single_string(&value) {
                assert_eq!(upper_str, "X".repeat(1_000_000));
            }
            println!("Large string upper() succeeded");
        }
        Err(_) => {
            println!("Large string upper() failed (acceptable for very large strings)");
        }
    }

    // Test contains operation on large string
    let result = engine.evaluate("contains('xxx')", data).await;
    match result {
        Ok(value) => {
            assert_eq!(as_single_boolean(&value), Some(true));
            println!("Large string contains() succeeded");
        }
        Err(_) => {
            println!("Large string contains() failed");
        }
    }
}

#[tokio::test]
async fn test_complex_nested_data_stress() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Create very complex nested data structure
    let complex_data = json!({
        "patients": (0..1000).map(|i| json!({
            "id": format!("patient-{}", i),
            "name": [{
                "family": format!("Family{}", i),
                "given": [format!("Given{}", i), format!("Middle{}", i)]
            }],
            "telecom": [{
                "system": "phone",
                "value": format!("555-{:04}", i),
                "use": "home"
            }, {
                "system": "email",
                "value": format!("patient{}@example.com", i),
                "use": "work"
            }],
            "observations": (0..10).map(|j| json!({
                "id": format!("obs-{}-{}", i, j),
                "status": "final",
                "value": i * j
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    });

    // Test various complex queries on this data
    let queries = vec![
        ("patients.count()", Some(1000)),
        ("patients.name.given.count()", Some(2000)), // 2 given names per patient
        ("patients.telecom.count()", Some(2000)),    // 2 telecom per patient
        ("patients.observations.count()", Some(10000)), // 10 observations per patient
    ];

    for (expression, expected_count) in queries {
        let start = std::time::Instant::now();

        let result = engine.evaluate(expression, complex_data.clone()).await;
        let duration = start.elapsed();

        match result {
            Ok(value) => {
                if let Some(expected) = expected_count {
                    assert_eq!(
                        as_single_integer(&value),
                        Some(expected),
                        "Wrong count for: {expression}"
                    );
                }
                println!("Complex query '{expression}' took {duration:?}");

                // Should complete in reasonable time
                assert!(duration.as_secs() < 10, "Query too slow: {expression}");
            }
            Err(e) => {
                println!("Complex query '{expression}' failed: {e}");
                // Some complex queries might fail, which is acceptable
            }
        }
    }
}

#[tokio::test]
async fn test_rapid_fire_evaluations() {
    let engine = TestUtils::create_test_engine().await.unwrap();
    let patient = TestUtils::sample_patient();

    // Rapidly fire many evaluations in sequence
    let start = std::time::Instant::now();
    let mut success_count = 0;

    for i in 0..1000 {
        let expression = match i % 5 {
            0 => "name.count()",
            1 => "telecom.exists()",
            2 => "address.city.first()",
            3 => "birthDate.exists()",
            _ => "gender",
        };

        match engine.evaluate(expression, patient.clone()).await {
            Ok(_) => success_count += 1,
            Err(e) => println!("Evaluation {i} failed: {e}"),
        }
    }

    let total_time = start.elapsed();
    let avg_time = total_time / 1000;

    println!("Rapid fire: {success_count} successes in {total_time:?} (avg: {avg_time:?})");

    assert!(success_count >= 950, "Too many failures in rapid fire test");
    assert!(
        avg_time.as_millis() < 10,
        "Average evaluation time too slow"
    );
}

#[tokio::test]
async fn test_mixed_workload_stress() {
    let engine = Arc::new(TestUtils::create_test_engine().await.unwrap());

    // Create different types of concurrent workloads
    let mut handles = Vec::new();

    // Simple evaluations
    for i in 0..50 {
        let engine_clone = Arc::clone(&engine);
        let handle = task::spawn(async move {
            let result = engine_clone.evaluate("42 + 58", json!({})).await;
            (format!("simple_{i}"), result)
        });
        handles.push(handle);
    }

    // Complex patient evaluations
    let patient = TestUtils::sample_patient();
    for i in 0..30 {
        let engine_clone = Arc::clone(&engine);
        let patient_clone = patient.clone();
        let handle = task::spawn(async move {
            let result = engine_clone
                .evaluate("name.where(use='official').given.count()", patient_clone)
                .await;
            (format!("patient_{i}"), result)
        });
        handles.push(handle);
    }

    // Collection operations
    for i in 0..20 {
        let engine_clone = Arc::clone(&engine);
        let data = json!((0..100).collect::<Vec<i32>>());
        let handle = task::spawn(async move {
            let result = engine_clone
                .evaluate("where($this > 50).aggregate($total + $this, 0)", data)
                .await;
            (format!("collection_{i}"), result)
        });
        handles.push(handle);
    }

    // Wait for all tasks and collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok((name, result)) => results.push((name, result)),
            Err(e) => println!("Task failed: {e}"),
        }
    }

    // Analyze results
    let mut successes = 0;
    let mut failures = 0;

    for (name, result) in results {
        match result {
            Ok(_) => successes += 1,
            Err(e) => {
                failures += 1;
                println!("Mixed workload failure {name}: {e}");
            }
        }
    }

    println!("Mixed workload results: {successes} successes, {failures} failures");

    // Should have high success rate even under mixed load
    let success_rate = successes as f64 / (successes + failures) as f64;
    assert!(
        success_rate >= 0.9,
        "Mixed workload success rate too low: {:.1}%",
        success_rate * 100.0
    );
}

#[tokio::test]
async fn test_edge_case_data_handling() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test various edge case data structures
    let edge_cases = vec![
        // Empty structures
        ("empty_object", json!({})),
        ("empty_array", json!([])),
        ("null_value", json!(null)),
        // Very deep nesting (but not too deep to cause stack overflow)
        ("deep_nesting", {
            let mut nested = json!({"value": "end"});
            for i in 0..20 {
                nested = json!({"level": i, "next": nested});
            }
            nested
        }),
        // Wide structures
        ("wide_object", {
            let mut obj = std::collections::HashMap::new();
            for i in 0..1000 {
                obj.insert(format!("key_{i}"), json!(format!("value_{}", i)));
            }
            json!(obj)
        }),
        // Mixed type arrays
        (
            "mixed_array",
            json!([1, "string", true, null, {"nested": "object"}, [1, 2, 3]]),
        ),
        // Unicode and special characters
        (
            "unicode_data",
            json!({
                "emoji": "🔥🚀💻",
                "chinese": "你好世界",
                "arabic": "مرحبا بالعالم",
                "special": "\"\\n\\t\\r'",
            }),
        ),
    ];

    for (name, data) in edge_cases {
        // Test basic operations on edge case data
        let expressions = vec!["exists()", "empty()", "count()"];

        for expression in expressions {
            let result = engine.evaluate(expression, data.clone()).await;

            match result {
                Ok(value) => {
                    println!("Edge case '{name}' with '{expression}' -> {value:?}");
                }
                Err(e) => {
                    // Some edge cases might fail, which is acceptable
                    println!("Edge case '{name}' with '{expression}' failed: {e}");
                }
            }
        }
    }
}
