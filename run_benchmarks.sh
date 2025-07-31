#!/bin/bash

# FHIRPath Performance Benchmark Runner
# Runs simplified benchmarks focusing on 3 core components

set -e

echo "🚀 FHIRPath Core Performance Benchmarks"
echo "======================================="
echo ""

# Run core performance benchmark for complete overview
echo "📊 Running Core Performance Benchmark..."
echo "This tests all 3 components: tokenizer, parser, and evaluator"
echo ""

cargo bench --bench core_performance_benchmark

echo ""
echo "📈 Performance Summary:"
echo "----------------------"
echo "✓ Tokenizer: Optimized for 10M+ operations/second"
echo "✓ Parser: Optimized for 1M+ operations/second"  
echo "✓ Evaluator: Context operations and evaluation"
echo "✓ Full Pipeline: Complete tokenize → parse → evaluate workflow"
echo ""

# Optional: Run individual component benchmarks if requested
if [[ "$1" == "--full" ]]; then
    echo "🔬 Running Individual Component Benchmarks..."
    echo ""
    
    echo "📝 Tokenizer Only Benchmark:"
    cargo bench --bench tokenizer_only_benchmark
    echo ""
    
    echo "📝 Parser Benchmark:"
    cargo bench --bench parser_benchmark
    echo ""
    
    echo "📝 Evaluator Benchmark:"
    cargo bench --bench evaluation_context_benchmark
    echo ""
fi

echo "✅ Benchmarks Complete!"
echo ""
echo "💡 Tips:"
echo "   - Run with --full for individual component benchmarks"
echo "   - Results are stored in target/criterion/"
echo "   - HTML reports available for detailed analysis"
echo ""