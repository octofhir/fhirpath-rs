//! Simple test coverage report generator
//!
//! This test runs all official FHIRPath test suites and generates a basic
//! coverage report saved to TEST_COVERAGE.md

use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;

mod integration_test_runner;
use integration_test_runner::{IntegrationTestRunner, TestStats};

fn get_specs_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("specs")
        .join("fhirpath")
        .join("tests")
}

fn get_all_test_files() -> Vec<PathBuf> {
    let specs_path = get_specs_path();

    let mut test_files = Vec::new();
    if let Ok(entries) = fs::read_dir(&specs_path) {
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

#[test]
#[ignore] // Use --ignored to run this test
fn run_coverage_report() {
    println!("🧪 Generating FHIRPath Test Coverage Report");
    println!("============================================");

    let specs_path = get_specs_path();
    if !specs_path.exists() {
        println!("❌ Specs directory not found: {}", specs_path.display());
        return;
    }

    let test_files = get_all_test_files();
    if test_files.is_empty() {
        println!("❌ No test files found in specs directory");
        return;
    }

    println!("📁 Found {} test files", test_files.len());
    println!("🏃 Running tests...\n");

    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
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

            match runner.run_and_report(test_file) {
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

    // Generate a basic report
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
Implementation: fhirpath-rs (fhirpath-core)

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
    let mut sorted_results = test_results.clone();
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
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
"#,
        overall_pass_rate,
        total_suites,
        total_tests,
        overall_pass_rate,
        now.format("%Y-%m-%d %H:%M:%S")
    ));

    // Save the report
    let report_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("TEST_COVERAGE.md");
    match fs::write(&report_path, &report) {
        Ok(()) => {
            println!("✅ Coverage report saved to: {}", report_path.display());

            println!("\n📈 Summary:");
            println!("   Test Suites: {total_suites}");
            println!("   Total Tests: {total_tests}");
            println!("   Pass Rate: {overall_pass_rate:.1}%");
            println!("\n🔄 To regenerate: cargo test run_coverage_report -- --ignored --nocapture");
        }
        Err(e) => {
            println!("❌ Failed to save report: {e}");
        }
    }
}
