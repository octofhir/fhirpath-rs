#!/bin/bash

# Script to check string function test coverage and show only failures and errors
# This reduces context window by focusing on the specific string function tests

echo "🧪 String Function Test Results"
echo "==============================="

# Define the string function test files we care about
STRING_TESTS=(
    "contains-string"
    "starts-with"
    "ends-with"
    "substring"
    "length"
    "trim"
    "index-of"
    "concatenate"
)

echo "📦 Building test infrastructure..."
cargo build --release --quiet

if [ $? -ne 0 ]; then
    echo "❌ Failed to build project"
    exit 1
fi

echo "🏃 Running string function tests via integration tests..."

# Run integration tests and filter for string function results
cd fhirpath-core
test_output=$(timeout 120 cargo test --release --test run_official_tests -- --nocapture 2>&1)

echo ""
echo "📊 String Function Test Analysis"
echo "================================="

TOTAL_IMPROVEMENTS=0

for test_name in "${STRING_TESTS[@]}"; do
    echo ""
    echo "📋 Analyzing $test_name tests..."
    
    # Extract results for this specific test
    result_line=$(echo "$test_output" | grep -i "Testing.*$test_name" -A 20 | grep -E "(PASS|FAIL|ERROR|✅|❌|⚠️)" | head -10)
    
    if [ -n "$result_line" ]; then
        echo "$result_line"
        
        # Count improvements (this is a simplified check)
        improvements=$(echo "$result_line" | grep -c "✅\|PASS")
        TOTAL_IMPROVEMENTS=$((TOTAL_IMPROVEMENTS + improvements))
    else
        echo "⚠️  No results found for $test_name (may not be running or test name changed)"
    fi
done

echo ""
echo "📊 Summary of String Function Status"
echo "===================================="

if [ "$TOTAL_IMPROVEMENTS" -gt 15 ]; then
    echo "🎉 Significant improvements detected in string functions!"
    echo "✅ Many string function tests are now passing"
elif [ "$TOTAL_IMPROVEMENTS" -gt 5 ]; then
    echo "🟡 Some improvements detected in string functions"
    echo "🔄 Progress made but more work needed"
else
    echo "🔴 Limited improvements detected"
    echo "💭 May need to check if method call fix is working correctly"
fi

echo ""
echo "🔍 Key String Function Issues (if any):"
echo "========================================"

# Show any failures or errors in string function tests
string_failures=$(echo "$test_output" | grep -A 5 -B 5 -E "(contains-string|starts-with|ends-with|substring|length|trim|index-of|concatenate)" | grep -E "(❌|⚠️|FAIL|ERROR)" | head -20)

if [ -n "$string_failures" ]; then
    echo "$string_failures"
else
    echo "✨ No major string function failures detected in output!"
fi

cd ..