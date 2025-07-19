#!/bin/bash

# Script to update test compliance documentation
# This script runs the official FHIRPath tests and extracts the results
# to update the test compliance documentation page

set -e

echo "Running official FHIRPath tests..."
cd fhirpath-core

# Run tests and capture output
TEST_OUTPUT=$(cargo test run_official_fhirpath_tests -- --nocapture 2>&1)

# Extract test results from the output
PASSED=$(echo "$TEST_OUTPUT" | grep "Passed:" | sed 's/Passed: //')
FAILED=$(echo "$TEST_OUTPUT" | grep "Failed:" | sed 's/Failed: //')
SKIPPED=$(echo "$TEST_OUTPUT" | grep "Skipped:" | sed 's/Skipped: //')
TOTAL=$(echo "$TEST_OUTPUT" | grep "Total:" | sed 's/Total: //')

echo "Test Results:"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "Skipped: $SKIPPED"
echo "Total: $TOTAL"

# Calculate percentages using awk (more portable than bc)
if [ ! -z "$TOTAL" ] && [ "$TOTAL" -gt 0 ]; then
    PASSED_PCT=$(awk "BEGIN {printf \"%.1f\", $PASSED * 100 / $TOTAL}")
    FAILED_PCT=$(awk "BEGIN {printf \"%.1f\", $FAILED * 100 / $TOTAL}")
    SKIPPED_PCT=$(awk "BEGIN {printf \"%.1f\", $SKIPPED * 100 / $TOTAL}")

    echo "Percentages:"
    echo "Passed: ${PASSED_PCT}%"
    echo "Failed: ${FAILED_PCT}%"
    echo "Skipped: ${SKIPPED_PCT}%"

    # Update the documentation file
    cd ../docs-site/src/content/docs/reference

    # Create a backup
    cp test-compliance.md test-compliance.md.bak

    # Update the test results table (using simpler patterns to avoid emoji issues)
    sed -i.tmp "s/| ✅ \*\*Passed\*\* | [0-9]* | [0-9.]*% |/| ✅ **Passed** | $PASSED | ${PASSED_PCT}% |/" test-compliance.md
    sed -i.tmp "s/| ❌ \*\*Failed\*\* | [0-9]* | [0-9.]*% |/| ❌ **Failed** | $FAILED | ${FAILED_PCT}% |/" test-compliance.md
    sed -i.tmp "s/| ⏭️ \*\*Skipped\*\* | [0-9]* | [0-9.]*% |/| ⏭️ **Skipped** | $SKIPPED | ${SKIPPED_PCT}% |/" test-compliance.md
    sed -i.tmp "s/| \*\*Total\*\* | \*\*[0-9]*\*\* | \*\*100%\*\* |/| **Total** | **$TOTAL** | **100%** |/" test-compliance.md

    # Update the last updated date
    TODAY=$(date +"%B %d, %Y")
    sed -i.tmp "s/\*Last updated: .*/\*Last updated: $TODAY\*/" test-compliance.md

    # Clean up temporary files
    rm test-compliance.md.tmp

    echo "Documentation updated successfully!"
    echo "Updated file: docs-site/src/content/docs/reference/test-compliance.md"
else
    echo "Error: Could not extract test results from output"
    exit 1
fi
