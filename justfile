# FHIRPath-rs Justfile
# Common development commands for FHIRPath implementation

# Show available commands
default:
    @just --list

# Build commands
build:
    cargo build

build-release:
    cargo build --release

# Test commands
test:
    cargo test

test-coverage:
    ./scripts/update-test-coverage.sh

test-official:
    cd fhirpath-core && cargo test run_official_tests -- --ignored --nocapture

test-failed:
    cd fhirpath-core && cargo test failed_expressions_tests -- --nocapture

# Benchmark commands - Simplified 3-component focus
bench:
    @echo "🚀 Running Core Performance Benchmark (all 3 components)"
    cargo bench --bench core_performance_benchmark

bench-full:
    @echo "🚀 Running All Individual Component Benchmarks"
    @echo "📝 Tokenizer:"
    cargo bench --bench tokenizer_only_benchmark
    @echo "📝 Parser:"
    cargo bench --bench parser_benchmark
    @echo "📝 Evaluator:"
    cargo bench --bench evaluation_context_benchmark

bench-tokenizer:
    @echo "📝 Running Tokenizer Benchmark"
    cargo bench --bench tokenizer_only_benchmark

bench-parser:
    @echo "📝 Running Parser Benchmark" 
    cargo bench --bench parser_benchmark

bench-evaluator:
    @echo "📝 Running Evaluator Benchmark"
    cargo bench --bench evaluation_context_benchmark

# Development commands
fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

# Clean commands
clean:
    cargo clean

clean-bench:
    rm -rf target/criterion

# Run specific test case
test-case CASE:
    cargo run -p fhirpath-core specs/fhirpath/tests/{{CASE}}.json

# Development server
server:
    cargo run --bin benchmark_server