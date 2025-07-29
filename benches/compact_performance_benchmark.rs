//! Compact FHIRPath Performance Benchmark
//!
//! This benchmark provides clear, focused performance measurements for:
//! - Tokenizer performance (operations/second)
//! - Parser performance (operations/second)
//! - Direct comparison between tokenizer and parser
//!
//! Results show concrete numbers for performance optimization decisions.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fhirpath_parser::{parse_expression, tokenizer::Tokenizer};
use std::hint::black_box;

/// Test expressions representing different complexity levels
const TEST_EXPRESSIONS: &[(&str, &str)] = &[
    ("simple", "Patient.name"),
    ("medium", "Patient.name.where(use = 'official')"),
    (
        "complex",
        "Patient.name.where(use = 'official').given.first()",
    ),
    (
        "very_complex",
        "Bundle.entry.resource.where($this is Patient).name.where(use = 'official').given",
    ),
];

/// Tokenizer performance benchmark
fn benchmark_tokenizer_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_performance");
    group.throughput(Throughput::Elements(1));

    for (name, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(BenchmarkId::new("tokenize", name), expression, |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(*expr));
                let mut token_count = 0;
                while let Ok(Some(_)) = tokenizer.next_token() {
                    token_count += 1;
                }
                black_box(token_count)
            })
        });
    }
    group.finish();
}

/// Parser performance benchmark
fn benchmark_parser_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_performance");
    group.throughput(Throughput::Elements(1));

    for (name, expression) in TEST_EXPRESSIONS {
        group.bench_with_input(BenchmarkId::new("parse", name), expression, |b, expr| {
            b.iter(|| black_box(parse_expression(black_box(*expr))))
        });
    }
    group.finish();
}

/// Direct comparison between tokenizer and parser
fn benchmark_tokenizer_vs_parser(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official')";

    let mut group = c.benchmark_group("comparison");
    group.throughput(Throughput::Elements(1));

    group.bench_function("tokenizer_ops", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            let mut token_count = 0;
            while let Ok(Some(_)) = tokenizer.next_token() {
                token_count += 1;
            }
            black_box(token_count)
        })
    });

    group.bench_function("parser_ops", |b| {
        b.iter(|| black_box(parse_expression(black_box(expression))))
    });

    group.finish();
}

/// High-throughput benchmark to measure raw performance
fn benchmark_throughput_targets(c: &mut Criterion) {
    let expression = "Patient.name.where(use = 'official')";

    let mut group = c.benchmark_group("throughput_targets");
    group.sample_size(1000);
    group.throughput(Throughput::Elements(1));

    // Target: 10M+ tokenizations per second
    group.bench_function("tokenizer_10M_target", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expression));
            let mut token_count = 0;
            while let Ok(Some(_)) = tokenizer.next_token() {
                token_count += 1;
            }
            black_box(token_count)
        })
    });

    // Target: 1M+ parses per second
    group.bench_function("parser_1M_target", |b| {
        b.iter(|| black_box(parse_expression(black_box(expression))))
    });

    group.finish();
}

/// Performance summary benchmark with results display
fn benchmark_performance_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_summary");
    group.throughput(Throughput::Elements(1));

    // Standard test expression
    let expr = "Patient.name.where(use = 'official').given";

    group.bench_function("summary_tokenizer", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new(black_box(expr));
            let mut token_count = 0;
            while let Ok(Some(_)) = tokenizer.next_token() {
                token_count += 1;
            }
            black_box(token_count)
        })
    });

    group.bench_function("summary_parser", |b| {
        b.iter(|| black_box(parse_expression(black_box(expr))))
    });

    // Test Pratt parser specifically
    group.bench_function("summary_pratt_parser", |b| {
        b.iter(|| black_box(parse_expression(black_box(expr))))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_tokenizer_performance,
    benchmark_parser_performance,
    benchmark_tokenizer_vs_parser,
    benchmark_throughput_targets,
    benchmark_performance_summary
);
criterion_main!(benches);
