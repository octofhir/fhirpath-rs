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

//! Generate flamegraph for baseline performance

use octofhir_fhirpath::FhirPathEngine;
#[cfg(feature = "profiling")]
use pprof::ProfilerGuard;
use serde_json::Value;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    // Load large dataset for performance analysis
    let data = serde_json::from_str::<Value>(&fs::read_to_string("benches/fixtures/large.json")?)?;

    // Test the specific expression for analysis
    let expression = "Bundle.entry.resource.where(resourceType='Encounter' and meta.profile.contains('http://fhir.mimic.mit.edu/StructureDefinition/mimic-encounter-icu')).partOf.reference";

    println!("Running complex Bundle operation for flamegraph profiling...");

    #[cfg(feature = "profiling")]
    let guard = ProfilerGuard::new(100)?;

    let engine = FhirPathEngine::with_mock_provider();

    // Run enough iterations to get meaningful profiling data
    for i in 0..50 {
        let _result = engine.evaluate(expression, data.clone()).await?;
        if i % 10 == 0 {
            println!("Iteration {}/50 completed", i + 1);
        }
    }

    #[cfg(feature = "profiling")]
    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("opt/flamegraphs/baseline_flamegraph.svg")?;
        report.flamegraph(file)?;
        println!("Flamegraph saved to opt/flamegraphs/baseline_flamegraph.svg");
    }

    #[cfg(not(feature = "profiling"))]
    {
        println!(
            "Profiling feature not enabled. Run with --features profiling to generate flamegraph."
        );
    }

    println!("Flamegraph profiling completed!");

    Ok(())
}
