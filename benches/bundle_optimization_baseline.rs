//! Bundle Operation Baseline Benchmarks
//!
//! Focused benchmarks for the specific operations we want to optimize in Phase 0

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use octofhir_fhirpath::engine::FhirPathEngine;
use serde_json::Value;
use std::fs;
use std::hint::black_box;

/// Bundle-focused expressions that stress the areas we're optimizing
const BUNDLE_EXPRESSIONS: &[(&str, &str)] = &[
    ("simple_bundle_traversal", "Bundle.entry"),
    ("bundle_resource_filter", "Bundle.entry.resource"),
    (
        "bundle_patient_names",
        "Bundle.entry.resource.where($this is Patient).name",
    ),
    (
        "complex_bundle_filter",
        "Bundle.entry.resource.where($this is Patient).name.where(use = 'official').given",
    ),
    (
        "deep_bundle_traversal",
        "Bundle.entry.resource.where($this is Patient).extension.where(url = 'http://example.com').value",
    ),
];

fn load_test_data() -> (Value, Value, Value) {
    let small =
        serde_json::from_str(&fs::read_to_string("benches/fixtures/small.json").unwrap()).unwrap();
    let medium =
        serde_json::from_str(&fs::read_to_string("benches/fixtures/medium.json").unwrap()).unwrap();
    let large =
        serde_json::from_str(&fs::read_to_string("benches/fixtures/large.json").unwrap()).unwrap();
    (small, medium, large)
}

fn bench_bundle_operations_baseline(c: &mut Criterion) {
    let (small, medium, large) = load_test_data();
    let datasets = [("small", &small), ("medium", &medium), ("large", &large)];

    let mut group = c.benchmark_group("bundle_baseline");
    group.sample_size(20); // Reduced sample size for faster execution

    for (dataset_name, dataset) in &datasets {
        for (expr_name, expression) in BUNDLE_EXPRESSIONS {
            group.bench_with_input(
                BenchmarkId::new(format!("{dataset_name}_{expr_name}"), ""),
                &(expression, dataset),
                |b, (expr, data)| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    b.iter(|| {
                        let mut engine = FhirPathEngine::new();
                        black_box(rt.block_on(engine.evaluate(black_box(expr), (**data).clone())))
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_memory_cloning_baseline(c: &mut Criterion) {
    let (small, medium, large) = load_test_data();
    let datasets = [("small", &small), ("medium", &medium), ("large", &large)];

    let mut group = c.benchmark_group("memory_cloning_baseline");
    group.sample_size(20);

    for (dataset_name, dataset) in &datasets {
        group.bench_with_input(
            BenchmarkId::new("json_clone", dataset_name),
            dataset,
            |b, data| b.iter(|| black_box((*data).clone())),
        );

        group.bench_with_input(
            BenchmarkId::new("nested_access", dataset_name),
            dataset,
            |b, data| {
                b.iter(|| {
                    if let Some(entries) = data.get("entry") {
                        if let Some(array) = entries.as_array() {
                            for entry in array.iter().take(10) {
                                // Limit to first 10 for performance
                                black_box(entry.get("resource"));
                            }
                        }
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    bundle_baseline_benchmarks,
    bench_bundle_operations_baseline,
    bench_memory_cloning_baseline
);

criterion_main!(bundle_baseline_benchmarks);
