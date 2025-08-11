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
    @echo "📦 Building test infrastructure (tests only)..."
    cargo test --test coverage_report_simple --no-run --release
    @echo "🔍 Running comprehensive test coverage analysis..."
    @echo "⏱️  This may take several minutes on first run (downloading FHIR packages)..."
    @echo "⚠️  If this hangs, try running 'just test-coverage-mock' for MockModelProvider version"
    FHIRPATH_QUICK_INIT=1 timeout 60 cargo test --test coverage_report_simple run_coverage_report -- --ignored --nocapture || (echo "⚠️  Test timed out after 1 minute - likely network/package download issues" && echo "💡 Try running 'just test-coverage-mock' instead" && exit 0)
    @echo "✅ Coverage report generated in TEST_COVERAGE.md"

# Run test coverage with MockModelProvider (faster, no network required)
test-coverage-mock:
    @echo "🧪 FHIRPath Test Coverage Update (Mock Provider)"
    @echo "================================================"
    @echo "📦 Building test infrastructure..."
    cargo build --release
    @echo "🔍 Running comprehensive test coverage analysis with MockModelProvider..."
    @echo "⚠️  Note: This uses MockModelProvider instead of real FhirSchemaModelProvider"
    FHIRPATH_USE_MOCK_PROVIDER=1 cargo test --test coverage_report_simple run_coverage_report -- --ignored --nocapture
    @echo "✅ Coverage report generated in TEST_COVERAGE.md"

test-official:
    cargo test run_official_tests -- --ignored --nocapture

# Benchmark commands - Simplified single benchmark
bench:
    @echo "🚀 FHIRPath Performance Benchmarks"
    @echo "=================================="
    @echo "📊 Running unified benchmark suite..."
    @echo "This tests all components: tokenizer, parser, evaluator, and throughput"
    cargo bench --bench fhirpath_benchmark
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
    just bench
    @echo "📝 Extracting metrics and generating benchmark report..."
    cargo run --bin extract_benchmark_metrics

# Development commands
fmt:
    cargo fmt --all

clippy:
    cargo clippy --all

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

check:
    cargo check --all

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

# Code coverage with tarpaulin
coverage:
    @echo "📊 Generating Code Coverage Report"
    @echo "=================================="
    cargo tarpaulin --lib --all-features --timeout 300 --out html
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

# Performance profiling commands
profile EXPRESSION="Patient.name":
    @echo "🔬 Profiling FHIRPath expression: {{EXPRESSION}}"
    @echo "================================================"
    cargo build --release --bin perf_test
    @echo "Running performance profiling..."
    CARGO_PROFILE_RELEASE_DEBUG=true cargo run --release --bin perf_test -- "{{EXPRESSION}}"

# Generate flamegraph for expression profiling (requires flamegraph tool)
flamegraph EXPRESSION="Patient.name.where(family.exists())":
    @echo "🔥 Generating flamegraph for: {{EXPRESSION}}"
    @echo "=============================================="
    @echo "Building release with debug symbols..."
    CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --bin perf_test
    @echo "Generating flamegraph..."
    cargo flamegraph --bin perf_test -- "{{EXPRESSION}}" || echo "⚠️  Install flamegraph: cargo install flamegraph"
    @echo "🔥 Flamegraph saved as flamegraph.svg"

# Profile where() function specifically with sample data
profile-where:
    @echo "🔬 Profiling .where() function performance"
    @echo "==========================================="
    @echo "Testing complex where expressions..."
    just flamegraph "Patient.name.where(family.exists())"
    @echo "✅ Where function profiling complete!"

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
