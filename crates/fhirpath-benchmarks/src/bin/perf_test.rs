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

//! Performance test and profiling binary for FHIRPath expressions

use octofhir_fhirpath::FhirPathEngine;
use octofhir_fhirpath::model::string_intern::global_interner_stats;
use octofhir_fhirpath::model::{Collection, FhirPathValue};
use serde_json::{Value, json};
use std::env;
use std::time::Instant;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Profile specific FHIRPath expression
        let expression = &args[1];
        profile_expression(expression).await;
    } else {
        // Run default performance tests
        println!("🚀 FHIRPath Performance Optimization Test");
        println!("==========================================");

        test_string_interning();
        test_collection_operations();
        test_integration();
        test_fhirpath_evaluation().await;
    }
}

async fn profile_expression(expression: &str) {
    println!("🔬 Profiling FHIRPath Expression: {expression}");
    println!("=====================================");

    // Create sample FHIR Patient resource for testing
    let patient_data = create_sample_patient();
    let engine = FhirPathEngine::with_mock_provider();

    println!("📊 Warming up...");
    // Warmup runs
    for _ in 0..10 {
        let _ = engine.evaluate_str(expression, &patient_data).await;
    }

    println!("🔥 Profiling {} iterations...", 1000);
    let start = Instant::now();

    // Main profiling loop - this is what flamegraph will capture
    for _ in 0..1000 {
        match engine.evaluate_str(expression, &patient_data).await {
            Ok(_result) => {}
            Err(e) => {
                eprintln!("⚠️ Error evaluating expression: {e}");
                break;
            }
        }
    }

    let duration = start.elapsed();
    let per_eval = duration.as_micros() as f64 / 1000.0;

    println!("⏱️ Performance Results:");
    println!("   Total time: {duration:?}");
    println!("   Time per evaluation: {per_eval:.2}μs");
    println!("   Evaluations per second: {:.0}", 1_000_000.0 / per_eval);

    // Show memory stats
    let stats = global_interner_stats();
    println!("📈 Memory Stats:");
    println!("   Interned strings: {}", stats.entries);
}

fn create_sample_patient() -> Value {
    json!({
        "resourceType": "Patient",
        "id": "test-patient",
        "active": true,
        "name": [
            {
                "use": "official",
                "family": "Doe",
                "given": ["John", "Michael"]
            },
            {
                "use": "nickname",
                "given": ["Johnny"]
            }
        ],
        "gender": "male",
        "birthDate": "1980-01-15",
        "address": [
            {
                "use": "home",
                "line": ["123 Main St"],
                "city": "Springfield",
                "state": "IL",
                "postalCode": "62701",
                "country": "US"
            }
        ],
        "telecom": [
            {
                "system": "phone",
                "value": "555-123-4567",
                "use": "home"
            },
            {
                "system": "email",
                "value": "john.doe@example.com",
                "use": "work"
            }
        ],
        "contact": [
            {
                "relationship": [{"coding": [{"code": "emergency"}]}],
                "name": {"family": "Doe", "given": ["Jane"]},
                "telecom": [{"system": "phone", "value": "555-987-6543"}]
            }
        ],
        "extension": [
            {
                "url": "http://example.org/patient-importance",
                "valueCode": "high"
            }
        ]
    })
}

fn test_string_interning() {
    println!("\n📊 String Interning Test");
    println!("-----------------------");

    let start_stats = global_interner_stats();
    println!("Initial interned strings: {}", start_stats.entries);

    // Create some values using interned strings
    for i in 0..100 {
        let _val = FhirPathValue::interned_string(format!("test_{}", i % 10));
    }

    let end_stats = global_interner_stats();
    println!("Final interned strings: {}", end_stats.entries);
    println!(
        "✅ String interning working (created {} unique strings)",
        end_stats.entries - start_stats.entries
    );
}

fn test_collection_operations() {
    println!("\n📊 Collection Performance Test");
    println!("------------------------------");

    let start = Instant::now();

    // Test efficient collection operations
    let mut collections = Vec::new();
    for i in 0..1000 {
        let mut col = Collection::with_capacity(100);
        for j in 0..100 {
            col.push(FhirPathValue::Integer(i * 100 + j));
        }
        collections.push(col);
    }

    // Test efficient extend operations
    let mut master_collection = Collection::new();
    for col in collections {
        master_collection.extend(col);
    }

    let duration = start.elapsed();
    println!(
        "Created and merged {} items in {:?}",
        master_collection.len(),
        duration
    );
    println!(
        "✅ Collection operations: {:.2} items/ms",
        master_collection.len() as f64 / duration.as_millis() as f64
    );
}

fn test_integration() {
    println!("\n📊 Integration Test");
    println!("------------------");

    let start = Instant::now();

    // Create a large collection with mixed types
    let items: Vec<FhirPathValue> = (0..1000)
        .map(|i| match i % 4 {
            0 => FhirPathValue::Integer(i),
            1 => FhirPathValue::Boolean(i % 2 == 0),
            2 => FhirPathValue::interned_string(format!("item_{}", i % 100)),
            _ => FhirPathValue::Empty,
        })
        .collect();

    let collection = Collection::from_iter(items);

    let duration = start.elapsed();
    println!(
        "Created mixed collection with {} items in {:?}",
        collection.len(),
        duration
    );
    println!(
        "✅ Integration test: {:.2} items/ms",
        collection.len() as f64 / duration.as_millis() as f64
    );

    let final_stats = global_interner_stats();
    println!("Final interned strings: {}", final_stats.entries);
}

async fn test_fhirpath_evaluation() {
    println!("\n📊 FHIRPath Evaluation Test");
    println!("---------------------------");

    let patient_data = create_sample_patient();
    let engine = FhirPathEngine::with_mock_provider();

    // Test various expressions including .where() function
    let expressions = [
        "Patient.name",
        "Patient.name.given",
        "Patient.name.where(use = 'official')",
        "Patient.name.where(family.exists())",
        "Patient.telecom.where(system = 'phone')",
        "Patient.address.where(use = 'home').city",
    ];

    for expression in &expressions {
        let start = Instant::now();

        // Run multiple iterations for timing
        for _ in 0..100 {
            match engine.evaluate_str(expression, &patient_data).await {
                Ok(_result) => {}
                Err(e) => {
                    eprintln!("Error evaluating '{expression}': {e}");
                    break;
                }
            }
        }

        let duration = start.elapsed();
        let per_eval = duration.as_micros() as f64 / 100.0;
        println!("  {expression}: {per_eval:.2}μs per evaluation");
    }

    println!("✅ FHIRPath evaluation test complete");
}
