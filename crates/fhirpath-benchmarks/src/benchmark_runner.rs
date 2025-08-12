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

//! Benchmark suite runner

use criterion::{BenchmarkId, Criterion, Throughput};
use octofhir_fhirpath::model::MockModelProvider;
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue, parse};
use octofhir_fhirpath_parser::Tokenizer;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

/// Benchmark suite for FHIRPath performance testing
pub struct BenchmarkSuite {
    engine: FhirPathEngine,
    criterion: Criterion,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        let provider = Arc::new(MockModelProvider::empty());
        let engine = FhirPathEngine::new(provider);
        let criterion = Criterion::default()
            .measurement_time(Duration::from_secs(10))
            .sample_size(100);

        Self { engine, criterion }
    }

    /// Run all benchmarks
    pub fn run_all(&mut self) {
        self.run_parsing_benchmarks();
        self.run_evaluation_benchmarks();
        self.run_compilation_benchmarks();
    }

    /// Run parsing benchmarks
    pub fn run_parsing_benchmarks(&mut self) {
        let expressions = vec![
            "Patient.name",
            "Patient.name.given",
            "Patient.name.given[0]",
            "Patient.identifier.where(system = 'http://example.org')",
            "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()",
        ];

        for expr in expressions {
            // Set throughput for operations per second calculation
            let mut group = self.criterion.benchmark_group("parse");
            group.throughput(Throughput::Elements(1));

            group.bench_with_input(
                BenchmarkId::new("expression", expr),
                &expr,
                |b, expression| {
                    b.iter(|| parse(expression).unwrap());
                },
            );
            group.finish();
        }
    }

    /// Run evaluation benchmarks
    pub fn run_evaluation_benchmarks(&mut self) {
        // Load medium.json fixture
        let medium_json = include_str!("../fixtures/medium.json");
        let medium_data: Value = serde_json::from_str(medium_json).unwrap();

        let expressions = vec![
            ("Patient.name", &medium_data),
            (
                "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()",
                &medium_data,
            ),
        ];

        for (expr, data) in expressions {
            let mut group = self.criterion.benchmark_group("evaluate");
            group.throughput(Throughput::Elements(1));

            group.bench_with_input(
                BenchmarkId::new("expression", expr),
                &(expr, data),
                |b, (expression, resource)| {
                    b.iter_batched(
                        || {
                            let provider = Arc::new(MockModelProvider::empty());
                            let engine = FhirPathEngine::new(provider);
                            (engine, (*resource).clone())
                        },
                        |(engine, data)| {
                            let ast = parse(expression).unwrap();
                            let fhir_value = FhirPathValue::resource_from_json(data);
                            let _ = futures::executor::block_on(engine.evaluate(&ast, fhir_value));
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
            group.finish();
        }
    }

    /// Run compilation benchmarks
    pub fn run_compilation_benchmarks(&mut self) {
        let expressions = vec![
            "Patient.name",
            "Patient.name.given[0]",
            "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()",
        ];

        for expr in expressions {
            let mut group = self.criterion.benchmark_group("tokenize");
            group.throughput(Throughput::Elements(1));

            group.bench_with_input(
                BenchmarkId::new("expression", expr),
                &expr,
                |b, expression| {
                    b.iter(|| {
                        // Tokenization benchmark
                        let mut tokenizer = Tokenizer::new(expression);
                        let mut tokens = Vec::new();
                        while let Ok(Some(token)) = tokenizer.next_token() {
                            tokens.push(token);
                        }
                        tokens
                    });
                },
            );
            group.finish();
        }
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}
