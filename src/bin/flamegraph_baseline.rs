//! Generate flamegraph for baseline performance

use octofhir_fhirpath::engine::FhirPathEngine;
use pprof::ProfilerGuard;
use serde_json::Value;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load medium dataset for balanced profiling
    let data = serde_json::from_str::<Value>(&fs::read_to_string("benches/fixtures/medium.json")?)?;

    // Test complex Bundle operation that stresses the system
    let expression =
        "Bundle.entry.resource.where($this is Patient).name.where(use = 'official').given";

    println!("Running complex Bundle operation for flamegraph profiling...");

    let guard = ProfilerGuard::new(100)?;

    let mut engine = FhirPathEngine::new();

    // Run enough iterations to get meaningful profiling data
    for i in 0..50 {
        let _result = engine.evaluate(expression, data.clone()).await?;
        if i % 10 == 0 {
            println!("Iteration {}/50 completed", i + 1);
        }
    }

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("opt/flamegraphs/baseline_flamegraph.svg")?;
        report.flamegraph(file)?;
        println!("Flamegraph saved to opt/flamegraphs/baseline_flamegraph.svg");
    }

    println!("Flamegraph profiling completed!");

    Ok(())
}
