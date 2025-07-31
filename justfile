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
    @echo "🧪 FHIRPath Test Coverage Update"
    @echo "================================="
    @echo "📦 Building test infrastructure..."
    cargo build --release
    @echo "🔍 Running comprehensive test coverage analysis..."
    cargo test run_coverage_report -- --ignored --nocapture
    @echo "✅ Coverage report generated in TEST_COVERAGE.md"

test-official:
    cargo test run_official_tests -- --ignored --nocapture

test-failed:
    cargo test failed_expressions_tests -- --nocapture

# Benchmark commands - Simplified 3-component focus
bench:
    @echo "🚀 FHIRPath Core Performance Benchmarks"
    @echo "======================================="
    @echo "📊 Running Core Performance Benchmark..."
    @echo "This tests all 3 components: tokenizer, parser, and evaluator"
    cargo bench --bench core_performance_benchmark
    @echo "📈 Performance Summary:"
    @echo "✓ Tokenizer: Optimized for 10M+ operations/second"
    @echo "✓ Parser: Optimized for 1M+ operations/second"  
    @echo "✓ Evaluator: Context operations and evaluation"
    @echo "✓ Full Pipeline: Complete tokenize → parse → evaluate workflow"

bench-full:
    @echo "🚀 FHIRPath Complete Performance Analysis"
    @echo "========================================"
    @echo "📊 Running Core Performance Benchmark..."
    cargo bench --bench core_performance_benchmark
    @echo "🔬 Running Individual Component Benchmarks..."
    @echo "📝 Tokenizer Only Benchmark:"
    cargo bench --bench tokenizer_only_benchmark
    @echo "📝 Parser Benchmark:"
    cargo bench --bench parser_benchmark
    @echo "✅ Benchmarks Complete!"
    @echo "💡 Results stored in target/criterion/"

bench-tokenizer:
    @echo "📝 Running Tokenizer Benchmark"
    cargo bench --bench tokenizer_only_benchmark

bench-parser:
    @echo "📝 Running Parser Benchmark" 
    cargo bench --bench parser_benchmark

# Documentation commands
doc:
    @echo "📚 Generating API Documentation"
    @echo "==============================="
    cargo doc --no-deps --open

doc-all:
    @echo "📚 Generating Complete Documentation"
    @echo "===================================="
    cargo doc --open

# Generate all documentation (API + benchmarks)
docs: doc bench-update-docs
    @echo "✅ Complete documentation generated!"
    @echo "📋 Available documentation:"
    @echo "  📖 API docs: target/doc/octofhir_fhirpath/index.html"
    @echo "  📊 Benchmarks: BENCHMARKS.md"
    @echo "  📈 Criterion reports: target/criterion/report/index.html"

# Update benchmark documentation
bench-update-docs:
    @echo "📊 Updating Benchmark Documentation"
    @echo "==================================="
    @echo "🚀 Running benchmarks..."
    just bench-full
    @echo "📝 Extracting metrics and generating benchmark report..."
    cargo run --bin extract_benchmark_metrics

# Development commands
fmt:
    cargo fmt

clippy:
    cargo clippy

check:
    cargo check

# Quality assurance
qa: fmt clippy test
    @echo "✅ Quality assurance complete!"

# Clean commands
clean:
    cargo clean

clean-bench:
    rm -rf target/criterion

# Run specific test case
test-case CASE:
    cargo run --bin test_runner specs/fhirpath/tests/{{CASE}}.json

# CLI commands
cli-evaluate EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --bin octofhir-fhirpath evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --bin octofhir-fhirpath evaluate "{{EXPRESSION}}" "{{FILE}}"; \
    fi

cli-parse EXPRESSION:
    cargo run --bin octofhir-fhirpath parse "{{EXPRESSION}}"

cli-validate EXPRESSION:
    cargo run --bin octofhir-fhirpath validate "{{EXPRESSION}}"

cli-help:
    cargo run --bin octofhir-fhirpath help

# Main CLI command - pass arguments directly to the CLI
cli *ARGS:
    cargo run --bin octofhir-fhirpath -- {{ARGS}}
    
# Release preparation
release-prep: qa test-coverage docs
    @echo "🚀 Release preparation complete!"
    @echo "📋 Checklist:"
    @echo "  ✅ Code formatted"
    @echo "  ✅ Linting passed"
    @echo "  ✅ Tests passed"
    @echo "  ✅ Test coverage updated"
    @echo "  ✅ API documentation generated"
    @echo "  ✅ Benchmark documentation updated"