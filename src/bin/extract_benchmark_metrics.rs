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

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents benchmark performance data extracted from Criterion results
#[derive(Debug)]
#[allow(dead_code)]
struct BenchmarkData {
    /// Mean execution time in nanoseconds
    mean: f64,
    /// Standard deviation of execution time in nanoseconds
    std_dev: f64,
    /// Median execution time in nanoseconds
    median: f64,
    /// Human-readable formatted mean time (e.g., "2.45 μs")
    mean_formatted: String,
    /// Human-readable formatted median time (e.g., "2.43 μs")
    median_formatted: String,
    /// Formatted throughput (e.g., "408K ops/sec")
    ops_per_sec: String,
}

/// Formats nanoseconds into human-readable time units
///
/// # Arguments
/// * `nanoseconds` - Time in nanoseconds
///
/// # Returns
/// Formatted string with appropriate unit (ns, μs, ms, s)
fn format_time(nanoseconds: f64) -> String {
    if nanoseconds < 1000.0 {
        format!("{nanoseconds:.2} ns")
    } else if nanoseconds < 1_000_000.0 {
        format!("{:.2} μs", nanoseconds / 1000.0)
    } else if nanoseconds < 1_000_000_000.0 {
        format!("{:.2} ms", nanoseconds / 1_000_000.0)
    } else {
        format!("{:.2} s", nanoseconds / 1_000_000_000.0)
    }
}

/// Calculates operations per second from mean execution time
///
/// # Arguments
/// * `mean_time_ns` - Mean execution time in nanoseconds
///
/// # Returns
/// Formatted throughput string (e.g., "1.42M ops/sec")
fn calculate_ops_per_sec(mean_time_ns: f64) -> String {
    if mean_time_ns > 0.0 {
        let ops_per_sec = 1_000_000_000.0 / mean_time_ns;
        if ops_per_sec >= 1_000_000.0 {
            format!("{:.2}M ops/sec", ops_per_sec / 1_000_000.0)
        } else if ops_per_sec >= 1_000.0 {
            format!("{:.2}K ops/sec", ops_per_sec / 1_000.0)
        } else {
            format!("{ops_per_sec:.2} ops/sec")
        }
    } else {
        "N/A".to_string()
    }
}

/// Extracts benchmark performance data from Criterion output directory
///
/// # Arguments
/// * `criterion_dir` - Path to the target/criterion directory
///
/// # Returns
/// HashMap mapping benchmark names to their performance data
///
/// # Errors
/// Returns error if directory cannot be read or JSON parsing fails
fn extract_benchmark_data(
    criterion_dir: &Path,
) -> Result<HashMap<String, BenchmarkData>, Box<dyn std::error::Error>> {
    let mut benchmarks = HashMap::new();

    /// Recursively finds all estimates.json files in the new/ subdirectories
    fn find_estimates_files(
        dir: &Path,
        files: &mut Vec<std::path::PathBuf>,
    ) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    find_estimates_files(&path, files)?;
                } else if path.file_name().and_then(|n| n.to_str()) == Some("estimates.json")
                    && path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        == Some("new")
                {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    let mut estimate_files = Vec::new();
    find_estimates_files(criterion_dir, &mut estimate_files)?;

    for file_path in estimate_files {
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                    // Extract benchmark name from path
                    let parts: Vec<&str> = file_path
                        .components()
                        .map(|c| c.as_os_str().to_str().unwrap_or(""))
                        .collect();

                    if parts.len() >= 5 {
                        let group_idx = parts.len() - 5;
                        let test_idx = parts.len() - 4;
                        let complexity_idx = parts.len() - 3;

                        let group = parts[group_idx];
                        let test = parts[test_idx];
                        let complexity = parts[complexity_idx];

                        let benchmark_name = format!("{group}/{test}/{complexity}");

                        if let (Some(mean), Some(std_dev), Some(median)) = (
                            data["mean"]["point_estimate"].as_f64(),
                            data["std_dev"]["point_estimate"].as_f64(),
                            data["median"]["point_estimate"].as_f64(),
                        ) {
                            benchmarks.insert(
                                benchmark_name,
                                BenchmarkData {
                                    mean,
                                    std_dev,
                                    median,
                                    mean_formatted: format_time(mean),
                                    median_formatted: format_time(median),
                                    ops_per_sec: calculate_ops_per_sec(mean),
                                },
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not read {}: {}", file_path.display(), e);
            }
        }
    }

    Ok(benchmarks)
}

/// Generates BENCHMARKS.md file with extracted benchmark metrics
///
/// # Arguments
/// * `benchmarks` - HashMap of benchmark data
/// * `output_file` - Path to output BENCHMARKS.md file
///
/// # Returns
/// Result indicating success or failure
///
/// # Errors
/// Returns error if file cannot be written
fn generate_benchmarks_md(
    benchmarks: &HashMap<String, BenchmarkData>,
    output_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = format!(
        r#"# FHIRPath Benchmark Results

Last updated: {}

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

"#,
        chrono::Utc::now().format("%a %b %d %H:%M:%S UTC %Y")
    );

    // Group benchmarks by component
    let mut components: HashMap<String, Vec<(String, &BenchmarkData)>> = HashMap::new();
    for (benchmark_name, data) in benchmarks {
        let parts: Vec<&str> = benchmark_name.split('/').collect();
        if parts.len() >= 2 {
            let component = parts[0].to_string();
            components
                .entry(component)
                .or_default()
                .push((benchmark_name.clone(), data));
        }
    }

    // Add component sections
    let mut sorted_components: Vec<_> = components.iter().collect();
    sorted_components.sort_by_key(|(name, _)| name.as_str());

    for (component, bench_list) in sorted_components {
        content.push_str(&format!("#### {}\n\n", title_case(component)));
        content.push_str("| Complexity | Mean Time | Throughput | Median Time |\n");
        content.push_str("|------------|-----------|------------|-------------|\n");

        let mut sorted_bench_list = bench_list.clone();
        sorted_bench_list.sort_by_key(|(name, _)| name.clone());

        for (benchmark_name, data) in sorted_bench_list {
            let complexity = benchmark_name.split('/').next_back().unwrap_or("");
            let complexity_title = title_case(complexity);
            content.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                complexity_title, data.mean_formatted, data.ops_per_sec, data.median_formatted
            ));
        }

        content.push('\n');
    }

    content.push_str(r#"### Performance Summary

**Key Metrics:**
- **Tokenizer**: Processes FHIRPath expressions into tokens
- **Parser**: Builds AST from tokens using Pratt parsing
- **Evaluator**: Executes FHIRPath expressions against data
- **Full Pipeline**: Complete tokenize → parse → evaluate workflow

### Detailed Results

For detailed benchmark results, charts, and statistical analysis, see the HTML reports in `target/criterion/`.

### Running Benchmarks

```bash
# Run core benchmarks
just bench

# Run full benchmark suite
just bench-full

# Update this documentation
just bench-update-docs
```

### Benchmark Infrastructure

- **Framework**: Criterion.rs v0.7
- **Statistical Analysis**: Includes confidence intervals, outlier detection
- **Sample Sizes**: Adaptive sampling for statistical significance
- **Measurement**: Wall-clock time with warm-up cycles

"#);

    fs::write(output_file, content)?;
    Ok(())
}

/// Converts a string to title case (first letter uppercase, rest lowercase)
///
/// # Arguments
/// * `s` - Input string to convert
///
/// # Returns
/// Title-cased string
fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Main entry point for benchmark metrics extraction
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let criterion_dir = Path::new("target/criterion");
    let output_file = Path::new("BENCHMARKS.md");

    if !criterion_dir.exists() {
        eprintln!(
            "Error: Criterion directory '{}' not found.",
            criterion_dir.display()
        );
        eprintln!("Please run benchmarks first: just bench");
        std::process::exit(1);
    }

    println!("🔍 Extracting benchmark metrics from Criterion results...");
    let benchmarks = extract_benchmark_data(criterion_dir)?;

    if benchmarks.is_empty() {
        eprintln!("Warning: No benchmark data found in Criterion results.");
        std::process::exit(1);
    }

    println!("📊 Found {} benchmark results", benchmarks.len());

    println!("📝 Generating BENCHMARKS.md...");
    generate_benchmarks_md(&benchmarks, output_file)?;

    println!(
        "✅ BENCHMARKS.md updated successfully with {} benchmark metrics!",
        benchmarks.len()
    );

    // Print summary
    println!("\n📈 Performance Summary:");
    let mut sorted_benchmarks: Vec<_> = benchmarks.iter().collect();
    sorted_benchmarks.sort_by_key(|(name, _)| name.as_str());

    for (name, data) in sorted_benchmarks {
        println!("  {}: {} ({})", name, data.mean_formatted, data.ops_per_sec);
    }

    Ok(())
}
