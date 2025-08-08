//! Bundle Operation Baseline Benchmarks
//!
//! Focused benchmarks for the specific operations we want to optimize in Phase 0
//!
//! These benchmarks can run with or without test fixture files.
//! If fixtures are missing, synthetic test data will be generated.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use octofhir_fhirpath::engine::FhirPathEngine;
use octofhir_fhirpath::evaluator::bundle_arc::{ArcBundle, BundleView};
use serde_json::{Value, json};
use std::fs;
use std::hint::black_box;
use std::sync::Arc;

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
    // Try to load from files first, fallback to generated data if files don't exist
    let small = fs::read_to_string("benches/fixtures/small.json")
        .and_then(|s| {
            serde_json::from_str(&s)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
        .unwrap_or_else(|_| generate_test_bundle(10));

    let medium = fs::read_to_string("benches/fixtures/medium.json")
        .and_then(|s| {
            serde_json::from_str(&s)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
        .unwrap_or_else(|_| generate_test_bundle(100));

    let large = fs::read_to_string("benches/fixtures/large.json")
        .and_then(|s| {
            serde_json::from_str(&s)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
        .unwrap_or_else(|_| generate_test_bundle(1000));

    (small, medium, large)
}

fn generate_test_bundle(num_entries: usize) -> Value {
    let entries: Vec<Value> = (0..num_entries)
        .map(|i| {
            json!({
                "fullUrl": format!("http://example.org/Patient/{}", i),
                "resource": {
                    "resourceType": if i % 3 == 0 { "Patient" } else if i % 3 == 1 { "Observation" } else { "Encounter" },
                    "id": i.to_string(),
                    "meta": {
                        "versionId": "1",
                        "lastUpdated": "2024-01-01T00:00:00Z"
                    },
                    "name": [
                        {
                            "family": format!("Family{}", i),
                            "given": [format!("Given{}", i)]
                        }
                    ],
                    "identifier": [
                        {
                            "system": "http://example.org/ids",
                            "value": format!("ID{}", i)
                        }
                    ]
                },
                "search": {
                    "mode": "match",
                    "score": 1.0
                }
            })
        })
        .collect();

    json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": num_entries,
        "timestamp": "2024-01-01T00:00:00Z",
        "entry": entries
    })
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

fn bench_arc_bundle_operations(c: &mut Criterion) {
    let (small, medium, large) = load_test_data();
    let datasets = [("small", &small), ("medium", &medium), ("large", &large)];

    let mut group = c.benchmark_group("arc_bundle_operations");
    group.sample_size(20);

    for (dataset_name, dataset) in &datasets {
        // Benchmark Arc Bundle creation vs JSON clone
        group.bench_with_input(
            BenchmarkId::new("arc_bundle_creation", dataset_name),
            dataset,
            |b, data| {
                b.iter(|| {
                    let arc_bundle = ArcBundle::from_json(data).unwrap();
                    black_box(arc_bundle);
                });
            },
        );

        // Benchmark Arc Bundle clone (cheap Arc clone) vs JSON clone
        let arc_bundle = Arc::new(ArcBundle::from_json(dataset).unwrap());
        group.bench_with_input(
            BenchmarkId::new("arc_bundle_clone", dataset_name),
            &arc_bundle,
            |b, bundle| {
                b.iter(|| {
                    let cloned = bundle.clone();
                    black_box(cloned);
                });
            },
        );

        // Benchmark resource access by type with Arc Bundle
        group.bench_with_input(
            BenchmarkId::new("arc_get_by_type", dataset_name),
            &arc_bundle,
            |b, bundle| {
                b.iter(|| {
                    let patients = bundle.get_entries_by_type("Patient");
                    black_box(patients);
                });
            },
        );

        // Benchmark traditional JSON filtering by type
        group.bench_with_input(
            BenchmarkId::new("json_filter_by_type", dataset_name),
            dataset,
            |b, data| {
                b.iter(|| {
                    if let Some(entries) = data.get("entry") {
                        if let Some(array) = entries.as_array() {
                            let patients: Vec<&Value> = array
                                .iter()
                                .filter(|e| {
                                    e.get("resource")
                                        .and_then(|r| r.get("resourceType"))
                                        .and_then(|rt| rt.as_str())
                                        == Some("Patient")
                                })
                                .collect();
                            black_box(patients);
                        }
                    }
                });
            },
        );

        // Benchmark materialization with Arc Bundle
        group.bench_with_input(
            BenchmarkId::new("arc_materialize", dataset_name),
            &arc_bundle,
            |b, bundle| {
                b.iter(|| {
                    let resources = bundle.materialize_all_resources();
                    black_box(resources);
                });
            },
        );

        // Benchmark Bundle View creation and iteration
        group.bench_with_input(
            BenchmarkId::new("arc_view_operations", dataset_name),
            &arc_bundle,
            |b, bundle| {
                b.iter(|| {
                    let view = BundleView::from_type_filter(bundle.clone(), "Patient");
                    let count = view.iter().count();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    bundle_baseline_benchmarks,
    bench_bundle_operations_baseline,
    bench_memory_cloning_baseline,
    bench_arc_bundle_operations
);

criterion_main!(bundle_baseline_benchmarks);
