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

//! Performance benchmarks for PrecomputedTypeRegistry

use divan::{Bencher, black_box};
use octofhir_fhirpath_model::precomputed_registry::PrecomputedTypeRegistry;

fn main() {
    divan::main();
}

// Since we can't access build_system_types (private), we'll benchmark the public interface
fn setup_registry() -> PrecomputedTypeRegistry {
    PrecomputedTypeRegistry::new()
}

#[divan::bench]
fn bench_system_type_lookup(bencher: Bencher) {
    let registry = setup_registry();

    bencher.bench_local(|| {
        black_box(registry.get_system_type(black_box("Boolean")));
        black_box(registry.get_system_type(black_box("Integer")));
        black_box(registry.get_system_type(black_box("String")));
        black_box(registry.get_system_type(black_box("Decimal")));
    })
}

#[divan::bench]
fn bench_namespace_lookup(bencher: Bencher) {
    let registry = setup_registry();

    bencher.bench_local(|| {
        black_box(registry.get_namespace(black_box("Boolean")));
        black_box(registry.get_namespace(black_box("Integer")));
        black_box(registry.get_namespace(black_box("String")));
        black_box(registry.get_namespace(black_box("UnknownType")));
    })
}

#[divan::bench]
fn bench_subtype_check(bencher: Bencher) {
    let registry = setup_registry();

    bencher.bench_local(|| {
        black_box(registry.is_subtype_of(black_box("Boolean"), black_box("Boolean")));
        black_box(registry.is_subtype_of(black_box("Integer"), black_box("String")));
        black_box(registry.is_subtype_of(black_box("Patient"), black_box("Resource")));
    })
}

#[divan::bench]
fn bench_statistics_generation(bencher: Bencher) {
    let registry = setup_registry();

    bencher.bench_local(|| {
        black_box(registry.statistics());
    })
}

#[divan::bench]
fn bench_registry_build(bencher: Bencher) {
    bencher.bench_local(|| {
        let registry = PrecomputedTypeRegistry::new();
        black_box(registry);
    })
}

#[divan::bench]
fn bench_mixed_operations(bencher: Bencher) {
    let registry = setup_registry();

    bencher.bench_local(|| {
        // Simulate a typical type operation sequence
        let type_name = black_box("Boolean");
        black_box(registry.get_system_type(type_name));
        black_box(registry.get_namespace(type_name));
        black_box(registry.is_subtype_of(type_name, "Boolean"));
        black_box(registry.get_properties(type_name));
        black_box(registry.get_property(type_name, "value"));
    })
}

#[divan::bench(args = [100, 1000])]
fn bench_bulk_lookups(bencher: Bencher, count: usize) {
    let registry = setup_registry();
    let type_names = [
        "Boolean", "Integer", "String", "Decimal", "Date", "DateTime", "Time", "Quantity",
    ];

    bencher.bench_local(|| {
        for _ in 0..count {
            for type_name in &type_names {
                black_box(registry.get_system_type(black_box(type_name)));
            }
        }
    })
}

#[divan::bench]
fn bench_repeated_lookups(bencher: Bencher) {
    let registry = setup_registry();

    bencher.bench_local(|| {
        for _ in 0..1000 {
            black_box(registry.get_system_type(black_box("Boolean")));
        }
    })
}
