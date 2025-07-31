//! Command-line tool for running FHIRPath official test suites
//!
//! This binary provides a convenient way to run the official FHIRPath test suites
//! against our implementation from the command line.

use std::env;
use std::path::PathBuf;
use std::process;

// Include the integration test runner module
#[path = "../../tests/integration_test_runner.rs"]
mod integration_test_runner;

use integration_test_runner::IntegrationTestRunner;

fn print_usage() {
    println!("FHIRPath Test Runner");
    println!();
    println!("USAGE:");
    println!("    test_runner [OPTIONS] <TEST_FILE_OR_DIRECTORY>");
    println!();
    println!("ARGS:");
    println!(
        "    <TEST_FILE_OR_DIRECTORY>    Path to JSON test file or directory containing tests"
    );
    println!();
    println!("OPTIONS:");
    println!("    -v, --verbose               Enable verbose output");
    println!("    -h, --help                  Print help information");
    println!("    --base-path <PATH>          Set base path for resolving input files");
    println!();
    println!("EXAMPLES:");
    println!("    test_runner specs/fhirpath/tests/basics.json");
    println!("    test_runner --verbose specs/fhirpath/tests/literals.json");
    println!("    test_runner --base-path specs/fhirpath/tests basics.json");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut verbose = false;
    let mut base_path: Option<PathBuf> = None;
    let mut test_path: Option<PathBuf> = None;
    let mut i = 1;

    // Parse command line arguments
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--verbose" => {
                verbose = true;
                i += 1;
            }
            "--base-path" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --base-path requires a value");
                    process::exit(1);
                }
                base_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown option '{}'", arg);
                print_usage();
                process::exit(1);
            }
            _ => {
                if test_path.is_some() {
                    eprintln!("Error: Multiple test paths specified");
                    process::exit(1);
                }
                test_path = Some(PathBuf::from(&args[i]));
                i += 1;
            }
        }
    }

    let test_path = match test_path {
        Some(path) => path,
        None => {
            eprintln!("Error: No test file or directory specified");
            process::exit(1);
        }
    };

    // Create test runner
    let mut runner = IntegrationTestRunner::new().with_verbose(verbose);
    if let Some(base) = base_path {
        runner = runner.with_base_path(base);
    }

    // Check if test path exists
    // if !test_path.exists() {
    //     eprintln!("Error: Test path does not exist: {}", test_path.display());
    //     process::exit(1);
    // }

    // Run tests
    println!("🚀 FHIRPath Test Runner");
    println!("📁 Test path: {}", test_path.display());
    if verbose {
        println!("🔊 Verbose mode enabled");
    }
    println!();

    let result = if test_path.is_file() {
        // Single test file
        runner.run_and_report(&test_path)
    } else if test_path.is_dir() {
        // Directory - find all JSON files
        match find_test_files(&test_path) {
            Ok(test_files) => {
                if test_files.is_empty() {
                    eprintln!("No JSON test files found in: {}", test_path.display());
                    process::exit(1);
                }
                runner.run_multiple_test_files(&test_files)
            }
            Err(e) => {
                eprintln!("Error scanning directory: {}", e);
                process::exit(1);
            }
        }
    } else {
        eprintln!(
            "Error: Path is neither a file nor a directory: {}",
            test_path.display()
        );
        process::exit(1);
    };

    match result {
        Ok(stats) => {
            println!();
            if stats.failed > 0 || stats.errored > 0 {
                println!("❌ Some tests failed or had errors.");
                process::exit(1);
            } else {
                println!("✅ All tests passed!");
                process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("Error running tests: {}", e);
            process::exit(1);
        }
    }
}

/// Find all JSON test files in a directory
fn find_test_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut test_files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "json" {
                    test_files.push(path);
                }
            }
        }
    }

    // Sort for consistent ordering
    test_files.sort();
    Ok(test_files)
}
