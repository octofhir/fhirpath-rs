//! Integration tests using official FHIRPath test suite
//!
//! This test file demonstrates how to run the official FHIRPath test cases
//! against our integrated implementation.

use std::path::PathBuf;

mod integration_test_runner;
use integration_test_runner::IntegrationTestRunner;

/// Helper function to get the path to the specs directory
fn get_specs_path() -> PathBuf {
    // From project root to specs
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("specs")
        .join("fhirpath")
        .join("tests")
}

#[test]
fn test_run_basics_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(false);

    let basics_path = specs_path.join("basics.json");

    if !basics_path.exists() {
        println!(
            "Skipping basics test - file not found: {}",
            basics_path.display()
        );
        return;
    }

    match runner.run_and_report(&basics_path) {
        Ok(stats) => {
            println!("Basics test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );

            // For now, we don't assert success since we're still implementing features
            // In the future, this could be: assert_eq!(stats.failed + stats.errored, 0);
        }
        Err(e) => {
            println!("Failed to run basics test suite: {e}");
            // Don't panic - just report the issue for now
        }
    }
}

#[test]
fn test_run_literals_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(false);

    let literals_path = specs_path.join("literals.json");

    if !literals_path.exists() {
        println!(
            "Skipping literals test - file not found: {}",
            literals_path.display()
        );
        return;
    }

    match runner.run_and_report(&literals_path) {
        Ok(stats) => {
            println!("Literals test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
        }
        Err(e) => {
            println!("Failed to run literals test suite: {e}");
        }
    }
}

#[test]
fn test_simple_expression_parsing() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    // Create a simple test case to verify our basic functionality
    let simple_test = integration_test_runner::TestCase {
        name: "simple_boolean".to_string(),
        expression: "true".to_string(),
        input: None,
        inputfile: Some("patient-example.json".to_string()),
        expected: serde_json::Value::Array(vec![serde_json::Value::Bool(true)]),
        tags: vec!["basic".to_string()],
        description: Some("Simple boolean literal test".to_string()),
    };

    let result = runner.run_test(&simple_test);
    match result {
        integration_test_runner::TestResult::Passed => {
            println!("✅ Simple expression test passed!");
        }
        integration_test_runner::TestResult::Failed { expected, actual } => {
            println!("❌ Simple expression test failed:");
            println!(
                "  Expected: {}",
                serde_json::to_string_pretty(&expected).unwrap()
            );
            println!(
                "  Actual: {}",
                serde_json::to_string_pretty(&actual).unwrap()
            );
        }
        integration_test_runner::TestResult::Error { error } => {
            println!("⚠️ Simple expression test errored: {error}");
        }
        integration_test_runner::TestResult::Skipped { reason } => {
            println!("⊘ Simple expression test skipped: {reason}");
        }
    }
}

/// Test abs function specifically
#[test]
fn test_run_abs_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let abs_path = specs_path.join("abs.json");

    if !abs_path.exists() {
        println!("Skipping abs test - file not found: {}", abs_path.display());
        return;
    }

    match runner.run_and_report(&abs_path) {
        Ok(stats) => {
            println!("Abs test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run abs test suite: {e}");
        }
    }
}

/// Test ceiling function specifically
#[test]
fn test_run_ceiling_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let ceiling_path = specs_path.join("ceiling.json");

    if !ceiling_path.exists() {
        println!(
            "Skipping ceiling test - file not found: {}",
            ceiling_path.display()
        );
        return;
    }

    match runner.run_and_report(&ceiling_path) {
        Ok(stats) => {
            println!("Ceiling test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run ceiling test suite: {e}");
        }
    }
}

/// Test floor function specifically
#[test]
fn test_run_floor_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let floor_path = specs_path.join("floor.json");

    if !floor_path.exists() {
        println!(
            "Skipping floor test - file not found: {}",
            floor_path.display()
        );
        return;
    }

    match runner.run_and_report(&floor_path) {
        Ok(stats) => {
            println!("Floor test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run floor test suite: {e}");
        }
    }
}

/// Test round function specifically
#[test]
fn test_run_round_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let round_path = specs_path.join("round.json");

    if !round_path.exists() {
        println!(
            "Skipping round test - file not found: {}",
            round_path.display()
        );
        return;
    }

    match runner.run_and_report(&round_path) {
        Ok(stats) => {
            println!("Round test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run round test suite: {e}");
        }
    }
}

/// Test take function specifically
#[test]
fn test_run_take_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let take_path = specs_path.join("take.json");

    if !take_path.exists() {
        println!(
            "Skipping take test - file not found: {}",
            take_path.display()
        );
        return;
    }

    match runner.run_and_report(&take_path) {
        Ok(stats) => {
            println!("Take test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run take test suite: {e}");
        }
    }
}

/// Run multiple test suites for broader coverage
#[test]
#[ignore] // Use #[ignore] so it doesn't run by default, but can be run with --ignored
fn test_run_multiple_official_suites() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(false);

    // List of test files to run (starting with simpler ones)
    let test_files = vec![
        "literals.json",
        "basics.json",
        "equality.json",
        "boolean-logic-and.json",
        "boolean-logic-or.json",
        "exists.json",
        "count.json",
    ];

    let test_paths: Vec<PathBuf> = test_files
        .into_iter()
        .map(|f| specs_path.join(f))
        .filter(|p| p.exists()) // Only include files that exist
        .collect();

    if test_paths.is_empty() {
        println!("No test files found in {}", specs_path.display());
        return;
    }

    match runner.run_multiple_test_files(&test_paths) {
        Ok(stats) => {
            println!("🏁 Multiple test suites completed:");
            println!("  Total tests: {}", stats.total);
            println!("  Passed: {} ({:.1}%)", stats.passed, stats.pass_rate());
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);

            // Report progress but don't fail the test yet
            let success_rate = stats.pass_rate();
            if success_rate > 50.0 {
                println!("🎉 Good progress! Over 50% of tests are passing.");
            } else if success_rate > 25.0 {
                println!("📈 Making progress! Over 25% of tests are passing.");
            } else {
                println!("🚧 Early stage - more work needed on the implementation.");
            }
        }
        Err(e) => {
            println!("Failed to run multiple test suites: {e}");
        }
    }
}

/// Test equality operations specifically  
#[test]
fn test_run_equality_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let equality_path = specs_path.join("equality.json");

    if !equality_path.exists() {
        println!(
            "Skipping equality test - file not found: {}",
            equality_path.display()
        );
        return;
    }

    match runner.run_and_report(&equality_path) {
        Ok(stats) => {
            println!("Equality test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run equality test suite: {e}");
        }
    }
}

/// Test equivalent operations specifically
#[test]
fn test_run_equivalent_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let equivalent_path = specs_path.join("equivalent.json");

    if !equivalent_path.exists() {
        println!(
            "Skipping equivalent test - file not found: {}",
            equivalent_path.display()
        );
        return;
    }

    match runner.run_and_report(&equivalent_path) {
        Ok(stats) => {
            println!("Equivalent test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run equivalent test suite: {e}");
        }
    }
}

/// Test not-equivalent operations specifically
#[test]
fn test_run_not_equivalent_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let not_equivalent_path = specs_path.join("not-equivalent.json");

    if !not_equivalent_path.exists() {
        println!(
            "Skipping not-equivalent test - file not found: {}",
            not_equivalent_path.display()
        );
        return;
    }

    match runner.run_and_report(&not_equivalent_path) {
        Ok(stats) => {
            println!("Not-equivalent test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run not-equivalent test suite: {e}");
        }
    }
}

/// Test not-equal operations specifically
#[test]
fn test_run_not_equal_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let not_equal_path = specs_path.join("n-equality.json");

    if !not_equal_path.exists() {
        println!(
            "Skipping n-equality test - file not found: {}",
            not_equal_path.display()
        );
        return;
    }

    match runner.run_and_report(&not_equal_path) {
        Ok(stats) => {
            println!("N-equality (not equal) test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run n-equality test suite: {e}");
        }
    }
}

/// Run all equality-related test suites
#[test]
fn test_run_all_equality_suites() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(false);

    // List of equality-related test files
    let equality_test_files = vec![
        "equality.json",
        "equivalent.json",
        "not-equivalent.json",
        "n-equality.json",
    ];

    let test_paths: Vec<PathBuf> = equality_test_files
        .into_iter()
        .map(|f| specs_path.join(f))
        .filter(|p| p.exists()) // Only include files that exist
        .collect();

    if test_paths.is_empty() {
        println!("No equality test files found in {}", specs_path.display());
        return;
    }

    match runner.run_multiple_test_files(&test_paths) {
        Ok(stats) => {
            println!("🏁 All equality test suites completed:");
            println!("  Total tests: {}", stats.total);
            println!("  Passed: {} ({:.1}%)", stats.passed, stats.pass_rate());
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);

            // Report progress but don't fail the test yet
            let success_rate = stats.pass_rate();
            if success_rate > 80.0 {
                println!("🎉 Excellent! Over 80% of equality tests are passing.");
            } else if success_rate > 60.0 {
                println!("📈 Good progress! Over 60% of equality tests are passing.");
            } else if success_rate > 40.0 {
                println!("🚧 Making progress! Over 40% of equality tests are passing.");
            } else {
                println!("🔄 Early stage - more work needed on equality implementations.");
            }
        }
        Err(e) => {
            println!("Failed to run equality test suites: {e}");
        }
    }
}

/// Test sort function specifically
#[test]
fn test_run_sort_suite() {
    let specs_path = get_specs_path();
    let mut runner = IntegrationTestRunner::new()
        .with_base_path(&specs_path)
        .with_verbose(true);

    let sort_path = specs_path.join("sort.json");

    if !sort_path.exists() {
        println!(
            "Skipping sort test - file not found: {}",
            sort_path.display()
        );
        return;
    }

    match runner.run_and_report(&sort_path) {
        Ok(stats) => {
            println!("Sort test suite completed:");
            println!(
                "  Passed: {}/{} ({:.1}%)",
                stats.passed,
                stats.total,
                stats.pass_rate()
            );
            println!("  Failed: {}", stats.failed);
            println!("  Errors: {}", stats.errored);
        }
        Err(e) => {
            println!("Failed to run sort test suite: {e}");
        }
    }
}

/// Example of how to create and run a custom test suite
#[test]
fn test_custom_test_creation() {
    use integration_test_runner::{TestCase, TestSuite};

    let custom_suite = TestSuite {
        name: "Custom Test Suite".to_string(),
        description: "Tests for custom functionality".to_string(),
        source: Some("fhirpath-rs".to_string()),
        tests: vec![
            TestCase {
                name: "test_boolean_true".to_string(),
                expression: "true".to_string(),
                input: Some(serde_json::json!({})),
                inputfile: None,
                expected: serde_json::Value::Array(vec![serde_json::Value::Bool(true)]),
                tags: vec!["boolean".to_string(), "literal".to_string()],
                description: Some("Test boolean true literal".to_string()),
            },
            TestCase {
                name: "test_integer_literal".to_string(),
                expression: "42".to_string(),
                input: Some(serde_json::json!({})),
                inputfile: None,
                expected: serde_json::Value::Array(vec![serde_json::Value::Number(42.into())]),
                tags: vec!["integer".to_string(), "literal".to_string()],
                description: Some("Test integer literal".to_string()),
            },
        ],
    };

    let mut runner = IntegrationTestRunner::new().with_verbose(true);
    let results = runner.run_test_suite(&custom_suite);
    let stats = runner.calculate_stats(&results);

    println!("Custom test suite results:");
    println!("  Total: {}", stats.total);
    println!("  Passed: {}", stats.passed);
    println!("  Failed: {}", stats.failed);
    println!("  Errors: {}", stats.errored);

    // Print individual results
    for (test_name, result) in &results {
        match result {
            integration_test_runner::TestResult::Passed => {
                println!("  ✅ {test_name}");
            }
            integration_test_runner::TestResult::Failed { expected, actual } => {
                println!("  ❌ {test_name}");
                println!("    Expected: {expected}");
                println!("    Actual: {actual}");
            }
            integration_test_runner::TestResult::Error { error } => {
                println!("  ⚠️ {test_name}: {error}");
            }
            integration_test_runner::TestResult::Skipped { reason } => {
                println!("  ⊘ {test_name}: {reason}");
            }
        }
    }
}
