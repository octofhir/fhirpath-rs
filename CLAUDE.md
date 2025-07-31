# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a modular FHIRPath implementation in Rust using a workspace structure:

- **fhirpath-core**: Main library crate that re-exports and orchestrates all components
- **fhirpath-parser**: Tokenizer and parser using nom library (version 8)
- **fhirpath-ast**: Abstract syntax tree definitions and visitor pattern
- **fhirpath-evaluator**: Expression evaluation engine with context management
- **fhirpath-registry**: Function registry and built-in function implementations
- **fhirpath-model**: Value types, resources, and FHIR data model
- **fhirpath-diagnostics**: Error handling and diagnostic reporting

## Development Commands

### Building and Testing
```bash
# Build entire workspace
cargo build

# Build with release optimization  
cargo build --release

# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run integration tests (official FHIRPath test suites)
cd fhirpath-core && cargo test run_official_tests -- --ignored --nocapture

# Update test coverage report
./scripts/update-test-coverage.sh

# Run benchmarks
cargo bench
```

For debuggin parser error create simple test instead of dedicated binary 

Run specifigc json test case 
cargo run -p fhirpath-core specs/fhirpath/tests/{test-case-name}.json

### Performance and Quality
```bash
# Run clippy linting
cargo clippy

# Format code
cargo fmt

# Run specific benchmark
cargo bench tokenizer_only_benchmark
cargo bench parser_benchmark
```

### Debug Utilities
The project includes debug binaries for troubleshooting specific issues:
```bash
cargo run --bin debug_simple_variable
cargo run --bin debug_lambda_variables
cargo run --bin debug_context_flow
# See Cargo.toml for full list of debug binaries
```

## Guidelines

Apply the following guidelines when developing fhirpath-core:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Coding Guidelines](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Style Guide](https://rust-lang.github.io/rust-style-guide/)

## Specifications and Dependencies

- FHIRPath specification reference in `specs/` folder
- Official test cases in `specs/fhirpath/tests/` 
- FHIRSchema spec: https://fhir-schema.github.io/fhir-schema/intro.html
- Uses nom library version 8 for parsing
- For UCUM units: use https://github.com/octofhir/ucum-rs or local path `./…/ucum-rs`
- Criterion version 0.7


## Architecture Decision Records (ADRs)

Before implementing major features:
1. Create ADR following: https://github.com/joelparkerhenderson/architecture-decision-record
2. Split implementation into phases/tasks stored in `tasks/` directory  
3. Update task files with implementation status

## Planing Phase

For every ADR implementation split record into phases/tasks and store in `tasks` directory. Maintain a specific task file when working on it. Before starting on the first task, create all tasks for future use. After implementing features from a task file update it status
For debugging cases create a simple test inside the test directory and delete it after resolving the issue


## Task executing phase
Update task file for aligh with implemented features


## Test Coverage

To track progress and maintain visibility into implementation completeness:

### Updating Test Coverage Report
Run the automated test coverage generator:
```bash
./scripts/update-test-coverage.sh
```

This script:
- Builds the test infrastructure 
- Runs all official FHIRPath test suites
- Generates a comprehensive report in `fhirpath-core/TEST_COVERAGE.md`
- Provides statistics on pass rates and identifies missing functionality

The coverage report should be updated after completing any major functionality to track progress.
