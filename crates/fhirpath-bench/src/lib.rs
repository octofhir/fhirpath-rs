use divan::Bencher;
use octofhir_fhirpath_evaluator::FhirPathEngine;
use octofhir_fhirpath_model::FhirSchemaModelProvider;
use octofhir_fhirpath_parser::{Tokenizer, parse_expression};
use octofhir_fhirpath_registry::FhirPathRegistry;
use serde_json::Value;
use std::sync::Arc;

pub mod profiling;

/// Benchmark expressions categorized by complexity
#[derive(Debug, Clone)]
pub struct BenchmarkExpressions {
    pub simple: Vec<&'static str>,
    pub medium: Vec<&'static str>,
    pub complex: Vec<&'static str>,
}

impl Default for BenchmarkExpressions {
    fn default() -> Self {
        Self {
            simple: vec![
                "Patient.active",
                "Patient.name.family",
                "Patient.birthDate",
                "Patient.gender",
                "true",
                "false",
                "1 + 2",
                "Patient.name.count()",
            ],
            medium: vec![
                "Patient.name.where(use = 'official').family",
                "Patient.telecom.where(system = 'phone').value",
                "Patient.extension.where(url = 'http://example.org').value",
                "Patient.contact.name.family",
                "Patient.birthDate > @1980-01-01",
                "Patient.name.family.substring(0, 3)",
                "Patient.telecom.exists(system = 'email')",
                "Patient.identifier.where(system = 'http://example.org/mrn').value",
            ],
            complex: vec![
                // From resolve.json test cases
                "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()",
                "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()",
                // Additional complex expressions
                "Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()",
                "Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100",
                "Bundle.entry.resource.descendants().where($this is Reference).reference",
                "Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value",
            ],
        }
    }
}

/// Sample FHIR data for benchmarking
pub fn get_sample_patient() -> Value {
    serde_json::json!({
        "resourceType": "Patient",
        "id": "example",
        "active": true,
        "name": [
            {
                "use": "official",
                "family": "Doe",
                "given": ["John", "James"]
            },
            {
                "use": "usual",
                "family": "Doe",
                "given": ["Johnny"]
            }
        ],
        "telecom": [
            {
                "system": "phone",
                "value": "+1-555-555-5555",
                "use": "home"
            },
            {
                "system": "email",
                "value": "john.doe@example.com",
                "use": "work"
            }
        ],
        "gender": "male",
        "birthDate": "1974-12-25",
        "address": [
            {
                "use": "home",
                "line": ["123 Main St"],
                "city": "Anytown",
                "state": "NY",
                "postalCode": "12345",
                "country": "US"
            }
        ]
    })
}

pub fn get_sample_bundle() -> Value {
    // Load bundle-medium.json from the specs directory
    let bundle_path = "specs/fhirpath/tests/input/bundle-medium.json";

    match std::fs::read_to_string(bundle_path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse bundle-medium.json: {e}");
                // Fallback to a minimal bundle structure
                serde_json::json!({
                    "resourceType": "Bundle",
                    "id": "fallback-bundle",
                    "type": "collection",
                    "entry": [
                        {
                            "resource": get_sample_patient()
                        }
                    ]
                })
            })
        }
        Err(e) => {
            eprintln!("Failed to read bundle-medium.json: {e}");
            eprintln!(
                "Using fallback bundle. Make sure to run benchmarks from the workspace root."
            );
            // Fallback to a minimal bundle structure
            serde_json::json!({
                "resourceType": "Bundle",
                "id": "fallback-bundle",
                "type": "collection",
                "entry": [
                    {
                        "resource": get_sample_patient()
                    }
                ]
            })
        }
    }
}

/// Benchmark tokenization of FHIRPath expressions
#[divan::bench(args = BenchmarkExpressions::default().simple)]
pub fn bench_tokenize_simple(bencher: Bencher, expression: &str) {
    bencher.bench_local(|| {
        let _ = Tokenizer::new(expression).tokenize_all();
    });
}

#[divan::bench(args = BenchmarkExpressions::default().medium)]
pub fn bench_tokenize_medium(bencher: Bencher, expression: &str) {
    bencher.bench_local(|| {
        let _ = Tokenizer::new(expression).tokenize_all();
    });
}

#[divan::bench(args = BenchmarkExpressions::default().complex)]
pub fn bench_tokenize_complex(bencher: Bencher, expression: &str) {
    bencher.bench_local(|| {
        let _ = Tokenizer::new(expression).tokenize_all();
    });
}

/// Benchmark parsing of FHIRPath expressions
#[divan::bench(args = BenchmarkExpressions::default().simple)]
pub fn bench_parse_simple(bencher: Bencher, expression: &str) {
    bencher.bench_local(|| {
        let _ = parse_expression(expression);
    });
}

#[divan::bench(args = BenchmarkExpressions::default().medium)]
pub fn bench_parse_medium(bencher: Bencher, expression: &str) {
    bencher.bench_local(|| {
        let _ = parse_expression(expression);
    });
}

#[divan::bench(args = BenchmarkExpressions::default().complex)]
pub fn bench_parse_complex(bencher: Bencher, expression: &str) {
    bencher.bench_local(|| {
        let _ = parse_expression(expression);
    });
}

/// Benchmark evaluation of FHIRPath expressions
#[divan::bench(args = BenchmarkExpressions::default().simple)]
pub fn bench_evaluate_simple(bencher: Bencher, expression: &str) {
    let registry = Arc::new(FhirPathRegistry::default());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let model_provider = Arc::new(rt.block_on(async {
        FhirSchemaModelProvider::r5()
            .await
            .expect("Failed to create R5 FHIR Schema Provider")
    }));
    let engine = FhirPathEngine::new(registry, model_provider);
    let data = get_sample_patient();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| {
        runtime.block_on(async {
            let _ = engine.evaluate(expression, data.clone()).await;
        });
    });
}

#[divan::bench(args = BenchmarkExpressions::default().medium)]
pub fn bench_evaluate_medium(bencher: Bencher, expression: &str) {
    let registry = Arc::new(FhirPathRegistry::default());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let model_provider = Arc::new(rt.block_on(async {
        FhirSchemaModelProvider::r5()
            .await
            .expect("Failed to create R5 FHIR Schema Provider")
    }));
    let engine = FhirPathEngine::new(registry, model_provider);
    let data = get_sample_patient();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| {
        runtime.block_on(async {
            let _ = engine.evaluate(expression, data.clone()).await;
        });
    });
}

#[divan::bench(args = BenchmarkExpressions::default().complex)]
pub fn bench_evaluate_complex(bencher: Bencher, expression: &str) {
    let registry = Arc::new(FhirPathRegistry::default());
    let rt = tokio::runtime::Runtime::new().unwrap();
    let model_provider = Arc::new(rt.block_on(async {
        FhirSchemaModelProvider::r5()
            .await
            .expect("Failed to create R5 FHIR Schema Provider")
    }));
    let engine = FhirPathEngine::new(registry, model_provider);
    let data = get_sample_bundle();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    bencher.bench_local(|| {
        runtime.block_on(async {
            let _ = engine.evaluate(expression, data.clone()).await;
        });
    });
}

/// Generate benchmark results summary
pub fn generate_benchmark_summary() -> String {
    format!(
        r#"# FHIRPath-rs Benchmark Results

Generated on: {}

## Overview

This benchmark suite measures the performance of FHIRPath-rs library across three main operations:
- **Tokenization**: Converting FHIRPath expressions into tokens
- **Parsing**: Building AST from tokens
- **Evaluation**: Executing expressions against FHIR data

## Expression Categories

### Simple Expressions
Basic field access and simple operations:
{}

### Medium Expressions  
Filtered queries and basic functions:
{}

### Complex Expressions
Bundle operations and resolve() calls:
{}

## Benchmark Results

Run `cargo bench --package fhirpath-bench` to generate detailed results.

Use `fhirpath-bench profile <expression>` to generate flamegraphs for specific expressions.
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        BenchmarkExpressions::default().simple.join("\n- "),
        BenchmarkExpressions::default().medium.join("\n- "),
        BenchmarkExpressions::default().complex.join("\n- "),
    )
}
