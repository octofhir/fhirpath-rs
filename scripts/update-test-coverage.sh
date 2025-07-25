#!/bin/bash

# FHIRPath Test Coverage Update Script
# 
# This script runs the official FHIRPath tests and updates the TEST_COVERAGE.md file
# Usage: ./scripts/update-test-coverage.sh

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🧪 FHIRPath Test Coverage Update"
echo "================================="

echo "📦 Building test infrastructure..."

# Build the test runner
cd fhirpath-core
if ! cargo build --release 2>/dev/null; then
    echo "❌ Failed to build test runner"
    exit 1
fi

echo "🔍 Running comprehensive test coverage analysis..."

# Use our Rust-based coverage generator
echo "Using Rust coverage generator..."
if cargo test run_coverage_report -- --ignored --nocapture; then
    echo "✅ Coverage report generated successfully!"
    
    # Check if the report was created
    if [[ -f "TEST_COVERAGE.md" ]]; then
        echo "📁 Report location: fhirpath-core/TEST_COVERAGE.md"
        
        # Show a quick summary
        echo ""
        echo "📊 Quick Summary:"
        if command -v grep &> /dev/null; then
            TOTAL_SUITES=$(grep "Total Test Suites" TEST_COVERAGE.md | grep -o '[0-9]\+' | head -1 2>/dev/null || echo "N/A")
            TOTAL_TESTS=$(grep "Estimated Total Tests" TEST_COVERAGE.md | grep -o '[0-9]\+' | head -1 2>/dev/null || echo "N/A")
            PASS_RATE=$(grep "Estimated Passing Tests" TEST_COVERAGE.md | grep -o '[0-9]\+\.[0-9]\+%' | head -1 2>/dev/null || echo "N/A")
            
            echo "   Test Suites: $TOTAL_SUITES"
            echo "   Total Tests: $TOTAL_TESTS"
            echo "   Pass Rate: $PASS_RATE"
        fi
    else
        echo "⚠️  Report file not found"
    fi
else
    echo "❌ Failed to generate coverage report"
    exit 1
fi

cd "$PROJECT_ROOT"

echo ""
echo "🎉 Test coverage update completed!"
echo ""
echo "📊 View the full report:"
echo "   cat fhirpath-core/TEST_COVERAGE.md"
echo ""
echo "🔄 To run this again:"
echo "   ./scripts/update-test-coverage.sh"
echo ""
echo "💡 To run just the coverage generator:"
echo "   cd fhirpath-core && cargo test run_coverage_report -- --ignored --nocapture"