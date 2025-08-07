//! FHIRPath Performance Benchmarks
//!
//! Simplified benchmark suite covering tokenizer, parser, and evaluator performance.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use octofhir_fhirpath::engine::FhirPathEngine;
use octofhir_fhirpath::parser::{parse_expression_pratt, tokenizer::Tokenizer};
use serde_json::Value;
use std::hint::black_box;

const TEST_EXPRESSIONS: &[(&str, &str)] = &[
    ("simple", "Patient.name"),
    ("medium", "Patient.name.where(use = 'official')"),
    (
        "complex",
        "Patient.name.where(use = 'official').given.first()",
    ),
];

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
            },
        );
    }

    group.finish();
}

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    group.throughput(Throughput::Elements(1));

    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("parse", complexity),
            expression,
            |b, expr| b.iter(|| black_box(parse_expression_pratt(black_box(expr)))),
        );
    }

    group.finish();
}

fn bench_evaluator(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluator");
    group.throughput(Throughput::Elements(1));

    let input = Value::String("test".to_string());

    for (complexity, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(
            BenchmarkId::new("evaluate", complexity),
            expression,
            |b, expr| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    let mut engine = FhirPathEngine::new();
                    black_box(rt.block_on(engine.evaluate(black_box(expr), input.clone())))
                })
            },
        );
    }

    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official')";
    let input = Value::String("test".to_string());

    let mut group = c.benchmark_group("throughput");
    group.sample_size(1000);

    group.bench_function("tokenizer_throughput", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            black_box(tokenizer.tokenize_all())
        })
    });

    group.bench_function("parser_throughput", |b| {
        b.iter(|| black_box(parse_expression_pratt(black_box(expression))))
    });

    group.bench_function("evaluator_throughput", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            let mut engine = FhirPathEngine::new();
            black_box(rt.block_on(engine.evaluate(black_box(expression), input.clone())))
        })
    });

    group.finish();
}

criterion_group!(
    fhirpath_benchmarks,
    bench_tokenizer,
    bench_parser,
    bench_evaluator,
    bench_throughput
);

criterion_main!(fhirpath_benchmarks);
