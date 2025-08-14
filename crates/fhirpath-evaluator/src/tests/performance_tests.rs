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

//! Performance benchmarking tests for the unified FHIRPath engine

use super::{TestUtils, as_single_integer, as_single_boolean, as_single_string};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_performance_vs_existing_engine() {
    let unified_engine = TestUtils::create_test_engine();
    
    // Note: We'll test against the same engine for now since we're replacing the legacy ones
    // In a real scenario, this would test against the previous implementation
    
    let test_cases = vec![
        ("Simple literal", "42", json!({})),
        ("Property access", "name.given", TestUtils::sample_patient()),
        ("Complex navigation", "name.where(use='official').given.first()", TestUtils::sample_patient()),
        ("Math operations", "(1 + 2) * (3 + 4) / 2", json!({})),
        ("Collection operations", "[1,2,3,4,5].where($this > 2).count()", json!({})),
        ("String operations", "'hello world'.upper().contains('HELLO')", json!({})),
        ("Boolean logic", "true and (false or true)", json!({})),
    ];
    
    for (name, expression, data) in test_cases {
        // Benchmark unified engine
        let (unified_time, unified_result) = TestUtils::benchmark_expression(
            &unified_engine, expression, data.clone(), 100
        ).await;
        
        // Verify result is valid
        match unified_result {
            Ok(_) => {
                println!("{}: {:?} (100 iterations)", name, unified_time);
                
                // Performance should be reasonable (< 1s for 100 simple iterations)
                assert!(unified_time < Duration::from_secs(1), 
                       "{}: Too slow: {:?}", name, unified_time);
            },
            Err(e) => panic!("{}: Evaluation failed: {}", name, e),
        }
    }
}

#[tokio::test]
async fn test_lambda_performance_optimizations() {
    let engine = TestUtils::create_test_engine();
    let large_collection = json!((0..10000).collect::<Vec<i32>>());
    
    // Test any() early exit optimization
    let start = std::time::Instant::now();
    let result = engine.evaluate("any($this = 5)", large_collection.clone()).await.unwrap();
    let any_time = start.elapsed();
    
    assert_eq!(as_single_boolean(&result), Some(true));
    assert!(any_time.as_millis() < 100, "any() should exit early, took: {:?}", any_time);
    
    // Test all() early exit optimization  
    let start = std::time::Instant::now();
    let result = engine.evaluate("all($this < 500)", large_collection.clone()).await.unwrap();
    let all_time = start.elapsed();
    
    assert_eq!(as_single_boolean(&result), Some(false));
    assert!(all_time.as_millis() < 100, "all() should exit early, took: {:?}", all_time);
    
    // Compare with operations that must process all items
    let start = std::time::Instant::now();
    let result = engine.evaluate("count()", large_collection.clone()).await.unwrap();
    let count_time = start.elapsed();
    
    assert_eq!(as_single_integer(&result), Some(10000));
    
    let start = std::time::Instant::now();
    let result = engine.evaluate("sum()", large_collection).await.unwrap();
    let sum_time = start.elapsed();
    
    // Sum of 0..9999 = 9999 * 10000 / 2 = 49995000
    assert_eq!(as_single_integer(&result), Some(49995000));
    
    println!("Lambda performance - any: {:?}, all: {:?}, count: {:?}, sum: {:?}", 
             any_time, all_time, count_time, sum_time);
    
    // Early exit functions should be much faster
    assert!(any_time < count_time / 10);
    assert!(all_time < count_time / 10);
}

#[tokio::test]
async fn test_memory_usage_stability() {
    let engine = TestUtils::create_test_engine();
    
    // Test with increasingly large data
    for size in [100, 1000, 5000] {
        let data = json!((0..size).collect::<Vec<i32>>());
        
        // Multiple evaluations to test for memory leaks
        for _ in 0..10 {
            let result = engine.evaluate("count()", data.clone()).await.unwrap();
            assert_eq!(as_single_integer(&result), Some(size as i64));
        }
    }
    
    // Test recursive evaluation memory usage
    let nested_data = json!({
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "level5": {
                            "value": "deep"
                        }
                    }
                }
            }
        }
    });
    
    for _ in 0..100 {
        let result = engine.evaluate("level1.level2.level3.level4.level5.value", nested_data.clone()).await.unwrap();
        assert_eq!(as_single_string(&result), Some("deep".to_string()));
    }
}

#[tokio::test]
async fn test_expression_parsing_performance() {
    let engine = TestUtils::create_test_engine();
    
    // Test parsing performance with complex expressions
    let complex_expressions = vec![
        "Patient.name.where(use='official').given.first() & ' ' & Patient.name.where(use='official').family.first()",
        "(telecom.where(system='phone').value | telecom.where(system='email').value).exists()",
        "address.where(use='home').line.first() & ', ' & address.where(use='home').city & ', ' & address.where(use='home').state",
        "name.given.count() + name.family.count() + telecom.count() + address.count()",
        "birthDate.exists() and name.exists() and (telecom.exists() or address.exists())",
    ];
    
    let patient = TestUtils::sample_patient();
    
    for expression in complex_expressions {
        let start = std::time::Instant::now();
        
        // Parse and evaluate multiple times
        for _ in 0..50 {
            let result = engine.evaluate(expression, patient.clone()).await;
            assert!(result.is_ok(), "Failed to evaluate: {}", expression);
        }
        
        let total_time = start.elapsed();
        let avg_time = total_time / 50;
        
        println!("Complex expression '{}...': avg {:?}", &expression[..50.min(expression.len())], avg_time);
        
        // Each evaluation should be reasonably fast
        assert!(avg_time < Duration::from_millis(10), 
               "Expression too slow: {} took {:?}", expression, avg_time);
    }
}

#[tokio::test]
async fn test_collection_processing_performance() {
    let engine = TestUtils::create_test_engine();
    
    // Test with different collection sizes
    let sizes = vec![100, 1000, 5000];
    
    for size in sizes {
        let data = json!((1..=size).collect::<Vec<i32>>());
        
        // Test various collection operations
        let operations = vec![
            ("count", "count()"),
            ("sum", "sum()"),
            ("first", "first()"),
            ("last", "last()"),
            ("exists", "exists()"),
            ("empty", "empty()"),
        ];
        
        for (op_name, expression) in operations {
            let start = std::time::Instant::now();
            
            let result = engine.evaluate(expression, data.clone()).await.unwrap();
            let operation_time = start.elapsed();
            
            // Verify result makes sense
            match op_name {
                "count" => assert_eq!(as_single_integer(&result), Some(size as i64)),
                "sum" => assert_eq!(as_single_integer(&result), Some((size * (size + 1) / 2) as i64)),
                "first" => assert_eq!(as_single_integer(&result), Some(1)),
                "last" => assert_eq!(as_single_integer(&result), Some(size as i64)),
                "exists" => assert_eq!(as_single_boolean(&result), Some(true)),
                "empty" => assert_eq!(as_single_boolean(&result), Some(false)),
                _ => {}
            }
            
            println!("{} on {} items: {:?}", op_name, size, operation_time);
            
            // Operations should scale reasonably
            assert!(operation_time < Duration::from_millis(100), 
                   "{} on {} items too slow: {:?}", op_name, size, operation_time);
        }
    }
}

#[tokio::test]
async fn test_string_operation_performance() {
    let engine = TestUtils::create_test_engine();
    
    // Test string operations with different string lengths
    let long_string = "very long string ".repeat(100);
    let strings = vec![
        "short",
        "this is a medium length string for testing",
        long_string.as_str(), // Very long string
    ];
    
    for test_string in strings {
        let data = json!(test_string);
        
        let operations = vec![
            ("length", "length()"),
            ("upper", "upper()"),
            ("lower", "lower()"),
            ("contains", "contains('test')"),
            ("startsWith", "startsWith('test')"),
            ("endsWith", "endsWith('test')"),
        ];
        
        for (op_name, expression) in operations {
            let start = std::time::Instant::now();
            
            // Run operation multiple times
            for _ in 0..100 {
                let result = engine.evaluate(expression, data.clone()).await;
                assert!(result.is_ok(), "Failed string operation: {}", op_name);
            }
            
            let total_time = start.elapsed();
            let avg_time = total_time / 100;
            
            println!("{} on {} chars: avg {:?}", op_name, test_string.len(), avg_time);
            
            // String operations should be very fast
            assert!(avg_time < Duration::from_millis(1), 
                   "{} on {} chars too slow: {:?}", op_name, test_string.len(), avg_time);
        }
    }
}

#[tokio::test]
async fn test_arithmetic_performance() {
    let engine = TestUtils::create_test_engine();
    
    // Test arithmetic operations performance
    let arithmetic_ops = vec![
        ("addition", "123 + 456"),
        ("subtraction", "456 - 123"),
        ("multiplication", "123 * 456"),
        ("division", "456 / 123"),
        ("modulo", "456 mod 123"),
        ("complex", "(123 + 456) * (789 - 123) / (456 + 789)"),
    ];
    
    for (op_name, expression) in arithmetic_ops {
        let start = std::time::Instant::now();
        
        // Run arithmetic operation many times
        for _ in 0..1000 {
            let result = engine.evaluate(expression, json!({})).await;
            assert!(result.is_ok(), "Failed arithmetic: {}", op_name);
        }
        
        let total_time = start.elapsed();
        let avg_time = total_time / 1000;
        
        println!("Arithmetic {}: avg {:?}", op_name, avg_time);
        
        // Arithmetic should be very fast
        assert!(avg_time < Duration::from_micros(100), 
               "Arithmetic {} too slow: {:?}", op_name, avg_time);
    }
}

#[tokio::test]
async fn test_nested_property_access_performance() {
    let engine = TestUtils::create_test_engine();
    
    // Create deeply nested data
    let mut nested_data = json!({"value": "bottom"});
    for i in 0..20 {
        nested_data = json!({"level": i, "nested": nested_data});
    }
    
    // Test navigation through nested structure
    let mut path = String::new();
    for _ in 0..20 {
        path.push_str("nested.");
    }
    path.push_str("value");
    
    let start = std::time::Instant::now();
    
    // Access nested property multiple times
    for _ in 0..100 {
        let result = engine.evaluate(&path, nested_data.clone()).await.unwrap();
        assert_eq!(as_single_string(&result), Some("bottom".to_string()));
    }
    
    let total_time = start.elapsed();
    let avg_time = total_time / 100;
    
    println!("Nested property access (20 levels): avg {:?}", avg_time);
    
    // Nested access should be reasonable
    assert!(avg_time < Duration::from_millis(5), 
           "Nested access too slow: {:?}", avg_time);
}

#[tokio::test]
async fn test_concurrent_evaluation_performance() {
    use std::sync::Arc;
    use tokio::task;
    
    let engine = Arc::new(TestUtils::create_test_engine());
    let patient = TestUtils::sample_patient();
    
    let start = std::time::Instant::now();
    
    let mut handles = Vec::new();
    
    // Spawn many concurrent evaluation tasks
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
        
        let handle = task::spawn(async move {
            engine_clone.evaluate(&expr_str, patient_clone).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => println!("Evaluation error: {}", e),
            Err(e) => println!("Task error: {}", e),
        }
    }
    
    let total_time = start.elapsed();
    
    println!("Concurrent evaluation: {} successes in {:?}", success_count, total_time);
    
    assert_eq!(success_count, 100, "Not all concurrent evaluations succeeded");
    
    // Concurrent evaluation should benefit from parallelism
    assert!(total_time < Duration::from_secs(5), 
           "Concurrent evaluation too slow: {:?}", total_time);
}