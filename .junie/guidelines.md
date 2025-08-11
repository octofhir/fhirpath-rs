# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a FHIRPath implementation in Rust as a single consolidated crate with modular structure:

- **src/ast/**: Abstract syntax tree definitions and visitor pattern
- **src/parser/**: Tokenizer and parser using nom library (version 8)
- **src/evaluator/**: Expression evaluation engine with context management
- **src/registry/**: Function registry and built-in function implementations
- **src/model/**: Value types, resources, and FHIR data model
- **src/diagnostics/**: Error handling and diagnostic reporting
- **src/engine.rs**: Main evaluation engine
- **src/error.rs**: Core error types
- **src/types.rs**: Core type definitions

## Development Commands

### Building and Testing
```bash
# Build entire workspace
just build

# Build with release optimization  
just build-release

# Run all tests
just test

# Run integration tests (official FHIRPath test suites)
just test-official

# Update test coverage report
just test-coverage

# Run benchmarks
just bench

# Run full benchmark suite
just bench-full

# Update benchmark documentation
just bench-update-docs
```

### Performance and Quality
```bash
# Run clippy linting
just clippy

# Format code
just fmt

# Check code without building
just check

# Quality assurance (format + lint + test)
just qa

# Run specific benchmarks
just bench-tokenizer
just bench-parser

# Clean build artifacts
just clean
just clean-bench
```

### Test-Specific Commands
```bash
# Run specific test case
just test-case test-case-name

# Run failed expression tests
just test-failed

# Release preparation (full QA + docs)
just release-prep
```

### Documentation Commands
```bash
# Generate API documentation
just doc

# Generate complete documentation (including dependencies)
just doc-all

# Generate all documentation (API + benchmarks)
just docs
```

### CLI Commands
```bash
# Evaluate FHIRPath expression (read FHIR resource from stdin)
just cli-evaluate "Patient.name.given"

# Evaluate FHIRPath expression with specific file
just cli-evaluate "Patient.name.given" path/to/resource.json

# Parse FHIRPath expression to AST
just cli-parse "Patient.name.given"

# Validate FHIRPath expression syntax
just cli-validate "Patient.name.given"

# Show CLI help
just cli-help

# Run CLI with custom arguments
just cli [arguments...]
```

### Debug Utilities
For debugging parser errors, create simple tests instead of dedicated binaries. 
The project includes utilities for troubleshooting:
```bash
# Run specific test case from official FHIRPath test suite
just test-case test-case-name

# Example: just test-case literals
# This runs specs/fhirpath/tests/literals.json
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
just test-coverage
```

This command:
- Builds the test infrastructure 
- Runs all official FHIRPath test suites
- Generates a comprehensive report in `TEST_COVERAGE.md`
- Provides statistics on pass rates and identifies missing functionality

The coverage report should be updated after completing any major functionality to track progress.

## Library Usage

The consolidated crate provides a clean API:

```rust
use fhirpath::{FhirPathEngine, FhirPathValue, model::MockModelProvider};
use std::sync::Arc;

let provider = Arc::new(MockModelProvider::new());
let engine = FhirPathEngine::new(provider);
let result = engine.evaluate("Patient.name.given", &patient_resource)?;
```

Main exports:
- `FhirPathEngine`: Main evaluation engine
- `FhirPathValue`: Value types
- `parse()`: Parse FHIRPath expressions
- `FunctionRegistry`: Function registry for extensions
