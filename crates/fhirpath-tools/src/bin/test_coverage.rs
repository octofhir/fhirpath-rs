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

//! Test coverage generation binary for FHIRPath implementation
//!
//! This binary runs all official FHIRPath test suites and generates a comprehensive
//! coverage report saved to TEST_COVERAGE.md

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use std::fs;
use std::path::PathBuf;

// Import the integration test runner components from parent module
use octofhir_fhirpath_tools::integration_test_runner::{IntegrationTestRunner, TestStats};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("test-coverage")
        .version("0.1.0")
        .about("Generate test coverage reports for FHIRPath implementation")
        .arg(
            Arg::new("specs-dir")
                .long("specs-dir")
                .value_name("DIR")
                .help("Path to FHIRPath test specifications")
                .default_value("specs/fhirpath/tests"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file for coverage report")
                .default_value("TEST_COVERAGE.md"),
        )
        .get_matches();

    let specs_dir = PathBuf::from(matches.get_one::<String>("specs-dir").unwrap());
    let output_file = PathBuf::from(matches.get_one::<String>("output").unwrap());

    println!("🧪 Generating FHIRPath Test Coverage Report");
    println!("============================================");

    if !specs_dir.exists() {
        println!("❌ Specs directory not found: {}", specs_dir.display());
        return Ok(());
    }

    let test_files = get_all_test_files(&specs_dir);
    if test_files.is_empty() {
        println!("❌ No test files found in specs directory");
        return Ok(());
    }

    println!("📁 Found {} test files", test_files.len());
    println!("🏃 Running tests...\n");

    let mut runner = IntegrationTestRunner::new()
        .await
        .with_base_path(&specs_dir)
        .with_verbose(false);

    let mut test_results = Vec::new();
    let mut processed = 0;

    for test_file in &test_files {
        if let Some(filename) = test_file.file_stem().and_then(|s| s.to_str()) {
            processed += 1;
            print!(
                "({:2}/{:2}) Testing {:<25}",
                processed,
                test_files.len(),
                format!("{}.json", filename)
            );

            match runner.run_and_report_quiet(test_file).await {
                Ok(stats) => {
                    let emoji = if stats.pass_rate() == 100.0 {
                        "✅"
                    } else if stats.pass_rate() >= 70.0 {
                        "🟡"
                    } else if stats.pass_rate() >= 30.0 {
                        "🟠"
                    } else {
                        "🔴"
                    };

                    println!(
                        " {} {}/{} ({:.1}%)",
                        emoji,
                        stats.passed,
                        stats.total,
                        stats.pass_rate()
                    );

                    test_results.push((filename.to_string(), stats));
                }
                Err(e) => {
                    println!(" ❌ Error: {e}");
                    test_results.push((
                        filename.to_string(),
                        TestStats {
                            total: 1,
                            passed: 0,
                            failed: 0,
                            errored: 1,
                            skipped: 0,
                        },
                    ));
                }
            }
        }
    }

    println!("\n📊 Generating coverage report...");

    // Generate comprehensive report
    let report = generate_coverage_report(&test_results);
    fs::write(&output_file, report)?;

    let total_tests: usize = test_results.iter().map(|(_, r)| r.total).sum();
    let total_passed: usize = test_results.iter().map(|(_, r)| r.passed).sum();
    let overall_pass_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    println!("✅ Coverage report saved to: {}", output_file.display());
    println!("\n📈 Summary:");
    println!("   Test Suites: {}", test_results.len());
    println!("   Total Tests: {total_tests}");
    println!("   Pass Rate: {overall_pass_rate:.1}%");

    Ok(())
}

fn get_all_test_files(specs_path: &PathBuf) -> Vec<PathBuf> {
    let mut test_files = Vec::new();
    if let Ok(entries) = fs::read_dir(specs_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                test_files.push(path);
            }
        }
    }

    test_files.sort();
    test_files
}

fn generate_coverage_report(test_results: &[(String, TestStats)]) -> String {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.format("%Y-%m-%d").to_string();

    let total_suites = test_results.len();
    let total_tests: usize = test_results.iter().map(|(_, r)| r.total).sum();
    let total_passed: usize = test_results.iter().map(|(_, r)| r.passed).sum();
    let total_failed: usize = test_results.iter().map(|(_, r)| r.failed).sum();
    let total_errors: usize = test_results.iter().map(|(_, r)| r.errored).sum();

    let overall_pass_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    let mut report = format!(
        r#"# FHIRPath Test Coverage Report

Generated on: {}
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | {} | 100% |
| **Total Individual Tests** | {} | 100% |
| **Passing Tests** | {} | {:.1}% |
| **Failing Tests** | {} | {:.1}% |
| **Error Tests** | {} | {:.1}% |

## Test Results by Suite

"#,
        timestamp,
        total_suites,
        total_tests,
        total_passed,
        overall_pass_rate,
        total_failed,
        if total_tests > 0 {
            (total_failed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        },
        total_errors,
        if total_tests > 0 {
            (total_errors as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        }
    );

    // Sort by pass rate
    let mut sorted_results = test_results.to_vec();
    sorted_results.sort_by(|a, b| b.1.pass_rate().partial_cmp(&a.1.pass_rate()).unwrap());

    // Fully passing tests
    report.push_str("### ✅ Fully Passing (100%)\n\n");
    let fully_passing: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() == 100.0)
        .collect();
    if fully_passing.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &fully_passing {
            report.push_str(&format!(
                "- **{}.json** - {}/{} tests\n",
                name, stats.passed, stats.total
            ));
        }
        report.push('\n');
    }

    // Well implemented
    report.push_str("### 🟡 Well Implemented (70%+)\n\n");
    let well_implemented: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() >= 70.0 && s.pass_rate() < 100.0)
        .collect();
    if well_implemented.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &well_implemented {
            report.push_str(&format!(
                "- **{}.json** - {:.1}% ({}/{} tests)\n",
                name,
                stats.pass_rate(),
                stats.passed,
                stats.total
            ));
        }
        report.push('\n');
    }

    // Partially implemented
    report.push_str("### 🟠 Partially Implemented (30-70%)\n\n");
    let partially_implemented: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() >= 30.0 && s.pass_rate() < 70.0)
        .collect();
    if partially_implemented.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &partially_implemented {
            report.push_str(&format!(
                "- **{}.json** - {:.1}% ({}/{} tests)\n",
                name,
                stats.pass_rate(),
                stats.passed,
                stats.total
            ));
        }
        report.push('\n');
    }

    // Major issues
    report.push_str("### 🔴 Major Issues (0-30%)\n\n");
    let major_issues: Vec<_> = sorted_results
        .iter()
        .filter(|(_, s)| s.pass_rate() < 30.0)
        .collect();
    if major_issues.is_empty() {
        report.push_str("None currently.\n\n");
    } else {
        for (name, stats) in &major_issues {
            let status = if stats.passed == 0 {
                "Missing"
            } else {
                "Issues"
            };
            report.push_str(&format!(
                "- **{}.json** - {:.1}% ({}/{} tests) - {}\n",
                name,
                stats.pass_rate(),
                stats.passed,
                stats.total,
                status
            ));
        }
        report.push('\n');
    }

    report.push_str(&format!(
        r#"## Summary

The fhirpath-rs implementation currently passes approximately **{:.1}% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: {}
- **Total Tests**: {}
- **Pass Rate**: {:.1}%

---

*Report generated on: {}*
*Command: `just test-coverage` or `cargo run --package fhirpath-tools --bin test-coverage`*
"#,
        overall_pass_rate,
        total_suites,
        total_tests,
        overall_pass_rate,
        now.format("%Y-%m-%d %H:%M:%S")
    ));

    report
}
