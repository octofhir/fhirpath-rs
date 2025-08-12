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

//! Performance baseline measurement for Phase 0 optimizations

use octofhir_fhirpath::FhirPathEngine;
use serde_json::Value;
use std::fs;
use std::time::Instant;

const EXPRESSIONS: &[(&str, &str)] = &[
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
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FHIRPath Performance Baseline - Phase 0");
    println!("============================================");

    // Load test datasets
    let small = serde_json::from_str::<Value>(&fs::read_to_string("benches/fixtures/small.json")?)?;
    let medium =
        serde_json::from_str::<Value>(&fs::read_to_string("benches/fixtures/medium.json")?)?;
    let large = serde_json::from_str::<Value>(&fs::read_to_string("benches/fixtures/large.json")?)?;

    let datasets = [
        ("Small (822KB)", small),
        ("Medium (5MB)", medium),
        ("Large (17MB)", large),
    ];

    for (dataset_name, dataset) in &datasets {
        println!("\n📊 Dataset: {dataset_name}");
        println!("{:-<50}", "");

        for (expr_name, expression) in EXPRESSIONS {
            let engine = FhirPathEngine::with_mock_provider();

            // Warm up
            for _ in 0..3 {
                let _ = engine.evaluate_str(expression, dataset).await;
            }

            // Measure 10 iterations
            let start = Instant::now();
            let iterations = 10;

            for _ in 0..iterations {
                let result = engine.evaluate_str(expression, &dataset).await;
                match result {
                    Ok(_) => {}
                    Err(e) => println!("  ❌ Error: {e}"),
                }
            }

            let elapsed = start.elapsed();
            let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

            println!("  {expr_name} - {avg_ms:.2}ms/eval");
        }
    }

    // Memory cloning baseline
    println!("\n🧠 Memory Operation Baseline");
    println!("{:-<50}", "");

    for (dataset_name, dataset) in &datasets {
        let start = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let _cloned = dataset.clone();
        }

        let elapsed = start.elapsed();
        let avg_us = elapsed.as_micros() as f64 / iterations as f64;

        println!("  {dataset_name} clone - {avg_us:.2}μs/clone");
    }

    Ok(())
}
