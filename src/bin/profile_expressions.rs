use anyhow::{Context, Result};
use clap::Parser;
use octofhir_fhirpath::engine::FhirPathEngine;
use pprof::ProfilerGuardBuilder;
use serde::Serialize;
use serde_json::Value;
use std::fs::{File, create_dir_all, read_to_string};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::runtime::Builder as RtBuilder;

/// Profiles two FHIRPath expressions against benches/fixtures/{small,medium,large}.json
/// and generates flamegraphs per (dataset, expression) combination into opt/flamegraphs.
#[derive(Debug, Parser)]
#[command(
    name = "profile-expressions",
    version,
    about = "Generate flamegraphs for selected FHIRPath expressions"
)]
struct Args {
    /// Number of evaluation iterations inside the sampling window
    #[arg(short = 'n', long = "iterations", default_value_t = 5_000)]
    iterations: usize,

    /// Sampling frequency in Hz
    #[arg(short = 'f', long = "frequency", default_value_t = 997)]
    frequency_hz: i32,

    /// Output folder for flamegraphs
    #[arg(short = 'o', long = "out-dir", default_value = "opt/flamegraphs")]
    out_dir: PathBuf,

    /// Warmup iterations (not sampled)
    #[arg(long = "warmup", default_value_t = 500)]
    warmup: usize,
}

// Expressions to profile (name, expression)
const EXPRESSIONS: &[(&str, &str)] = &[
    (
        "medication_request_count",
        "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()",
    ),
    (
        "icu_encounter_part_of_ref",
        "Bundle.entry.resource.where(resourceType='Encounter' and meta.profile.contains('http://fhir.mimic.mit.edu/StructureDefinition/mimic-encounter-icu')).partOf.reference",
    ),
];

// Datasets to profile
const DATASETS: &[&str] = &["small", "medium", "large"];

#[derive(Debug, Serialize, Clone)]
struct EvalStats {
    iterations: usize,
    total_us: f64,
    avg_us_per_iter: f64,
}

#[derive(Debug, Serialize, Clone)]
struct RunResult {
    dataset: String,
    expression_name: String,
    iterations: usize,
    total_us: f64,
    avg_us_per_iter: f64,
    output_svg: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Prepare output directory (use provided out_dir as flamegraph dir)
    let flame_dir = args.out_dir.clone();
    create_dir_all(&flame_dir).context("creating output directory")?;

    // Current-thread runtime to reduce noise
    let rt = RtBuilder::new_current_thread().enable_all().build()?;

    let mut results: Vec<RunResult> = Vec::new();

    for dataset in DATASETS {
        let json = load_fixture(dataset).with_context(|| format!("loading dataset {dataset}"))?;
        for (name, expr) in EXPRESSIONS {
            let outfile = flame_dir.join(format!("{}-{}.svg", dataset, filename_slug(name, expr)));
            println!(
                "Profiling dataset='{}' expr='{}' -> {}",
                dataset,
                name,
                outfile.display()
            );
            let stats = profile_expression(
                &rt,
                &json,
                expr,
                &outfile,
                args.iterations,
                args.frequency_hz,
                args.warmup,
            )
            .with_context(|| format!("profiling {name} on {dataset}"))?;
            results.push(RunResult {
                dataset: (*dataset).to_string(),
                expression_name: (*name).to_string(),
                iterations: stats.iterations,
                total_us: stats.total_us,
                avg_us_per_iter: stats.avg_us_per_iter,
                output_svg: outfile.to_string_lossy().into_owned(),
            });
        }
    }

    // Write JSON results summary
    let results_path = flame_dir.join("results.json");
    let mut f = File::create(&results_path).context("creating results.json")?;
    serde_json::to_writer_pretty(&mut f, &results).context("writing results.json")?;

    // Also print a compact summary
    println!("\nSummary (avg µs/iter):");
    for r in &results {
        println!(
            "  {:<6} | {:<28} | {:>10.3}",
            r.dataset, r.expression_name, r.avg_us_per_iter
        );
    }

    println!("\nFlamegraphs written to: {}", flame_dir.display());
    println!("Results JSON: {}", results_path.display());
    println!("Tip: For best results, compile with debug symbols and enable frame pointers.");
    println!("Example: RUSTFLAGS='-C force-frame-pointers=yes' cargo build --release");
    Ok(())
}

fn load_fixture(name: &str) -> Result<Value> {
    let path = format!("benches/fixtures/{name}.json");
    let contents = read_to_string(&path).with_context(|| format!("reading {path}"))?;
    let json: Value = serde_json::from_str(&contents).with_context(|| format!("parsing {path}"))?;
    Ok(json)
}

fn profile_expression(
    rt: &tokio::runtime::Runtime,
    input: &Value,
    expr: &str,
    output_svg: &Path,
    iterations: usize,
    frequency_hz: i32,
    warmup: usize,
) -> Result<EvalStats> {
    // Warmup outside the profiler to stabilize JIT-like effects and caches
    {
        let mut engine = FhirPathEngine::new();
        for _ in 0..warmup {
            let _ = rt.block_on(engine.evaluate(expr, input.clone()));
        }
    }

    // Run inside profiler
    let guard = ProfilerGuardBuilder::default()
        .frequency(frequency_hz)
        .blocklist(&[
            // Reduce runtime noise in profiles
            "libc",
            "libunwind",
            "pthread",
            "tokio",
            "std",
            "libsystem",
        ])
        .build()
        .context("starting pprof profiler")?;

    let mut engine = FhirPathEngine::new();

    let start = Instant::now();
    for _ in 0..iterations {
        // Evaluate and ignore output; we care about time/cpu samples
        let _ = rt.block_on(engine.evaluate(expr, input.clone()));
    }
    let elapsed = start.elapsed();
    println!(
        "Ran {} iterations in {:?} (avg {:.3} µs/iter)",
        iterations,
        elapsed,
        (elapsed.as_secs_f64() * 1e6) / iterations as f64
    );

    // Build and write the flamegraph
    if let Ok(report) = guard.report().build() {
        let mut file = File::create(output_svg)
            .with_context(|| format!("creating {}", output_svg.display()))?;
        report.flamegraph(&mut file).context("writing flamegraph")?;
    } else {
        eprintln!("Warning: profiler report not available (insufficient samples?)");
    }

    Ok(EvalStats {
        iterations,
        total_us: elapsed.as_secs_f64() * 1e6,
        avg_us_per_iter: (elapsed.as_secs_f64() * 1e6) / iterations as f64,
    })
}

/// Create a stable, short, filesystem-safe slug for filenames
fn filename_slug(name: &str, expr: &str) -> String {
    // Combine a readable prefix with a short hash of the expr to keep names unique
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    expr.hash(&mut hasher);
    let h = hasher.finish();
    let safe = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    format!("{safe}-{h:016x}")
}
