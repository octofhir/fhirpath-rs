use anyhow::Result;
use clap::{Parser, Subcommand};
use fhirpath_bench::profiling::ProfileRunner;
use fhirpath_bench::{BenchmarkExpressions, generate_benchmark_summary};
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

            let profiler = ProfileRunner::new(output, iterations, bundle);
            profiler.profile_expression(&expression).await?;
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
    use fhirpath_bench::{BenchmarkExpressions, get_sample_bundle, get_sample_patient};
    use octofhir_fhirpath_evaluator::FhirPathEngine;
    use octofhir_fhirpath_model::FhirSchemaModelProvider;
    use octofhir_fhirpath_parser::{Tokenizer, parse_expression};
    use octofhir_fhirpath_registry::FunctionRegistry;
    use std::sync::Arc;
    use std::time::Instant;

    println!("Running benchmarks directly...");
    let start = Instant::now();

    let expressions = BenchmarkExpressions::default();
    let mut results = Vec::new();

    // Setup for evaluation benchmarks
    let registry = Arc::new(octofhir_fhirpath_registry::create_standard_registry());

    // Use real FhirSchemaModelProvider with R5 for accurate benchmarks
    println!("Initializing FhirSchemaModelProvider R5...");
    let model_provider = Arc::new(
        FhirSchemaModelProvider::r5()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create R5 FHIR Schema Provider: {}", e))?,
    ) as Arc<dyn octofhir_fhirpath_model::ModelProvider>;

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
                let _ = Tokenizer::new(expr).tokenize_all();
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
        data: &sonic_rs::Value,
        engine: &FhirPathEngine,
    ) -> Vec<String> {
        let mut bench_results = Vec::new();
        println!("  Running {name} benchmarks...");

        for expr in expressions {
            let iterations = 100; // Fewer iterations for evaluation (more expensive)
            let start_time = Instant::now();

            for _ in 0..iterations {
                let _ = engine.evaluate(expr, data.clone()).await;
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
