//! Core FHIRPath Performance Benchmark
//! Simplified benchmark focusing on the 3 main components: tokenizer, parser, and evaluator

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;
use fhirpath_parser::{parse_expression_pratt, tokenizer::Tokenizer};
use fhirpath_core::engine::FhirPathEngine;
use serde_json::Value;

/// Test expressions of varying complexity
const TEST_EXPRESSIONS: &[(&str, &str)] = &[
    ("simple", "Patient.name"),
    ("medium", "Patient.name.where(use = 'official')"),
    ("complex", "Patient.name.where(use = 'official').given.first()"),
];

/// Create engine for evaluator benchmarks
fn create_engine() -> FhirPathEngine {
    FhirPathEngine::new()
}

/// Benchmark tokenizer performance
fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    group.throughput(Throughput::Elements(1));
    
    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("tokenize", complexity),
            expression,
            |b, expr| {
                b.iter(|| {
                    let mut tokenizer = Tokenizer::new(black_box(expr));
                    black_box(tokenizer.tokenize_all())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark parser performance
fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    group.throughput(Throughput::Elements(1));
    
    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("parse", complexity),
            expression,
            |b, expr| {
                b.iter(|| {
                    black_box(parse_expression_pratt(black_box(expr)))
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark evaluator performance
fn bench_evaluator(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator");
    group.throughput(Throughput::Elements(1));
    
    let input = Value::String("test".to_string());
    
    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("evaluate", complexity),
            expression,
            |b, expr| {
                b.iter(|| {
                    let mut engine = create_engine();
                    black_box(engine.evaluate(black_box(expr), input.clone()))
                })
            }
        );
    }
    
    group.finish();
}

/// Combined benchmark showing full pipeline performance
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");
    group.throughput(Throughput::Elements(1));
    
    let input = Value::String("test".to_string());
    
    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("tokenize_parse_evaluate", complexity),
            expression,
            |b, expr| {
                b.iter(|| {
                    // Full pipeline: tokenize -> parse -> evaluate
                    let mut engine = create_engine();
                    black_box(engine.evaluate(black_box(expr), input.clone()))
                })
            }
        );
    }
    
    group.finish();
}

/// Operations per second benchmark for throughput measurement
fn bench_operations_per_second(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official')";
    let input = Value::String("test".to_string());
    
    let mut group = c.benchmark_group("operations_per_second");
    group.sample_size(10000);
    
    group.bench_function("tokenizer_ops_per_sec", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            black_box(tokenizer.tokenize_all())
        })
    });
    
    group.bench_function("parser_ops_per_sec", |b| {
        b.iter(|| {
            black_box(parse_expression_pratt(black_box(expression)))
        })
    });
    
    group.bench_function("evaluator_ops_per_sec", |b| {
        b.iter(|| {
            let mut engine = create_engine();
            black_box(engine.evaluate(black_box(expression), input.clone()))
        })
    });
    
    group.finish();
}

criterion_group!(
    core_benchmarks,
    bench_tokenizer,
    bench_parser,
    bench_evaluator,
    bench_full_pipeline,
    bench_operations_per_second
);

criterion_main!(core_benchmarks);