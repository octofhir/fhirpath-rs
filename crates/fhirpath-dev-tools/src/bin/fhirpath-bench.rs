use anyhow::Result;
use clap::{Parser, Subcommand};
use octofhir_fhir_model::FhirVersion;
use std::fs;
use std::path::{Path, PathBuf};

// Memory and system info
use sysinfo::{Pid, System};

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
    /// Profile a FHIRPath expression and optionally generate a flamegraph
    Profile {
        /// FHIRPath expression to profile
        expression: String,
        /// Output directory for profile artifacts (results + flamegraph)
        #[arg(short, long, default_value = "./profile_output")]
        output: PathBuf,
        /// Number of iterations for profiling
        #[arg(short, long, default_value = "1000")]
        iterations: usize,
        /// Use bundle data instead of patient data
        #[arg(short, long)]
        bundle: bool,
        /// Generate a CPU flamegraph using pprof
        #[arg(long, default_value_t = false)]
        flame: bool,
        /// Sampling frequency (Hz) for pprof (only if --flame)
        #[arg(long, default_value_t = 99)]
        freq: i32,
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
            flame,
            freq,
        } => {
            println!("Profiling expression: {expression}");
            println!("Output directory: {}", output.display());
            println!("Iterations: {iterations}");
            println!("Using {} data", if bundle { "bundle" } else { "patient" });
            if flame {
                println!("Flamegraph: enabled (freq={freq} Hz)");
            }

            profile_expression(&expression, output, iterations, bundle, flame, freq).await?;
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
    flame: bool,
    freq: i32,
) -> Result<()> {
    use octofhir_fhirpath::FhirPathEngine;
    use octofhir_fhirschema::EmbeddedSchemaProvider;
    use std::sync::Arc;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir)?;

    println!("Setting up profiling environment...");

    // Initialize engine
    let registry = Arc::new(octofhir_fhirpath::create_function_registry());
    let model_provider = Arc::new(EmbeddedSchemaProvider::new(FhirVersion::R5))
        as Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>;
    let engine = FhirPathEngine::new(registry, model_provider.clone()).await?;

    // Get test data
    let data = if use_bundle {
        get_sample_bundle()
    } else {
        get_sample_patient()
    };

    println!("Running {iterations} iterations...");

    // Optional CPU profiling
    let mut flamegraph_path: Option<PathBuf> = None;
    let do_flame = if flame && cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        eprintln!(
            "⚠️  Skipping flamegraph on macOS aarch64 due to known profiler instability. Use Linux for flamegraphs."
        );
        false
    } else {
        flame
    };
    let profiler = if do_flame {
        match pprof::ProfilerGuard::new(freq) {
            Ok(guard) => Some(guard),
            Err(e) => {
                eprintln!("Failed to start pprof profiler: {e}");
                None
            }
        }
    } else {
        None
    };

    // Measure timing
    let start = std::time::Instant::now();
    for i in 0..iterations {
        if i % 100 == 0 && i > 0 {
            println!("Completed {i} iterations");
        }
        let collection = octofhir_fhirpath::Collection::single(
            octofhir_fhirpath::FhirPathValue::resource(data.clone()),
        );
        let ctx = octofhir_fhirpath::EvaluationContext::new(
            collection,
            model_provider.clone(),
            None,
            None,
            None,
        )
        .await;
        let _ = engine.evaluate(expression, &ctx).await;
    }
    let duration = start.elapsed();

    // Generate flamegraph if enabled
    if let Some(guard) = profiler {
        use std::fs::File;
        // Sanitize file name
        fn sanitize_for_filename(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            for ch in s.chars() {
                if ch.is_ascii_alphanumeric() {
                    out.push(ch);
                } else {
                    out.push('_');
                }
            }
            // Limit length
            if out.len() > 120 {
                out.truncate(120);
            }
            out
        }
        let fname = format!("flamegraph_{}.svg", sanitize_for_filename(expression));
        let path = output_dir.join(fname);
        match guard.report().build() {
            Ok(report) => match File::create(&path) {
                Ok(mut file) => {
                    if let Err(e) = report.flamegraph(&mut file) {
                        eprintln!("Failed to write flamegraph: {e}");
                    } else {
                        flamegraph_path = Some(path);
                    }
                }
                Err(e) => eprintln!("Failed to create flamegraph file: {e}"),
            },
            Err(e) => eprintln!("Failed to build pprof report: {e}"),
        }
    }

    let avg_time_ms = duration.as_millis() as f64 / iterations as f64;
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    println!("Profiling completed!");
    println!("Total time: {:.2}s", duration.as_secs_f64());
    println!("Average time per iteration: {avg_time_ms:.2}ms");
    println!("Operations per second: {}", format_ops_per_sec(ops_per_sec));
    if let Some(ref p) = flamegraph_path {
        println!("Flamegraph written to: {}", p.display());
    }

    // Write results to file
    let results_file = output_dir.join("profile_results.txt");
    let mut results_content = format!(
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
    if let Some(p) = &flamegraph_path {
        results_content.push_str(&format!("Flamegraph: {}\n", p.display()));
    }

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

fn get_rss_bytes() -> Option<u64> {
    let mut sys = System::new();
    sys.refresh_processes();
    let pid = Pid::from_u32(std::process::id());
    sys.process(pid).map(|p| p.memory() * 1024)
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GiB", b / GB)
    } else if b >= MB {
        format!("{:.2} MiB", b / MB)
    } else if b >= KB {
        format!("{:.2} KiB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

fn parse_ops_value(s: &str) -> Option<f64> {
    // expects like "35.0K ops/sec" or "1234 ops/sec"
    let s = s.trim();
    let num_part = s.split_whitespace().next()?;
    if let Some(stripped) = num_part.strip_suffix('K') {
        stripped.parse::<f64>().ok().map(|v| v * 1_000.0)
    } else if let Some(stripped) = num_part.strip_suffix('M') {
        stripped.parse::<f64>().ok().map(|v| v * 1_000_000.0)
    } else {
        num_part.parse::<f64>().ok()
    }
}

async fn run_benchmarks_and_generate(output_path: &Path) -> Result<()> {
    use octofhir_fhirpath::FhirPathEngine;
    use octofhir_fhirpath::parse_expression;
    use octofhir_fhirschema::EmbeddedSchemaProvider;

    use std::sync::Arc;
    use std::time::Instant;

    println!("Running benchmarks directly...");
    let mem_start = get_rss_bytes();
    let start = Instant::now();

    let expressions = BenchmarkExpressions::default();
    let mut results = Vec::new();

    // Setup for evaluation benchmarks
    let registry = Arc::new(octofhir_fhirpath::create_function_registry());

    // Use real FhirSchemaModelProvider with R5 for accurate benchmarks
    println!("Initializing EmbeddedModelProvider R5...");
    let model_provider = Arc::new(EmbeddedSchemaProvider::new(FhirVersion::R5))
        as Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>;

    let engine = FhirPathEngine::new(registry, model_provider.clone()).await?;
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
        model_provider: Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>,
        record_memory: bool,
    ) -> Vec<String> {
        let mut bench_results = Vec::new();
        println!("  Running {name} benchmarks...");

        for expr in expressions {
            let iterations = 100; // Fewer iterations for evaluation (more expensive)
            let mem_before = if record_memory { get_rss_bytes() } else { None };
            let start_time = Instant::now();

            for _ in 0..iterations {
                let collection = octofhir_fhirpath::Collection::single(
                    octofhir_fhirpath::FhirPathValue::resource(data.clone()),
                );
                let ctx = octofhir_fhirpath::EvaluationContext::new(
                    collection,
                    model_provider.clone(),
                    None,
                    None,
                    None,
                )
                .await;
                let _ = engine.evaluate(expr, &ctx).await;
            }

            let elapsed = start_time.elapsed();
            let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

            let mem_suffix = if record_memory {
                if let (Some(ms), Some(me)) = (mem_before, get_rss_bytes()) {
                    let delta = me.saturating_sub(ms);
                    format!(" (ΔRSS: {})", format_bytes(delta))
                } else {
                    " (ΔRSS: n/a)".to_string()
                }
            } else {
                String::new()
            };

            bench_results.push(format!(
                "  - `{expr}`: {}{}",
                format_ops_per_sec(ops_per_sec),
                mem_suffix
            ));
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
            model_provider.clone(),
            false,
        )
        .await,
    );
    results.extend(
        run_evaluate_benchmark(
            "Medium Evaluation",
            &expressions.medium,
            &patient_data,
            &engine,
            model_provider.clone(),
            false,
        )
        .await,
    );
    results.extend(
        run_evaluate_benchmark(
            "Complex Evaluation",
            &expressions.complex,
            &bundle_data,
            &engine,
            model_provider.clone(),
            true,
        )
        .await,
    );

    let duration = start.elapsed();
    println!("Benchmarks completed in {:.2}s", duration.as_secs_f64());

    let mem_end = get_rss_bytes();

    // Generate markdown content with actual results
    let benchmark_output = results.join("\n");
    let markdown_content = parse_and_format_results(&benchmark_output, mem_start.zip(mem_end));

    fs::write(output_path, markdown_content)?;
    println!("Benchmark results written to: {}", output_path.display());

    Ok(())
}

fn parse_and_format_results(benchmark_output: &str, mem_start_end: Option<(u64, u64)>) -> String {
    use std::collections::{HashMap, HashSet};

    let expressions = BenchmarkExpressions::default();

    let simple: HashSet<&'static str> = expressions.simple.iter().copied().collect();
    let medium: HashSet<&'static str> = expressions.medium.iter().copied().collect();
    let complex: HashSet<&'static str> = expressions.complex.iter().copied().collect();

    // Accumulators: (section, category) -> (sum ops, count)
    let mut sums: HashMap<(&str, &str), (f64, usize)> = HashMap::new();

    // Rows for the complex evaluation memory table: (expr, ops_fmt, mem_fmt)
    let mut complex_eval_rows: Vec<(String, String, String)> = Vec::new();

    // Current section tracker
    let mut section = ""; // "Tokenize" | "Parse" | "Evaluate"

    for raw in benchmark_output.lines() {
        let l = raw.trim();
        if l.starts_with("## ") {
            if l.contains("Tokenization") {
                section = "Tokenize";
            } else if l.contains("Parsing") {
                section = "Parse";
            } else if l.contains("Evaluation") {
                section = "Evaluate";
            }
            continue;
        }

        // Benchmark entry lines look like: "- `expr`: 12.3K ops/sec (ΔRSS: 4.2 MiB)"
        if !(l.contains('`') && l.contains("ops/sec")) {
            continue;
        }

        // Extract expression between backticks
        let expr = if let Some(start) = l.find('`') {
            if let Some(end_rel) = l[start + 1..].find('`') {
                &l[start + 1..start + 1 + end_rel]
            } else {
                continue;
            }
        } else {
            continue;
        };

        // Extract ops/sec numeric value (for averaging) and format string
        let ops_str = l.rsplit_once(':').map(|(_, rhs)| rhs.trim()).unwrap_or("");
        let ops = parse_ops_value(ops_str).unwrap_or(0.0);
        let ops_fmt = format_ops_per_sec(ops);

        let category = if simple.contains(expr) {
            "Simple"
        } else if medium.contains(expr) {
            "Medium"
        } else if complex.contains(expr) {
            "Complex"
        } else {
            "Unknown"
        };

        // Update averages
        let key = (section, category);
        let entry = sums.entry(key).or_insert((0.0, 0));
        entry.0 += ops;
        entry.1 += 1;

        // Capture complex evaluation memory rows
        if section == "Evaluate" && category == "Complex" {
            let mem_fmt = if let Some(tail) = l.split("ΔRSS:").nth(1) {
                tail.trim().trim_end_matches(')').to_string()
            } else {
                "n/a".to_string()
            };
            complex_eval_rows.push((expr.to_string(), ops_fmt.clone(), mem_fmt));
        }
    }

    let avg = |sec: &str, cat: &str| -> String {
        if let Some((sum, cnt)) = sums.get(&(sec, cat)) {
            if *cnt > 0 {
                return format_ops_per_sec(sum / *cnt as f64);
            }
        }
        "-".to_string()
    };

    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let tool_ver = env!("CARGO_PKG_VERSION");

    let (mem_start_s, mem_end_s, mem_delta_s) = if let Some((ms, me)) = mem_start_end {
        let delta = me.saturating_sub(ms);
        (format_bytes(ms), format_bytes(me), format_bytes(delta))
    } else {
        ("n/a".to_string(), "n/a".to_string(), "n/a".to_string())
    };

    // Build complex evaluation memory table (if we have rows)
    let complex_mem_table = if complex_eval_rows.is_empty() {
        String::new()
    } else {
        let mut s = String::from(
            "\n## Complex Evaluation Memory by Expression\n\n| Expression | Ops/sec | ΔRSS |\n|------------|---------|------|\n",
        );
        for (expr, ops_fmt, mem_fmt) in complex_eval_rows {
            s.push_str(&format!("| `{expr}` | {ops_fmt} | {mem_fmt} |\n"));
        }
        s
    };

    format!(
        r#"# FHIRPath-rs Benchmark Results

Generated on: {}

## Overview

This benchmark suite measures the performance of FHIRPath-rs library across three main operations:
- **Tokenization**: Converting FHIRPath expressions into tokens
- **Parsing**: Building AST from tokens  
- **Evaluation**: Executing expressions against FHIR data

## Environment
- Tool: fhirpath-bench v{}
- OS/Arch: {} / {}
- CPU cores: {}
- FHIR Schema: R5

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
| Simple   | Tokenize  | {} | Basic expressions |
| Simple   | Parse     | {} | Basic expressions |
| Simple   | Evaluate  | {} | Basic expressions |
| Medium   | Tokenize  | {} | Filtered queries |
| Medium   | Parse     | {} | Filtered queries |
| Medium   | Evaluate  | {} | Filtered queries |
| Complex  | Tokenize  | {} | Bundle operations |
| Complex  | Parse     | {} | Bundle operations |
| Complex  | Evaluate  | {} | Bundle operations |
{}
## Memory
- RSS at start: {}
- RSS at end: {}
- RSS delta: {}

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
        tool_ver,
        os,
        arch,
        cores,
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
        avg("Tokenize", "Simple"),
        avg("Parse", "Simple"),
        avg("Evaluate", "Simple"),
        avg("Tokenize", "Medium"),
        avg("Parse", "Medium"),
        avg("Evaluate", "Medium"),
        avg("Tokenize", "Complex"),
        avg("Parse", "Complex"),
        avg("Evaluate", "Complex"),
        complex_mem_table,
        mem_start_s,
        mem_end_s,
        mem_delta_s,
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
