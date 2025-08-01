//! Phase 1 Parser Optimizations Benchmark
//!
//! This benchmark suite specifically measures the performance improvements
//! from Phase 1 parser optimizations including:
//! - Direct pattern matching vs discriminant
//! - Const lookup tables for operator precedence
//! - Token interning for frequently used identifiers
//! - SmallVec for argument lists
//! - Enum memory layout optimizations

use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId, Throughput};
use octofhir_fhirpath::parser::{parse, tokenizer::{Tokenizer, InterningTokenizer}};
use std::hint::black_box;

/// Benchmark token interning vs standard tokenization
fn benchmark_token_interning(c: &mut Criterion) {
    let expressions = vec![
        // Repeated identifiers that benefit from interning
        "Patient.name.given",
        "Patient.name.family", 
        "Patient.name.use",
        "Patient.address.line",
        "Patient.address.city",
        "Patient.address.state",
        "Patient.telecom.system",
        "Patient.telecom.value",
        "Patient.telecom.use",
    ];

    let mut group = c.benchmark_group("token_interning");

    // Benchmark standard tokenization
    group.bench_function("standard_tokenizer", |b| {
        b.iter(|| {
            for expr in &expressions {
                let mut tokenizer = Tokenizer::new(black_box(expr));
                while let Ok(Some(_)) = tokenizer.next_token() {
                    // Process all tokens
                }
            }
        })
    });

    // Benchmark with interning tokenizer
    group.bench_function("interning_tokenizer", |b| {
        b.iter(|| {
            for expr in &expressions {
                let mut tokenizer = InterningTokenizer::new(black_box(expr));
                while let Ok(Some(_)) = tokenizer.next_token() {
                    // Process all tokens
                }
            }
        })
    });

    group.finish();
}

/// Benchmark parsing complex expressions with binary operations
/// This tests the const lookup table optimization
fn benchmark_operator_precedence(c: &mut Criterion) {
    let expressions = vec![
        "a + b * c",
        "x / y - z",
        "p and q or r",
        "1 + 2 * 3 - 4 / 5",
        "a = b and c != d or e > f",
        "Patient.active = true and Patient.name.exists()",
        "Observation.value > 100 or Observation.status = 'final'",
        "Bundle.entry.count() >= 1 and Bundle.total > 0",
    ];

    let mut group = c.benchmark_group("operator_precedence");
    group.throughput(Throughput::Elements(expressions.len() as u64));

    group.bench_function("parse_complex_operations", |b| {
        b.iter(|| {
            for expr in &expressions {
                black_box(parse(black_box(expr)));
            }
        })
    });

    group.finish();
}

/// Benchmark parsing expressions with function calls (SmallVec optimization)
fn benchmark_function_arguments(c: &mut Criterion) {
    let expressions = vec![
        // 0 arguments
        "Patient.name.empty()",
        "now()",
        "today()",
        
        // 1 argument
        "Patient.name.where($this.use = 'official')",
        "Bundle.entry.select(resource)",
        "Observation.value.as(Quantity)",
        
        // 2 arguments
        "Patient.name.where(use = 'official')",
        "substring('hello', 1, 3)",
        "Patient.telecom.where(system = 'phone')",
        
        // 3+ arguments
        "Patient.extension.where(url = 'http://example.com' and value.exists())",
        "replace('hello world', 'world', 'universe')",
        "Patient.address.where(use = 'home' and type = 'both')",
        
        // Nested function calls
        "Patient.name.where(given.count() > 1).select(family)",
        "Bundle.entry.where(resource.name.exists()).select(resource.id)",
    ];

    let mut group = c.benchmark_group("function_arguments");
    group.throughput(Throughput::Elements(expressions.len() as u64));

    group.bench_function("parse_function_calls", |b| {
        b.iter(|| {
            for expr in &expressions {
                black_box(parse(black_box(expr)));
            }
        })
    });

    group.finish();
}

/// Benchmark memory-intensive parsing operations
fn benchmark_memory_layout(c: &mut Criterion) {
    // Complex nested expressions that would benefit from boxing optimizations
    let expressions = vec![
        "Patient.name.where(use = 'official').given.where($this.length() > 2)",
        "Bundle.entry.where(resource.resourceType = 'Patient').resource.name.family",
        "Observation.component.where(code.coding.system = 'http://loinc.org').value",
        "Patient.extension.where(url = 'http://hl7.org/fhir/us/core/StructureDefinition/us-core-race').extension.where(url = 'ombCategory').value",
        "Bundle.entry.resource.where(resourceType = 'Patient').name.where(use = 'official').given.first()",
    ];

    let mut group = c.benchmark_group("memory_layout");
    
    for (i, expr) in expressions.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("parse_nested_expr", i),
            expr,
            |b, expr| {
                b.iter(|| black_box(parse(black_box(expr))))
            },
        );
    }

    group.finish();
}

/// Benchmark comprehensive parser performance across different expression types
fn benchmark_comprehensive_parser(c: &mut Criterion) {
    let test_cases = vec![
        ("literals", vec![
            "42",
            "3.14",
            "'hello'",
            "true",
            "@2023-01-01",
            "@2023-01-01T12:00:00",
        ]),
        ("paths", vec![
            "Patient",
            "Patient.name",
            "Patient.name.given",
            "Bundle.entry.resource",
            "Observation.component.value",
        ]),
        ("operations", vec![
            "Patient.active = true",
            "Observation.value > 100",
            "Bundle.entry.count() >= 1",
            "Patient.name.exists() and Patient.active",
        ]),
        ("functions", vec![
            "Patient.name.empty()",
            "Patient.name.first()",
            "Bundle.entry.count()",
            "Patient.name.where(use = 'official')",
        ]),
        ("complex", vec![
            "Patient.name.where(use = 'official').given.first()",
            "Bundle.entry.where(resource.resourceType = 'Patient').resource.name",
            "Observation.component.where(code.coding.exists()).value.as(Quantity)",
        ]),
    ];

    for (category, expressions) in test_cases {
        let mut group = c.benchmark_group(format!("comprehensive_{}", category));
        group.throughput(Throughput::Elements(expressions.len() as u64));

        group.bench_function("parse_batch", |b| {
            b.iter(|| {
                for expr in &expressions {
                    black_box(parse(black_box(expr)));
                }
            })
        });

        group.finish();
    }
}

/// Benchmark parsing performance vs expression complexity
fn benchmark_complexity_scaling(c: &mut Criterion) {
    let base_expressions = vec![
        ("simple", 1, "Patient.name"),
        ("medium", 3, "Patient.name.where(use = 'official').given"),
        ("complex", 5, "Bundle.entry.where(resource.resourceType = 'Patient').resource.name.where(use = 'official').given.first()"),
        ("very_complex", 8, "Patient.extension.where(url = 'http://hl7.org/fhir/us/core/StructureDefinition/us-core-race').extension.where(url = 'ombCategory').value.as(Coding).code"),
    ];

    let mut group = c.benchmark_group("complexity_scaling");

    for (name, complexity, expr) in base_expressions {
        group.bench_with_input(
            BenchmarkId::new("parse_by_complexity", complexity),
            &(name, expr),
            |b, &(_name, expr)| {
                b.iter(|| black_box(parse(black_box(expr))))
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns in parsing
fn benchmark_allocation_patterns(c: &mut Criterion) {
    let expressions = vec![
        // Expressions that stress different allocation patterns
        "Patient.name.given + Patient.name.family",  // String concatenation
        "Bundle.entry.select(resource.name.given)",  // Collection mapping
        "Patient.extension.where(url.contains('race')).value",  // Filtering
        "Observation.component.value.as(Quantity) * 2.54",  // Type casting and arithmetic
    ];

    let mut group = c.benchmark_group("allocation_patterns");
    
    for (i, expr) in expressions.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("parse_allocation_test", i),
            expr,
            |b, expr| {
                b.iter(|| {
                    // Parse multiple times to stress allocation patterns
                    for _ in 0..10 {
                        black_box(parse(black_box(expr)));
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_token_interning,
    benchmark_operator_precedence,
    benchmark_function_arguments,
    benchmark_memory_layout,
    benchmark_comprehensive_parser,
    benchmark_complexity_scaling,
    benchmark_allocation_patterns
);
criterion_main!(benches);