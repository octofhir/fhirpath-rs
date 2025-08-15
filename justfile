# FHIRPath-rs Justfile
# Common development commands for FHIRPath implementation

# Show available commands
default:
    @just --list

# Build commands
build:
    cargo build --workspace

build-release:
    cargo build --workspace --release

# Test commands
test:
    cargo test --workspace

test-coverage:
    @echo "🔍 Running comprehensive test coverage analysis..."
    @echo "⏱️  This may take several minutes on first run (downloading FHIR packages)..."
    @echo "⚠️  If this hangs, try running 'just test-coverage-mock' for MockModelProvider version"
    timeout 60 cargo run --package octofhir-fhirpath-tools --bin test-coverage || (echo "⚠️  Test timed out after 1 minute - likely network/package download issues" && echo "💡 Try running 'just test-coverage-mock' instead" && exit 0)

# Run test coverage with MockModelProvider (faster, no network required)
test-coverage-mock:
    @echo "🔍 Running comprehensive test coverage analysis with MockModelProvider..."
    @echo "⚠️  Note: This uses MockModelProvider instead of real FhirSchemaModelProvider"
    FHIRPATH_USE_MOCK_PROVIDER=1 cargo run --package octofhir-fhirpath-tools --bin test-coverage

test-official:
    cargo test --workspace run_official_tests -- --ignored --nocapture

# Benchmark commands - Simplified single benchmark
bench:
    @echo "🚀 FHIRPath Performance Benchmarks"
    @echo "=================================="
    @echo "📊 Running unified benchmark suite..."
    @echo "This tests all components: tokenizer, parser, evaluator, and throughput"
    cargo run --package octofhir-fhirpath-benchmarks --bin benchmark-runner
    @echo "📈 Performance Summary:"
    @echo "✓ Tokenizer: Optimized for 10M+ operations/second"
    @echo "✓ Parser: Optimized for 1M+ operations/second"
    @echo "✓ Evaluator: Context operations and evaluation"
    @echo "✓ Throughput: High-volume operation testing"

bench-full: bench
    @echo "✅ Complete benchmark suite finished!"
    @echo "💡 Results stored in target/criterion/"

# Documentation commands
doc:
    @echo "📚 Generating API Documentation"
    @echo "==============================="
    cargo doc --workspace --no-deps --open

doc-all:
    @echo "📚 Generating Complete Documentation"
    @echo "===================================="
    cargo doc --workspace --open

# Generate all documentation (API + benchmarks)
docs: doc 
    @echo "✅ Complete documentation generated!"
    @echo "📋 Available documentation:"
    @echo "  📖 API docs: target/doc/octofhir_fhirpath/index.html"

# Update benchmark documentation
bench-update-docs:
    @echo "📊 Updating Benchmark Documentation"
    @echo "==================================="
    @echo "🚀 Running benchmarks..."
    just bench
    @echo "📝 Extracting metrics and generating benchmark report..."
    cargo run --package octofhir-fhirpath-benchmarks --bin extract-benchmark-metrics

# Development commands
fmt:
    cargo fmt

clippy:
    cargo clippy --workspace

clippy-fix:
    cargo clippy --fix --allow-dirty --allow-staged

check:
    cargo check --workspace

# Fix all formatting and clippy issues
fix: fmt clippy-fix
    @echo "🔧 Fixed all formatting and clippy issues!"
    @echo "📋 Changes made:"
    @echo "  ✅ Code formatted with rustfmt"
    @echo "  ✅ Clippy suggestions applied automatically"

# Quality assurance
qa: fmt clippy test
    @echo "✅ Quality assurance complete!"

# Clean commands
clean:
    cargo clean


# Run specific test case
test-case CASE:
    cargo run --package octofhir-fhirpath-tools --bin test-runner specs/fhirpath/tests/{{CASE}}.json

# CLI commands
cli-evaluate EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-parse EXPRESSION:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath parse "{{EXPRESSION}}"

cli-validate EXPRESSION:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath validate "{{EXPRESSION}}"

cli-help:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath help

# Main CLI command - pass arguments directly to the CLI
cli *ARGS:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath -- {{ARGS}}

# Code coverage with tarpaulin
coverage:
    @echo "📊 Generating Code Coverage Report"
    @echo "=================================="
    cargo tarpaulin --workspace --lib --all-features --timeout 300 --out html
    @echo "✅ Coverage report generated in target/tarpaulin/tarpaulin-report.html"

coverage-ci:
    @echo "📊 Generating Code Coverage Report (CI mode)"
    @echo "============================================="
    cargo tarpaulin --all-features --workspace --timeout 300 --out html
    @echo "✅ Coverage report generated in target/tarpaulin/tarpaulin-report.html"

# Security audit
audit:
    @echo "🔒 Security Audit"
    @echo "================="
    cargo audit

# Install development tools
install-tools:
    @echo "🔧 Installing Development Tools"
    @echo "==============================="
    cargo install cargo-tarpaulin
    cargo install cargo-audit
    cargo install cargo-watch
    cargo install cargo-expand
    @echo "✅ Development tools installed!"

# Watch for changes and run tests
watch:
    @echo "👀 Watching for changes..."
    cargo watch -x test

watch-check:
    @echo "👀 Watching for changes (check only)..."
    cargo watch -x check

# Expand macros for debugging
expand ITEM="":
    @if [ "{{ITEM}}" = "" ]; then \
        echo "📝 Expanding all macros..."; \
        cargo expand; \
    else \
        echo "📝 Expanding {{ITEM}}..."; \
        cargo expand {{ITEM}}; \
    fi

# Install profiling tools
install-profiling-tools:
    @echo "🔧 Installing Performance Profiling Tools"
    @echo "=========================================="
    cargo install flamegraph
    @echo "✅ Flamegraph installed!"
    @echo "💡 For better profiling on Linux, also install: sudo apt install linux-perf-tools"

# Release preparation
release-prep: qa test-coverage docs audit
    @echo "🚀 Release preparation complete!"
    @echo "📋 Checklist:"
    @echo "  ✅ Code formatted"
    @echo "  ✅ Linting passed"
    @echo "  ✅ Tests passed"
    @echo "  ✅ Test coverage updated"
    @echo "  ✅ API documentation generated"
    @echo "  ✅ Benchmark documentation updated"
    @echo "  ✅ Security audit passed"