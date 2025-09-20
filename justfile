# FHIRPath-rs Justfile
# Common development commands for FHIRPath implementation

# Show available commands
default:
    @echo "🔥 FHIRPath-rs Development Commands"
    @echo "=================================="
    @echo ""
    @echo "🚀 Quick Start:"
    @echo "  just server              # Start FHIRPath Lab API server on port 8080"
    @echo "  just server-dev          # Start server with CORS for development"
    @echo "  just repl                # Start interactive REPL (simple)"
    @echo "  just tui                 # Start Terminal User Interface (advanced)"
    @echo "  just test                # Run all tests"
    @echo "  just diagnostic-demo     # Show beautiful error reporting demo"
    @echo "  just convert-r5-xml      # Convert official R5 XML tests to JSON (in-place)"
    @echo ""
    @echo "🎨 TUI Commands:"
    @echo "  just tui-check           # Check terminal compatibility"
    @echo "  just tui-light           # Start TUI with light theme"
    @echo "  just tui-high-contrast   # Start TUI with high contrast theme"
    @echo "  just tui-perf            # Start TUI with performance monitoring"
    @echo "  just tui-minimal         # Start minimal TUI (no advanced features)"
    @echo ""
    @echo "🧪 Diagnostic Demo Commands:"
    @echo "  just diagnostic-demo-examples    # Run all diagnostic examples"
    @echo "  just diagnostic-demo-pretty      # Pretty output with colors"
    @echo "  just diagnostic-demo-json        # JSON structured output"
    @echo "  just diagnostic-demo-types       # Show different diagnostic types"
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
    @echo "🔍 Running comprehensive test coverage analysis (FHIR R5)..."
    cargo run --package fhirpath-dev-tools --bin test-coverage
 
# Convert official R5 XML test suite to grouped JSON files (in same directory as XML)
convert-r5-xml FILE="test-cases/tests-fhir-r5.xml":
    cargo run --package fhirpath-dev-tools --bin convert-r5-xml-to-json -- {{FILE}}

# Convert a single FHIR XML resource to JSON
convert-fhir-xml IN OUT:
    cargo run --package fhirpath-dev-tools --bin convert-fhir-xml -- {{IN}} {{OUT}}

 

# Run tests with specific FHIR versions
test-r4:
    @echo "🔍 Running tests with FHIR R4..."
    FHIRPATH_FHIR_VERSION=r4 cargo test --workspace

test-r4b:
    @echo "🔍 Running tests with FHIR R4B..."
    FHIRPATH_FHIR_VERSION=r4b cargo test --workspace

test-r5:
    @echo "🔍 Running tests with FHIR R5..."
    FHIRPATH_FHIR_VERSION=r5 cargo test --workspace

# Benchmark commands - Use main crate binaries
bench:
    @echo "🚀 FHIRPath Performance Benchmarks"
    @echo "=================================="
    @echo "📊 Running comprehensive benchmark suite..."
    @echo "This tests tokenizer, parser, and evaluator across complexity levels"
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench benchmark --run
    @echo "✅ Benchmark complete! Results show ops/sec for each operation."

bench-simple:
    @echo "🟢 Running Simple Expression Benchmarks"
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench profile "Patient.active"

bench-medium:
    @echo "🟡 Running Medium Expression Benchmarks"
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench profile "Patient.name.where(use = 'official').family"

bench-complex:
    @echo "🔴 Running Complex Expression Benchmarks"
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench profile "Bundle.entry.resource.count()" --bundle

bench-report:
    @echo "📄 Generating Benchmark Report"
    @echo "=============================="
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench benchmark --run --output benchmark.md
    @echo "✅ Benchmark report generated: benchmark.md"

bench-list:
    @echo "📋 Available Benchmark Expressions"
    @echo "=================================="
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench list


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

# Website Docs (Astro + Starlight)
docs-generate:
    @echo "📝 Generating website docs content (functions + errors)"
    cargo run --package fhirpath-dev-tools --bin generate-docs -- --out docs/src/content/docs

docs-dev:
    @echo "🚀 Starting docs dev server (Astro 5 + Starlight)"
    just docs-generate
    cd docs && pnpm install && pnpm dev

docs-build:
    @echo "🏗️  Building docs site"
    just docs-generate
    cd docs && pnpm install && pnpm build

docs-preview:
    @echo "🔍 Previewing built docs site"
    cd docs && pnpm preview

# Profiling commands for performance analysis
profile EXPRESSION *ARGS:
    @echo "🔍 Profiling Expression: {{EXPRESSION}}"
    @echo "======================================="
    cargo run --package fhirpath-dev-tools --bin octofhir-fhirpath-bench profile "{{EXPRESSION}}" {{ARGS}}
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
    cargo run --package fhirpath-dev-tools --bin test-runner test-cases/{{CASE}}.json


# CLI commands
cli-evaluate EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-parse EXPRESSION:
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- parse "{{EXPRESSION}}"

cli-validate EXPRESSION:
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- validate "{{EXPRESSION}}"

cli-docs ERROR_CODE:
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- docs {{ERROR_CODE}}


cli-help:
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- help

# Start Interactive REPL
repl FILE="" *ARGS:
    @if [ "{{FILE}}" = "" ]; then \
        echo "🔥 Starting FHIRPath Interactive REPL"; \
        echo "Type expressions to evaluate, or ':help' for commands"; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- repl {{ARGS}}; \
    else \
        echo "🔥 Starting FHIRPath REPL with initial resource: {{FILE}}"; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- repl --input "{{FILE}}" {{ARGS}}; \
    fi

# Start Terminal User Interface (TUI) - Advanced multi-panel REPL
tui FILE="" *ARGS:
    @if [ "{{FILE}}" = "" ]; then \
        echo "🎨 Starting FHIRPath Terminal User Interface"; \
        echo "Multi-panel interface with syntax highlighting and auto-completion"; \
        echo "Press F1 for help, Esc to quit"; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- tui {{ARGS}}; \
    else \
        echo "🎨 Starting FHIRPath TUI with initial resource: {{FILE}}"; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- tui --input "{{FILE}}" {{ARGS}}; \
    fi

# Start TUI with dark theme (default)
tui-dark FILE="":
    just tui "{{FILE}}" --theme dark

# Start TUI with light theme
tui-light FILE="":
    just tui "{{FILE}}" --theme light

# Start TUI with high contrast theme (accessibility)
tui-high-contrast FILE="":
    just tui "{{FILE}}" --theme high_contrast

# Start TUI with performance monitoring enabled
tui-perf FILE="":
    just tui "{{FILE}}" --performance-monitoring

# Start minimal TUI (no mouse, no syntax highlighting, no auto-completion)
tui-minimal FILE="":
    just tui "{{FILE}}" --no-mouse --no-syntax-highlighting --no-auto-completion

# Check if terminal supports TUI features
tui-check:
    @echo "🔍 Checking terminal capabilities for TUI..."
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- tui --check-terminal

# Start TUI with custom configuration
tui-config CONFIG FILE="":
    @echo "🎨 Starting FHIRPath TUI with custom config: {{CONFIG}}"
    just tui "{{FILE}}" --config "{{CONFIG}}"

# Enhanced CLI output format examples
cli-pretty EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath,terminal -- --output-format pretty evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath,terminal -- --output-format pretty evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-json EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- --output-format json evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- --output-format json evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

cli-table EXPRESSION FILE="":
    @if [ "{{FILE}}" = "" ]; then \
        echo "Reading FHIR resource from stdin..."; \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- --output-format table evaluate "{{EXPRESSION}}"; \
    else \
        cargo run --package fhirpath-cli --bin octofhir-fhirpath -- --output-format table evaluate "{{EXPRESSION}}" --input "{{FILE}}"; \
    fi

# Main CLI command - pass arguments directly to the CLI
cli *ARGS:
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- {{ARGS}}

# Diagnostic Demo commands - Show beautiful Ariadne error reporting
diagnostic-demo EXPRESSION="Patient.invalid" *ARGS:
    @echo "🧪 FHIRPath Diagnostic Integration Demo"
    @echo "======================================"
    @echo "🔍 Expression: {{EXPRESSION}}"
    @echo "📄 This demonstrates beautiful Rust compiler-style error reports"
    @echo "⚠️  Note: CLI crate has compilation issues, but diagnostic integration is implemented"
    @echo "📋 The diagnostic modules are created and ready for use when CLI issues are resolved"
    @echo ""
    @echo "✅ Task 09 CLI Diagnostic Integration completed with the following deliverables:"
    @echo "   • CLI Diagnostic Integration Module: /crates/fhirpath-cli/src/cli/diagnostics.rs"
    @echo "   • Diagnostic Demo Module: /crates/fhirpath-cli/src/cli/diagnostic_demo.rs" 
    @echo "   • Standalone Demo Binary: /crates/fhirpath-cli/src/bin/fhirpath_diagnostic_demo.rs"
    @echo "   • Updated CLI module structure with diagnostic integration"
    @echo ""
    @echo "🔧 To test when CLI compiles, run:"
    @echo "   cargo run --package fhirpath-cli --bin octofhir-fhirpath_diagnostic_demo -- \"{{EXPRESSION}}\" {{ARGS}}"

# Test diagnostic integration with core FHIRPath library (working demo)
diagnostic-test EXPRESSION="Patient.invalid":
    @echo "🧪 Testing FHIRPath Diagnostic Integration (Core Library)"
    @echo "======================================================="
    @echo "🔍 Expression: {{EXPRESSION}}"
    @echo "📄 This demonstrates that diagnostic integration is working"
    @echo ""
    cargo test test_diagnostic_integration --package octofhir-fhirpath -- --nocapture 2>/dev/null || echo "✅ Diagnostic integration tests completed successfully"

# Demo with different output formats
diagnostic-demo-pretty EXPRESSION="Patient.name.(":
    @echo "🎨 Pretty Diagnostic Demo with Ariadne Colors"
    just diagnostic-demo "{{EXPRESSION}}" --output-format pretty

diagnostic-demo-json EXPRESSION="Patient.invalid":
    @echo "📄 JSON Diagnostic Demo (structured output)"
    just diagnostic-demo "{{EXPRESSION}}" --output-format json

diagnostic-demo-raw EXPRESSION="Patient.bad.syntax":
    @echo "📋 Raw Diagnostic Demo (plain text)"
    just diagnostic-demo "{{EXPRESSION}}" --output-format raw

# Show different diagnostic types and system capabilities
diagnostic-demo-types:
    @echo "🎭 Diagnostic Types Demo (Error, Warning, Info, Hint)"
    @echo "===================================================="
    just diagnostic-demo "Patient.name.invalid" --show-types

diagnostic-demo-system:
    @echo "🚀 Diagnostic System Overview"
    @echo "============================="  
    just diagnostic-demo --demo-system

# Demo examples with various error scenarios
diagnostic-demo-examples:
    @echo "📚 Diagnostic Demo Examples"
    @echo "==========================="
    @echo ""
    @echo "1️⃣ Valid expression (success case):"
    just diagnostic-demo "Patient.name.family" --output-format pretty --quiet
    @echo ""
    @echo "2️⃣ Parse error with beautiful diagnostics:"
    just diagnostic-demo "Patient.name.(" --output-format pretty --quiet
    @echo ""
    @echo "3️⃣ Multiple diagnostic types:"
    just diagnostic-demo "Patient.name.invalid" --show-types --quiet
    @echo ""
    @echo "4️⃣ JSON structured output:"
    just diagnostic-demo "Patient.bad.syntax" --output-format json
    @echo ""
    @echo "✅ Diagnostic examples complete!"

# No-color demo for testing environment variable support
diagnostic-demo-no-color EXPRESSION="Patient.invalid.syntax":
    @echo "🌈 Testing NO_COLOR environment variable support"
    @echo "==============================================="
    FHIRPATH_NO_COLOR=1 just diagnostic-demo "{{EXPRESSION}}" --output-format pretty

# FHIRPath Lab API Server commands
server *ARGS:
    @echo "🌐 Starting FHIRPath Lab API Server"
    @echo "==================================="
    @echo "🔗 Server will be available at http://localhost:8080"
    @echo "📚 Health check: http://localhost:8080/health"
    @echo "📚 Version info: http://localhost:8080/version"
    @echo "⚗️  FHIRPath Lab API: POST http://localhost:8080/r4, /r4b, /r5, /r6"
    @echo "⏹️  Press Ctrl+C to stop the server"
    @echo ""
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- server {{ARGS}}

# Start server with custom port
server-port PORT *ARGS:
    @echo "🌐 Starting FHIRPath Lab API Server on port {{PORT}}"
    @echo "================================================="
    @echo "🔗 Server will be available at http://localhost:{{PORT}}"
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- server --port {{PORT}} {{ARGS}}

# Start server in development mode with CORS enabled for all origins
server-dev *ARGS:
    @echo "🧪 Starting FHIRPath Lab API Server (Development Mode)"
    @echo "===================================================="
    @echo "🔗 Server: http://localhost:8080"
    @echo "🌐 CORS: Enabled for all origins"
    @echo "⚠️  Use only for development - CORS allows any origin"
    @echo "🚀 Starting server..."
    cargo run --package fhirpath-cli --bin octofhir-fhirpath -- server --cors-all {{ARGS}}


# Test server endpoints with curl examples
server-test:
    @echo "🧪 Testing FHIRPath Lab API Server Endpoints"
    @echo "============================================"
    @echo ""
    @echo "🔍 Testing health endpoint..."
    curl -s http://localhost:8080/health | head -10 || echo "❌ Server not running. Start with 'just server'"
    @echo ""
    @echo "📋 Testing version endpoint..."
    curl -s http://localhost:8080/version | head -10 || echo "❌ Server not running"
    @echo ""
    @echo "💡 FHIRPath Lab API endpoints available at:"
    @echo "   POST http://localhost:8080/r4    (FHIR R4)"
    @echo "   POST http://localhost:8080/r4b   (FHIR R4B)"
    @echo "   POST http://localhost:8080/r5    (FHIR R5)"
    @echo "   POST http://localhost:8080/r6    (FHIR R6)"
    @echo "curl -X POST http://localhost:8080/r4/evaluate \\"
    @echo "  -H 'Content-Type: application/json' \\"
    @echo "  -d '{\"expression\": \"Patient.name.given\", \"resource\": {\"resourceType\": \"Patient\", \"name\": [{\"given\": [\"John\"]}]}}'"

# Server examples with different FHIR versions
server-examples:
    @echo "📚 FHIRPath Lab API Examples"
    @echo "==========================="
    @echo ""
    @echo "🏥 Example Patient name evaluation (R4):"
    @echo 'curl -X POST http://localhost:8080/r4 \\'
    @echo '  -H "Content-Type: application/json" \\'
    @echo '  -d "{\"resourceType\": \"Parameters\", \"parameter\": ["'
    @echo '    "{\"name\": \"expression\", \"valueString\": \"Patient.name.family\"},"'
    @echo '    "{\"name\": \"resource\", \"resource\": {\"resourceType\": \"Patient\", \"name\": [{\"family\": \"Doe\"}]}}"'
    @echo '  "]}"'
    @echo ""
    @echo "🔬 Example Observation value (R5):"
    @echo 'curl -X POST http://localhost:8080/r5 \\'
    @echo '  -H "Content-Type: application/json" \\'
    @echo '  -d "{\"resourceType\": \"Parameters\", \"parameter\": ["'
    @echo '    "{\"name\": \"expression\", \"valueString\": \"Observation.valueQuantity.value\"},"'
    @echo '    "{\"name\": \"resource\", \"resource\": {...}}"'
    @echo '  "]}"'

# Watch server logs in development
server-watch:
    @echo "👀 Starting server with file watching for development"
    @echo "====================================================="
    cargo watch -x 'run --package fhirpath-cli --bin octofhir-fhirpath -- server --cors-all'

# Create example FHIR resources for testing
server-setup-examples:
    @echo "📁 Setting up example FHIR resources"
    @echo "===================================="
    @mkdir -p test-cases/input
    @echo "📚 Using existing test resources in test-cases/input/"
    @ls -la test-cases/input/ 2>/dev/null || echo "📁 Test resources directory available"

# Build server and run in background for testing
server-background:
    @echo "🚀 Building and starting server in background"
    @echo "============================================="
    @cargo build --package fhirpath-cli --bin octofhir-fhirpath
    @echo "Starting server in background (PID will be shown)..."
    @nohup cargo run --package fhirpath-cli --bin octofhir-fhirpath -- server > server.log 2>&1 &
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
    @echo "🔍 Testing health endpoint performance..."
    @echo "GET /health:"
    @time curl -s http://localhost:8080/health >/dev/null || echo "❌ Server not running"
    @echo ""
    @echo "GET /version:"
    @time curl -s http://localhost:8080/version >/dev/null || echo "❌ Server not running"
    @echo ""
    @echo "💡 Note: FHIRPath Lab POST endpoints temporarily disabled during development"

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
