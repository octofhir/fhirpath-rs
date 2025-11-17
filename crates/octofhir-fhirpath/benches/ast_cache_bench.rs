//! Benchmark AST cache performance
//!
//! Measures the impact of the AST compilation cache on expression evaluation.

use divan::{Bencher, black_box};
use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhir_model::FhirPathEvaluator;
use octofhir_fhirpath::core::Collection;
use octofhir_fhirpath::evaluator::context::EvaluationContext;
use octofhir_fhirpath::evaluator::engine::FhirPathEngine;
use octofhir_fhirpath::evaluator::function_registry::create_function_registry;
use std::sync::Arc;

fn main() {
    divan::main();
}

/// Benchmark: First evaluation (cache miss - includes parsing)
#[divan::bench]
fn cache_miss_simple_expression(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            // Create fresh engine for each iteration to ensure cache miss
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let registry = Arc::new(create_function_registry());
                let provider = Arc::new(EmptyModelProvider);
                let engine = FhirPathEngine::new(registry, provider.clone())
                    .await
                    .unwrap();

                let collection = Collection::empty();
                let context = EvaluationContext::new(collection, provider, None, None, None);

                (engine, context)
            })
        })
        .bench_values(|(engine, context)| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async { black_box(engine.evaluate("1 + 2", &context).await.unwrap()) })
        });
}

/// Benchmark: Second evaluation (cache hit - no parsing)
#[divan::bench]
fn cache_hit_simple_expression(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let registry = Arc::new(create_function_registry());
                let provider = Arc::new(EmptyModelProvider);
                let engine = FhirPathEngine::new(registry, provider.clone())
                    .await
                    .unwrap();

                let collection = Collection::empty();
                let context = EvaluationContext::new(collection, provider, None, None, None);

                // Warm up cache with first evaluation
                let _ = engine.evaluate("1 + 2", &context).await.unwrap();

                (engine, context)
            })
        })
        .bench_values(|(engine, context)| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                // This should hit the cache
                black_box(engine.evaluate("1 + 2", &context).await.unwrap())
            })
        });
}

/// Benchmark: Complex expression - cache miss
#[divan::bench]
fn cache_miss_complex_expression(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let registry = Arc::new(create_function_registry());
                let provider = Arc::new(EmptyModelProvider);
                let engine = FhirPathEngine::new(registry, provider.clone())
                    .await
                    .unwrap();

                let collection = Collection::empty();
                let context = EvaluationContext::new(collection, provider, None, None, None);

                (engine, context)
            })
        })
        .bench_values(|(engine, context)| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                black_box(
                    engine
                        .evaluate("Patient.name.where(use = 'official').family", &context)
                        .await
                        .unwrap(),
                )
            })
        });
}

/// Benchmark: Complex expression - cache hit
#[divan::bench]
fn cache_hit_complex_expression(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let registry = Arc::new(create_function_registry());
                let provider = Arc::new(EmptyModelProvider);
                let engine = FhirPathEngine::new(registry, provider.clone())
                    .await
                    .unwrap();

                let collection = Collection::empty();
                let context = EvaluationContext::new(collection, provider, None, None, None);

                // Warm up cache
                let _ = engine
                    .evaluate("Patient.name.where(use = 'official').family", &context)
                    .await
                    .unwrap();

                (engine, context)
            })
        })
        .bench_values(|(engine, context)| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                black_box(
                    engine
                        .evaluate("Patient.name.where(use = 'official').family", &context)
                        .await
                        .unwrap(),
                )
            })
        });
}

/// Benchmark: Multiple different expressions (realistic workload)
#[divan::bench]
fn mixed_workload_with_cache(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let registry = Arc::new(create_function_registry());
                let provider = Arc::new(EmptyModelProvider);
                let engine = FhirPathEngine::new(registry, provider.clone())
                    .await
                    .unwrap();

                let collection = Collection::empty();
                let context = EvaluationContext::new(collection, provider, None, None, None);

                (engine, context)
            })
        })
        .bench_values(|(engine, context)| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                // Simulate realistic workload: mix of repeated and new expressions
                let expressions = [
                    "Patient.name",      // Will be cached after first use
                    "Patient.gender",    // Will be cached
                    "Patient.name",      // Cache hit
                    "Patient.birthDate", // New
                    "Patient.gender",    // Cache hit
                    "Patient.name",      // Cache hit
                ];

                for expr in &expressions {
                    black_box(engine.evaluate(expr, &context).await.unwrap());
                }
            })
        });
}

/// Benchmark: Compile operation (pre-compilation)
#[divan::bench]
fn compile_expression(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let registry = Arc::new(create_function_registry());
                let provider = Arc::new(EmptyModelProvider);
                FhirPathEngine::new(registry, provider).await.unwrap()
            })
        })
        .bench_values(|engine| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime
                .block_on(async { black_box(engine.compile("Patient.name.family").await.unwrap()) })
        });
}
