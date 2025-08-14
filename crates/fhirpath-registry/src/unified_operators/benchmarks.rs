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

//! Performance benchmarks for unified operators

#[cfg(all(test, feature = "bench"))]
mod benches {
    use super::super::*;
    use crate::unified_operator_registry::create_unified_operator_registry;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};
    use std::time::Instant;

    /// Benchmark arithmetic operations
    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn bench_arithmetic_operations() {
        let registry = create_unified_operator_registry();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let iterations = 100_000;

        println!("Benchmarking arithmetic operations ({} iterations):", iterations);

        // Benchmark addition
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "+",
                FhirPathValue::Integer(1000),
                FhirPathValue::Integer(2000),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Addition: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark multiplication
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "*",
                FhirPathValue::Integer(123),
                FhirPathValue::Integer(456),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Multiplication: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark division
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "/",
                FhirPathValue::Integer(10000),
                FhirPathValue::Integer(7),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Division: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());
    }

    /// Benchmark comparison operations
    #[tokio::test]
    #[ignore]
    async fn bench_comparison_operations() {
        let registry = create_unified_operator_registry();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let iterations = 100_000;

        println!("Benchmarking comparison operations ({} iterations):", iterations);

        // Benchmark equality
        let start = Instant::now();
        for i in 0..iterations {
            let _ = registry.evaluate_binary(
                "=",
                FhirPathValue::Integer(i as i64 % 1000),
                FhirPathValue::Integer(500),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Equality: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark less than
        let start = Instant::now();
        for i in 0..iterations {
            let _ = registry.evaluate_binary(
                "<",
                FhirPathValue::Integer(i as i64 % 1000),
                FhirPathValue::Integer(500),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Less than: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark string equivalence
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "~",
                FhirPathValue::String("Hello World".into()),
                FhirPathValue::String("hello world".into()),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("String equivalence: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());
    }

    /// Benchmark collection operations
    #[tokio::test]
    #[ignore]
    async fn bench_collection_operations() {
        let registry = create_unified_operator_registry();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let iterations = 10_000; // Fewer iterations for more expensive operations

        println!("Benchmarking collection operations ({} iterations):", iterations);

        // Create test collections
        let small_collection = FhirPathValue::Collection(Collection::from_vec(
            (1..=10).map(|i| FhirPathValue::Integer(i)).collect()
        ));
        let large_collection = FhirPathValue::Collection(Collection::from_vec(
            (1..=100).map(|i| FhirPathValue::Integer(i)).collect()
        ));

        // Benchmark union
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "|",
                small_collection.clone(),
                large_collection.clone(),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Union: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark membership test
        let start = Instant::now();
        for i in 0..iterations {
            let _ = registry.evaluate_binary(
                "in",
                FhirPathValue::Integer((i % 150) as i64),
                large_collection.clone(),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Membership (in): {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark contains
        let start = Instant::now();
        for i in 0..iterations {
            let _ = registry.evaluate_binary(
                "contains",
                large_collection.clone(),
                FhirPathValue::Integer((i % 150) as i64),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Contains: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());
    }

    /// Benchmark type operations
    #[tokio::test]
    #[ignore]
    async fn bench_type_operations() {
        let registry = create_unified_operator_registry();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let iterations = 100_000;

        println!("Benchmarking type operations ({} iterations):", iterations);

        // Benchmark type checking
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "is",
                FhirPathValue::Integer(42),
                FhirPathValue::String("Integer".into()),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Type checking (is): {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());

        // Benchmark type casting
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = registry.evaluate_binary(
                "as",
                FhirPathValue::Integer(42),
                FhirPathValue::String("Decimal".into()),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("Type casting (as): {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());
    }

    /// Benchmark string operations
    #[tokio::test]
    #[ignore]
    async fn bench_string_operations() {
        let registry = create_unified_operator_registry();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let iterations = 100_000;

        println!("Benchmarking string operations ({} iterations):", iterations);

        // Benchmark string concatenation
        let start = Instant::now();
        for i in 0..iterations {
            let _ = registry.evaluate_binary(
                "&",
                FhirPathValue::String(format!("String {}", i % 1000).into()),
                FhirPathValue::String(" suffix".into()),
                &context,
            ).await.unwrap();
        }
        let duration = start.elapsed();
        println!("String concatenation: {:?} ({:.0} ops/sec)", 
            duration, iterations as f64 / duration.as_secs_f64());
    }

    /// Memory usage benchmark
    #[tokio::test]
    #[ignore]
    async fn bench_memory_usage() {
        let registry = create_unified_operator_registry();
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        println!("Memory usage analysis:");
        
        // Create large collections for memory testing
        let large_collection = FhirPathValue::Collection(Collection::from_vec(
            (1..=10000).map(|i| FhirPathValue::Integer(i)).collect()
        ));

        println!("Created collection with 10,000 integers");
        
        // Test union memory usage
        let start = Instant::now();
        let result = registry.evaluate_binary(
            "|",
            large_collection.clone(),
            large_collection.clone(),
            &context,
        ).await.unwrap();
        let duration = start.elapsed();
        
        if let FhirPathValue::Collection(items) = result {
            println!("Union result size: {} items, took {:?}", items.len(), duration);
        }

        println!("Memory benchmark completed");
    }
}