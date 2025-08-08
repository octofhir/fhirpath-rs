//! FHIRPath Performance Benchmarks
//!
//! Comprehensive benchmark suite covering tokenizer, parser, and evaluator performance
//! across a wide range of expression complexity levels to demonstrate arena integration benefits.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use octofhir_fhirpath::engine::FhirPathEngine;
use octofhir_fhirpath::evaluator::{
    ContextInheritance, FunctionClosureOptimizer, SharedContextBuilder, SharedEvaluationContext,
};
use octofhir_fhirpath::model::{Collection, FhirPathValue, string_intern::StringInterner};
use octofhir_fhirpath::parser::{parse_expression_pratt, tokenizer::Tokenizer};
use octofhir_fhirpath::pipeline::{AsyncPool, FhirPathPools, PoolConfig, global_pools};
use octofhir_fhirpath::registry::{FunctionRegistry, OperatorRegistry};
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::hint::black_box;
use std::sync::Arc;

/// Fast benchmark expressions - reduced set for quicker results
const TEST_EXPRESSIONS: &[(&str, &str)] = &[
    // Simple expressions for fast benchmarking
    ("simple_literal", "42"),
    ("simple_property", "Patient.name"),
    ("medium_where", "Patient.name.where(use = 'official')"),
    ("medium_function", "Patient.name.first()"),
    // One complex example
    (
        "complex_chained",
        "Patient.name.where(use = 'official').given.first()",
    ),
    // One extremely complex example for performance testing
    (
        "extremely_complex_multi_lambda",
        "Bundle.entry.resource.where($this is Patient).select(name.where(use = 'official').select(given.where(length() > 1).select($this.upper() + ', ' + %context.family.first().lower())))",
    ),
];

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    group.throughput(Throughput::Elements(1));
    group.measurement_time(std::time::Duration::from_secs(5)); // Fast benchmarking

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
    group.measurement_time(std::time::Duration::from_secs(5)); // Fast benchmarking

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
    group.measurement_time(std::time::Duration::from_secs(5)); // Fast benchmarking

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
    group.sample_size(50); // Reduced from 1000
    group.measurement_time(std::time::Duration::from_secs(5)); // Fast benchmarking

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

fn bench_string_interning_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning");
    group.sample_size(20); // Reduced from 100

    group.bench_function("string_interning_baseline", |b| {
        b.iter(|| {
            let interner = StringInterner::new();
            let mut interned_strings = Vec::new();

            // Create many strings to test interning performance
            for i in 0..100 {
                let s = format!("test_string_{i}");
                let interned = interner.intern(&s);
                interned_strings.push(interned);
            }

            // Test hit rate by re-interning the same strings
            for i in 0..100 {
                let s = format!("test_string_{i}");
                black_box(interner.intern(&s));
            }

            let stats = interner.stats();
            black_box(stats)
        })
    });

    group.bench_function("string_interning_hit_rate", |b| {
        b.iter(|| {
            let interner = StringInterner::new();

            // Create base set of strings
            let base_strings: Vec<String> = (0..100).map(|i| format!("base_string_{i}")).collect();

            // Intern base strings multiple times to test hit rate
            for _ in 0..10 {
                for s in &base_strings {
                    black_box(interner.intern(s));
                }
            }

            let stats = interner.stats();
            black_box(stats)
        })
    });

    group.finish();
}

/// Benchmark Arc-backed string interning in tokenizer
fn bench_tokenizer_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_interning");

    // Test with and without interning on common FHIR expressions
    let common_fhir_expressions = [
        "Patient.name.given.first()",
        "Patient.name.family.first()",
        "Bundle.entry.resource.name.given",
        "Patient.telecom.where(system = 'phone')",
        "Observation.value.where(code = 'vital-signs')",
        "Patient.identifier.where(system = 'official')",
        "Bundle.entry.resource.ofType(Patient).name",
        "Patient.address.line.first() + ', ' + Patient.address.city",
    ];

    // Benchmark regular tokenization
    group.bench_function("without_interning", |b| {
        b.iter(|| {
            for expr in &common_fhir_expressions {
                let mut tokenizer = Tokenizer::with_interning(black_box(expr), false);
                black_box(tokenizer.tokenize_all()).unwrap();
            }
        })
    });

    // Benchmark with interning enabled
    group.bench_function("with_interning", |b| {
        b.iter(|| {
            for expr in &common_fhir_expressions {
                let mut tokenizer = Tokenizer::with_interning(black_box(expr), true);
                black_box(tokenizer.tokenize_all()).unwrap();
            }
        })
    });

    // Benchmark interning statistics access
    group.bench_function("interner_stats", |b| {
        b.iter(|| {
            black_box(Tokenizer::interner_stats());
            black_box(Tokenizer::keyword_table_stats());
        })
    });

    group.finish();
}

/// Benchmark streaming tokenizer for large expressions
fn bench_tokenizer_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_streaming");

    // Create large expressions of different sizes
    let small_expr = "Patient.name.given.first()".repeat(10);
    let medium_expr = "Patient.name.given.first()".repeat(100);
    let large_expr = "Patient.name.given.first()".repeat(1000);

    // Benchmark regular tokenization vs streaming for different sizes
    group.bench_with_input(
        BenchmarkId::new("regular_small", small_expr.len()),
        &small_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("streaming_small", small_expr.len()),
        &small_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::with_streaming(black_box(expr), 256);
                let mut stream = tokenizer.tokenize_stream();
                black_box(stream.collect_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("regular_medium", medium_expr.len()),
        &medium_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("streaming_medium", medium_expr.len()),
        &medium_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::with_streaming(black_box(expr), 256);
                let mut stream = tokenizer.tokenize_stream();
                black_box(stream.collect_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("regular_large", large_expr.len()),
        &large_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all())
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("streaming_large", large_expr.len()),
        &large_expr,
        |b, expr| {
            b.iter(|| {
                let mut tokenizer = Tokenizer::with_streaming(black_box(expr), 256);
                let mut stream = tokenizer.tokenize_stream();
                black_box(stream.collect_all())
            })
        },
    );

    // Benchmark memory usage estimation
    group.bench_function("memory_estimation", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new(black_box(&large_expr));
            black_box(tokenizer.estimate_memory_usage())
        })
    });

    group.finish();
}

/// Benchmark shared keyword lookup table performance
fn bench_tokenizer_keywords(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_keywords");

    let keyword_expressions = [
        "true and false",
        "Patient where name exists",
        "Bundle select entry",
        "first() or last()",
        "count() > 0 and distinct().empty()",
        "Patient.name.where(use = 'official' and family.exists()).given.first()",
        "Bundle.entry.resource.ofType(Patient).name.family.first()",
        "true or false and not empty",
    ];

    // Benchmark keyword recognition performance
    group.bench_function("keyword_lookup", |b| {
        b.iter(|| {
            for expr in &keyword_expressions {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                black_box(tokenizer.tokenize_all()).unwrap();
            }
        })
    });

    // Benchmark keyword table statistics
    group.bench_function("keyword_stats", |b| {
        b.iter(|| {
            black_box(Tokenizer::keyword_table_stats());
        })
    });

    // Benchmark is_keyword_str function
    let test_keywords = [
        "true", "false", "and", "or", "where", "select", "first", "last",
    ];
    group.bench_function("is_keyword_str", |b| {
        b.iter(|| {
            for keyword in &test_keywords {
                black_box(Tokenizer::is_keyword_str(black_box(keyword)));
            }
        })
    });

    group.finish();
}

criterion_group!(
    fhirpath_benchmarks,
    bench_tokenizer,
    bench_parser,
    bench_evaluator,
    bench_throughput,
    bench_string_interning_performance,
    bench_tokenizer_interning,
    bench_tokenizer_streaming,
    bench_tokenizer_keywords,
);

criterion_main!(fhirpath_benchmarks);
