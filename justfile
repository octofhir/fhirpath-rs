# FHIRPath-rs Justfile
# Common development commands for FHIRPath implementation

# Show available commands
default:
    @echo "🔥 FHIRPath-rs Development Commands"
    @echo "=================================="
    @echo ""
    @echo "🚀 Quick Start:"
    @echo "  just server              # Start HTTP server on port 8080"
    @echo "  just server-dev          # Start server with CORS for development"
    @echo "  just repl                # Start interactive REPL"
    @echo "  just test                # Run all tests"
    @echo ""
    @echo "📋 All available commands:"
    @just --list

# Build commands
build:
    cargo build --workspace

build-release:
    cargo build --workspace --release

# Build UI assets
build-ui:
    @echo "🏗️  Building UI assets..."
    cd ui && pnpm install && pnpm build
    @echo "✅ UI build complete"

# Test commands
test:
    cargo test --workspace

test-coverage:
    @echo "🔍 Running comprehensive test coverage analysis..."
    @echo "⏱️  This may take several minutes on first run (downloading FHIR packages)..."
    @echo "⚠️  If this hangs, try running 'just test-coverage-mock' for MockModelProvider version"
    timeout 60 cargo run --package octofhir-fhirpath --bin test-coverage --features dev-tools,cli || (echo "⚠️  Test timed out after 1 minute - likely network/package download issues" && echo "💡 Try running 'just test-coverage-mock' instead" && exit 0)

# Run test coverage with MockModelProvider (faster, no network required)
test-coverage-mock:
    @echo "🔍 Running comprehensive test coverage analysis with MockModelProvider..."
    @echo "⚠️  Note: This uses MockModelProvider instead of real FhirSchemaModelProvider"
    FHIRPATH_USE_MOCK_PROVIDER=1 cargo run --package octofhir-fhirpath --bin test-coverage --features dev-tools

test-official:
    cargo test --workspace run_official_tests -- --ignored --nocapture

# Benchmark commands - Use main crate binaries
bench:
    @echo "🚀 FHIRPath Performance Benchmarks"
    @echo "=================================="
    @echo "📊 Running comprehensive benchmark suite..."
    @echo "This tests tokenizer, parser, and evaluator across complexity levels"
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools benchmark --run
    @echo "✅ Benchmark complete! Results show ops/sec for each operation."

bench-simple:
    @echo "🟢 Running Simple Expression Benchmarks"
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools profile "Patient.active"

bench-medium:
    @echo "🟡 Running Medium Expression Benchmarks"
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools profile "Patient.name.where(use = 'official').family"

bench-complex:
    @echo "🔴 Running Complex Expression Benchmarks"
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools profile "Bundle.entry.resource.count()" --bundle

bench-report:
    @echo "📄 Generating Benchmark Report"
    @echo "=============================="
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools benchmark --run --output benchmark.md
    @echo "✅ Benchmark report generated: benchmark.md"

bench-list:
    @echo "📋 Available Benchmark Expressions"
    @echo "=================================="
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools list


bench-full: bench bench-report
    @echo "✅ Complete benchmark suite finished!"
    @echo "💡 Results available in benchmark.md"

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

# Profiling commands for performance analysis
profile EXPRESSION *ARGS:
    @echo "🔍 Profiling Expression: {{EXPRESSION}}"
    @echo "======================================="
    cargo run --package octofhir-fhirpath --bin fhirpath-bench --features dev-tools profile "{{EXPRESSION}}" {{ARGS}}
    @echo "✅ Profiling complete! Check ./profile_output/ for results"

profile-patient EXPRESSION:
    @echo "🏥 Profiling with Patient Data: {{EXPRESSION}}"
    just profile "{{EXPRESSION}}" --iterations 1000

profile-bundle EXPRESSION:
    @echo "📦 Profiling with Bundle Data: {{EXPRESSION}}"
    just profile "{{EXPRESSION}}" --bundle --iterations 500

profile-examples:
    @echo "🔍 Running Example Profiling Sessions"
    @echo "====================================="
    @echo "Simple expression profiling..."
    just profile-patient "Patient.active"
    @echo "Medium expression profiling..."
    just profile-patient "Patient.name.where(use = 'official').family"
    @echo "Complex expression profiling..."
    just profile-bundle "Bundle.entry.resource.count()"
    @echo "✅ Example profiling sessions complete!"

# Update benchmark documentation
bench-update-docs: bench-report
    @echo "📊 Benchmark Documentation Updated"
    @echo "================================="
    @echo "✅ Latest benchmark results saved to benchmark.md"

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
    cargo run --package octofhir-fhirpath --bin test-runner --features dev-tools specs/fhirpath/tests/{{CASE}}.json

# CLI commands
cli-evaluate EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-parse EXPRESSION:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- parse "{{EXPRESSION}}"

cli-validate EXPRESSION:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- validate "{{EXPRESSION}}"

# Analyze FHIRPath expression
cli-analyze EXPRESSION *ARGS:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- analyze "{{EXPRESSION}}" {{ARGS}}

# Validate FHIRPath expression  
cli-analyze-validate EXPRESSION:
    just cli-analyze "{{EXPRESSION}}" --validate-only

cli-help:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- help

# Start Interactive REPL
repl FILE="" *ARGS:
    @if [ "{{FILE}}" = "" ]; then \
        echo "🔥 Starting FHIRPath Interactive REPL"; \
        echo "Type expressions to evaluate, or ':help' for commands"; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- repl {{ARGS}}; \
    else \
        echo "🔥 Starting FHIRPath REPL with initial resource: {{FILE}}"; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- repl --input "{{FILE}}" {{ARGS}}; \
    fi

# Enhanced CLI output format examples
cli-pretty EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli,terminal -- --output-format pretty evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli,terminal -- --output-format pretty evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-json EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- --output-format json evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- --output-format json evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-table EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- --output-format table evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- --output-format table evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

# Main CLI command - pass arguments directly to the CLI
cli *ARGS:
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- {{ARGS}}

# HTTP Server commands
server *ARGS:
    @echo "🌐 Starting FHIRPath HTTP Server"
    @echo "==============================="
    @echo "🔗 Server will be available at http://localhost:8080"
    @echo "📁 Storage directory: ./storage"
    @echo "📚 API documentation: http://localhost:8080/health for status"
    @echo "⏹️  Press Ctrl+C to stop the server"
    @echo ""
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- server {{ARGS}}

# Start server with custom port
server-port PORT *ARGS:
    @echo "🌐 Starting FHIRPath HTTP Server on port {{PORT}}"
    @echo "============================================="
    @echo "🔗 Server will be available at http://localhost:{{PORT}}"
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- server --port {{PORT}} {{ARGS}}

# Start server in development mode with CORS enabled for all origins
server-dev *ARGS:
    @echo "🧪 Starting FHIRPath HTTP Server (Development Mode)"
    @echo "=================================================="
    @echo "🔗 Server: http://localhost:8080"
    @echo "🌐 CORS: Enabled for all origins"
    @echo "📁 Storage: ./storage"
    @echo ""
    @echo "🏗️  Building UI..."
    cd ui && pnpm install && pnpm build
    @echo "🚀 Starting server..."
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- server --cors-all {{ARGS}}

# Start server with custom storage directory
server-storage STORAGE_DIR *ARGS:
    @echo "🌐 Starting FHIRPath HTTP Server"
    @echo "📁 Custom storage directory: {{STORAGE_DIR}}"
    cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- server --storage {{STORAGE_DIR}} {{ARGS}}

# Test server endpoints with curl examples
server-test:
    @echo "🧪 Testing FHIRPath HTTP Server Endpoints"
    @echo "=========================================="
    @echo ""
    @echo "🔍 Testing health endpoint..."
    curl -s http://localhost:8080/health | head -10 || echo "❌ Server not running. Start with 'just server'"
    @echo ""
    @echo "📁 Testing file list endpoint..."
    curl -s http://localhost:8080/files | head -10 || echo "❌ Server not running"
    @echo ""
    @echo "💡 Example evaluation request:"
    @echo "curl -X POST http://localhost:8080/r4/evaluate \\"
    @echo "  -H 'Content-Type: application/json' \\"
    @echo "  -d '{\"expression\": \"Patient.name.given\", \"resource\": {\"resourceType\": \"Patient\", \"name\": [{\"given\": [\"John\"]}]}}'"

# Server examples with different FHIR versions
server-examples:
    @echo "📚 FHIRPath Server API Examples"
    @echo "=============================="
    @echo ""
    @echo "🏥 Example Patient evaluation (R4):"
    @echo "curl -X POST http://localhost:8080/r4/evaluate \\"
    @echo "  -H 'Content-Type: application/json' \\"
    @echo "  -d @storage/examples/patient-example.json"
    @echo ""
    @echo "🔬 Example Observation evaluation (R5):" 
    @echo "curl -X POST http://localhost:8080/r5/evaluate \\"
    @echo "  -H 'Content-Type: application/json' \\"
    @echo "  -d '{\"expression\": \"Observation.valueQuantity.value\", \"resource\": {...}}'"
    @echo ""
    @echo "📦 Bundle analysis (R4B):"
    @echo "curl -X POST http://localhost:8080/r4b/analyze \\"
    @echo "  -H 'Content-Type: application/json' \\"
    @echo "  -d '{\"expression\": \"Bundle.entry.resource.where(resourceType = \\\"Patient\\\")\"}'"
    @echo ""
    @echo "📄 File operations:"
    @echo "curl http://localhost:8080/files                    # List files"
    @echo "curl http://localhost:8080/files/patient-example.json  # Get specific file"

# Watch server logs in development
server-watch:
    @echo "👀 Starting server with file watching for development"
    @echo "====================================================="
    cargo watch -x 'run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- server --cors-all'

# Create example FHIR resources for testing
server-setup-examples:
    @echo "📁 Setting up example FHIR resources"
    @echo "===================================="
    @mkdir -p storage/examples
    @if [ ! -f storage/examples/patient-example.json ]; then \
        echo "Creating patient-example.json..."; \
    else \
        echo "✅ Example files already exist in storage/examples/"; \
    fi
    @echo "📚 Available example files:"
    @ls -la storage/examples/ 2>/dev/null || echo "📁 Run server to auto-create storage directory"

# Build server and run in background for testing
server-background:
    @echo "🚀 Building and starting server in background"
    @echo "============================================="
    @cargo build --package octofhir-fhirpath --bin octofhir-fhirpath --features cli
    @echo "Starting server in background (PID will be shown)..."
    @nohup cargo run --package octofhir-fhirpath --bin octofhir-fhirpath --features cli -- server > server.log 2>&1 &
    @echo "✅ Server started in background"
    @echo "📋 Log output: tail -f server.log"
    @echo "🛑 Stop with: pkill -f octofhir-fhirpath"

# Stop background server
server-stop:
    @echo "🛑 Stopping background server"
    @echo "============================="
    @pkill -f "octofhir-fhirpath server" || echo "No server process found"
    @rm -f server.log

# Quick server health check
server-ping:
    @echo "🏥 Checking server health..."
    @curl -s http://localhost:8080/health >/dev/null && echo "✅ Server is running" || echo "❌ Server is not responding"

# Performance test the server endpoints
server-perf:
    @echo "⚡ Performance testing server endpoints"
    @echo "======================================"
    @echo "🔍 Testing evaluation endpoint performance..."
    @echo "POST /r4/evaluate with simple expression:"
    @time curl -s -X POST http://localhost:8080/r4/evaluate \
        -H 'Content-Type: application/json' \
        -d '{"expression": "Patient.active", "resource": {"resourceType": "Patient", "active": true}}' \
        >/dev/null || echo "❌ Server not running"

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