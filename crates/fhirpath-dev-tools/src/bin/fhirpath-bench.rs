use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

/// Format numbers in human-friendly format (K, M, etc.)
fn format_ops_per_sec(ops_per_sec: f64) -> String {
    if ops_per_sec >= 1_000_000.0 {
        format!("{:.1}M ops/sec", ops_per_sec / 1_000_000.0)
    } else if ops_per_sec >= 1_000.0 {
        format!("{:.1}K ops/sec", ops_per_sec / 1_000.0)
    } else {
        format!("{ops_per_sec:.0} ops/sec")
    }
}

#[derive(Parser)]
#[command(name = "fhirpath-bench")]
#[command(about = "FHIRPath-rs benchmarking and profiling tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Profile a FHIRPath expression and generate flamegraph
    Profile {
        /// FHIRPath expression to profile
        expression: String,
        /// Output directory for flamegraph files
        #[arg(short, long, default_value = "./profile_output")]
        output: PathBuf,
        /// Number of iterations for profiling
        #[arg(short, long, default_value = "1000")]
        iterations: usize,
        /// Use bundle data instead of patient data
        #[arg(short, long)]
        bundle: bool,
    },
    /// Generate benchmark.md file with results
    Benchmark {
        /// Output file path for benchmark results
        #[arg(short, long, default_value = "benchmark.md")]
        output: PathBuf,
        /// Run actual benchmarks (otherwise just generates template)
        #[arg(short, long)]
        run: bool,
    },
    /// List available expressions for benchmarking
    List,
}

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
                "Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value",
            ],
        }
    }
}

/// Sample FHIR data for benchmarking
pub fn get_sample_patient() -> serde_json::Value {
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

pub fn get_sample_bundle() -> serde_json::Value {
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Profile {
            expression,
            output,
            iterations,
            bundle,
        } => {
            println!("Profiling expression: {expression}");
            println!("Output directory: {}", output.display());
            println!("Iterations: {iterations}");
            println!("Using {} data", if bundle { "bundle" } else { "patient" });

            profile_expression(&expression, output, iterations, bundle).await?;
        }
        Commands::Benchmark { output, run } => {
            if run {
                println!("Running benchmarks and generating results...");
                run_benchmarks_and_generate(&output).await?;
            } else {
                println!("Generating benchmark template...");
                let content = generate_benchmark_summary();
                fs::write(&output, content)?;
                println!("Benchmark template written to: {}", output.display());
            }
        }
        Commands::List => {
            list_expressions();
        }
    }

    Ok(())
}

async fn profile_expression(
    expression: &str,
    output_dir: PathBuf,
    iterations: usize,
    use_bundle: bool,
) -> Result<()> {
    use octofhir_fhirpath::FhirPathEngine;
    use octofhir_fhirschema::provider::FhirSchemaModelProvider;
    use std::sync::Arc;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir)?;

    println!("Setting up profiling environment...");

    // Initialize engine
    let registry = Arc::new(octofhir_fhirpath::create_standard_registry().await);
    let model_provider = Arc::new(
        FhirSchemaModelProvider::r5()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create R5 FHIR Schema Provider: {}", e))?,
    ) as Arc<dyn octofhir_fhir_model::ModelProvider>;
    let engine = FhirPathEngine::new(registry, model_provider);

    // Get test data
    let data = if use_bundle {
        get_sample_bundle()
    } else {
        get_sample_patient()
    };

    println!("Running {} iterations...", iterations);

    // Simple profiling - just measure time for now
    let start = std::time::Instant::now();
    for i in 0..iterations {
        if i % 100 == 0 && i > 0 {
            println!("Completed {} iterations", i);
        }
        let collection = octofhir_fhirpath::Collection::single(octofhir_fhirpath::FhirPathValue::resource(data.clone()));
        let _ = engine.evaluate(expression, &collection).await;
    }
    let duration = start.elapsed();

    let avg_time_ms = duration.as_millis() as f64 / iterations as f64;
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    println!("Profiling completed!");
    println!("Total time: {:.2}s", duration.as_secs_f64());
    println!("Average time per iteration: {:.2}ms", avg_time_ms);
    println!("Operations per second: {}", format_ops_per_sec(ops_per_sec));

    // Write results to file
    let results_file = output_dir.join("profile_results.txt");
    let results_content = format!(
        "Expression: {}\n\
         Iterations: {}\n\
         Data type: {}\n\
         Total time: {:.2}s\n\
         Average time per iteration: {:.2}ms\n\
         Operations per second: {}\n",
        expression,
        iterations,
        if use_bundle { "Bundle" } else { "Patient" },
        duration.as_secs_f64(),
        avg_time_ms,
        format_ops_per_sec(ops_per_sec)
    );

    fs::write(&results_file, results_content)?;
    println!("Results written to: {}", results_file.display());

    Ok(())
}

fn list_expressions() {
    let expressions = BenchmarkExpressions::default();

    println!("Available benchmark expressions:\n");

    println!("🟢 Simple Expressions:");
    for (i, expr) in expressions.simple.iter().enumerate() {
        println!("  {}. {}", i + 1, expr);
    }

    println!("\n🟡 Medium Expressions:");
    for (i, expr) in expressions.medium.iter().enumerate() {
        println!("  {}. {}", i + 1, expr);
    }

    println!("\n🔴 Complex Expressions:");
    for (i, expr) in expressions.complex.iter().enumerate() {
        println!("  {}. {}", i + 1, expr);
    }

    println!("\nTo profile a specific expression:");
    println!("  fhirpath-bench profile \"Patient.active\"");
    println!("  fhirpath-bench profile \"Bundle.entry.resource.count()\" --bundle");
}

async fn run_benchmarks_and_generate(output_path: &PathBuf) -> Result<()> {
    use octofhir_fhirpath::FhirPathEngine;
    use octofhir_fhirschema::provider::FhirSchemaModelProvider;
    use octofhir_fhirpath::parse_expression;

    use std::sync::Arc;
    use std::time::Instant;

    println!("Running benchmarks directly...");
    let start = Instant::now();

    let expressions = BenchmarkExpressions::default();
    let mut results = Vec::new();

    // Setup for evaluation benchmarks
    let registry = Arc::new(octofhir_fhirpath::create_standard_registry().await);

    // Use real FhirSchemaModelProvider with R5 for accurate benchmarks
    println!("Initializing FhirSchemaModelProvider R5...");
    let model_provider = Arc::new(
        FhirSchemaModelProvider::r5()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create R5 FHIR Schema Provider: {}", e))?,
    ) as Arc<dyn octofhir_fhir_model::ModelProvider>;

    let engine = FhirPathEngine::new(registry, model_provider);
    let patient_data = get_sample_patient();
    let bundle_data = get_sample_bundle();

    // Helper function to run benchmarks and measure performance
    let run_tokenize_benchmark = |name: &str, expressions: &[&str]| -> Vec<String> {
        let mut bench_results = Vec::new();
        println!("  Running {name} benchmarks...");

        for expr in expressions {
            let iterations = 1000;
            let start_time = Instant::now();

            for _ in 0..iterations {
                let _ = parse_expression(expr);
            }

            let elapsed = start_time.elapsed();
            let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

            bench_results.push(format!("  - `{expr}`: {}", format_ops_per_sec(ops_per_sec)));
        }

        bench_results
    };

    let run_parse_benchmark = |name: &str, expressions: &[&str]| -> Vec<String> {
        let mut bench_results = Vec::new();
        println!("  Running {name} benchmarks...");

        for expr in expressions {
            let iterations = 1000;
            let start_time = Instant::now();

            for _ in 0..iterations {
                let _ = parse_expression(expr);
            }

            let elapsed = start_time.elapsed();
            let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

            bench_results.push(format!("  - `{expr}`: {}", format_ops_per_sec(ops_per_sec)));
        }

        bench_results
    };

    // Helper function to run evaluation benchmarks
    async fn run_evaluate_benchmark(
        name: &str,
        expressions: &[&str],
        data: &serde_json::Value,
        engine: &FhirPathEngine,
    ) -> Vec<String> {
        let mut bench_results = Vec::new();
        println!("  Running {name} benchmarks...");

        for expr in expressions {
            let iterations = 100; // Fewer iterations for evaluation (more expensive)
            let start_time = Instant::now();

            for _ in 0..iterations {
                let collection = octofhir_fhirpath::Collection::single(octofhir_fhirpath::FhirPathValue::resource(data.clone()));
                let _ = engine.evaluate(expr, &collection).await;
            }

            let elapsed = start_time.elapsed();
            let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

            bench_results.push(format!("  - `{expr}`: {}", format_ops_per_sec(ops_per_sec)));
        }

        bench_results
    }

    // Run tokenization benchmarks
    results.push("## Tokenization Benchmarks".to_string());
    results.extend(run_tokenize_benchmark(
        "Simple Tokenization",
        &expressions.simple,
    ));
    results.extend(run_tokenize_benchmark(
        "Medium Tokenization",
        &expressions.medium,
    ));
    results.extend(run_tokenize_benchmark(
        "Complex Tokenization",
        &expressions.complex,
    ));

    // Run parsing benchmarks
    results.push("\n## Parsing Benchmarks".to_string());
    results.extend(run_parse_benchmark("Simple Parsing", &expressions.simple));
    results.extend(run_parse_benchmark("Medium Parsing", &expressions.medium));
    results.extend(run_parse_benchmark("Complex Parsing", &expressions.complex));

    // Run evaluation benchmarks
    results.push("\n## Evaluation Benchmarks".to_string());
    results.extend(
        run_evaluate_benchmark(
            "Simple Evaluation",
            &expressions.simple,
            &patient_data,
            &engine,
        )
        .await,
    );
    results.extend(
        run_evaluate_benchmark(
            "Medium Evaluation",
            &expressions.medium,
            &patient_data,
            &engine,
        )
        .await,
    );
    results.extend(
        run_evaluate_benchmark(
            "Complex Evaluation",
            &expressions.complex,
            &bundle_data,
            &engine,
        )
        .await,
    );

    let duration = start.elapsed();
    println!("Benchmarks completed in {:.2}s", duration.as_secs_f64());

    // Generate markdown content with actual results
    let benchmark_output = results.join("\n");
    let markdown_content = parse_and_format_results(&benchmark_output);

    fs::write(output_path, markdown_content)?;
    println!("Benchmark results written to: {}", output_path.display());

    Ok(())
}

fn parse_and_format_results(benchmark_output: &str) -> String {
    let expressions = BenchmarkExpressions::default();

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

```
{}
```

## Performance Summary

| Category | Operation | Avg Ops/sec | Notes |
|----------|-----------|-------------|--------|
| Simple   | Tokenize  | -           | Basic expressions |
| Simple   | Parse     | -           | Basic expressions |
| Simple   | Evaluate  | -           | Basic expressions |
| Medium   | Tokenize  | -           | Filtered queries |
| Medium   | Parse     | -           | Filtered queries |
| Medium   | Evaluate  | -           | Filtered queries |
| Complex  | Tokenize  | -           | Bundle operations |
| Complex  | Parse     | -           | Bundle operations |
| Complex  | Evaluate  | -           | Bundle operations |

## Usage

To run benchmarks:
```bash
cargo bench --package fhirpath-bench
```

To profile specific expressions:
```bash
fhirpath-bench profile "Patient.active"
fhirpath-bench profile "Bundle.entry.resource.count()" --bundle
```

To generate updated results:
```bash
fhirpath-bench benchmark --run --output benchmark.md
```
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        expressions
            .simple
            .iter()
            .map(|e| format!("- `{e}`"))
            .collect::<Vec<_>>()
            .join("\n"),
        expressions
            .medium
            .iter()
            .map(|e| format!("- `{e}`"))
            .collect::<Vec<_>>()
            .join("\n"),
        expressions
            .complex
            .iter()
            .map(|e| format!("- `{e}`"))
            .collect::<Vec<_>>()
            .join("\n"),
        benchmark_output,
    )
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
- {}

### Medium Expressions  
Filtered queries and basic functions:
- {}

### Complex Expressions
Bundle operations and resolve() calls:
- {}

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
