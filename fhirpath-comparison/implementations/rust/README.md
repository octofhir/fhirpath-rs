# Rust FHIRPath Implementation for Comparison

This directory contains the Rust implementation of the FHIRPath test runner for the comparison framework.

## Structure

- `src/main.rs` - Entry point for the test runner binary
- `src/test_runner.rs` - Implementation of the test runner logic
- `Cargo.toml` - Rust project configuration

## Running Tests

To run the tests:

```bash
cargo run -- test
```

To run benchmarks:

```bash
cargo run -- benchmark
```

To run both tests and benchmarks:

```bash
cargo run
```

## Implementation Notes

The test runner uses the `fhirpath-core` crate from the parent project to evaluate FHIRPath expressions against FHIR resources. It loads test cases from the official FHIRPath XML test suite located in `fhirpath-core/tests/official-tests/r4/tests-fhir-r4.xml` and processes all 694+ test cases. Results are output in a standardized JSON format for comparison with other implementations.

## Recent Changes

- Updated to use official FHIRPath XML test suite from `fhirpath-core/tests/official-tests/r4/`
- Now processes 694+ official test cases instead of custom test configuration
- Restructured the code to follow Rust conventions with a proper `main.rs` file
- Moved the main function from `test_runner.rs` to `main.rs`
- Made result structs public to allow proper module access
- Updated `Cargo.toml` to point to the new entry point
