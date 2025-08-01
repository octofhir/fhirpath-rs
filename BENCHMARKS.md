# FHIRPath Benchmark Results

Last updated: Fri Aug 01 19:31:57 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Tokenizer_ops_per_sec | 148.99 ns | 6.71M ops/sec | 148.99 ns |
| Tokenizer_throughput | 160.34 ns | 6.24M ops/sec | 158.57 ns |

#### Evaluator

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 196.64 μs | 5.09K ops/sec | 195.19 μs |
| Medium | 195.61 μs | 5.11K ops/sec | 192.65 μs |
| Simple | 187.44 μs | 5.34K ops/sec | 188.44 μs |

#### Parser

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 903.49 ns | 1.11M ops/sec | 884.37 ns |
| Medium | 614.93 ns | 1.63M ops/sec | 598.57 ns |
| Simple | 176.60 ns | 5.66M ops/sec | 179.28 ns |

#### Target

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Expr_0_tokenizer | 65.05 ns | 15.37M ops/sec | 65.05 ns |
| Expr_1_tokenizer | 149.59 ns | 6.69M ops/sec | 149.59 ns |
| Expr_2_tokenizer | 234.40 ns | 4.27M ops/sec | 234.40 ns |
| Expr_3_tokenizer | 258.34 ns | 3.87M ops/sec | 258.34 ns |
| Expr_4_tokenizer | 265.94 ns | 3.76M ops/sec | 265.94 ns |
| Expr_5_tokenizer | 293.96 ns | 3.40M ops/sec | 293.96 ns |
| Expr_6_tokenizer | 263.07 ns | 3.80M ops/sec | 263.07 ns |
| Expr_7_tokenizer | 295.92 ns | 3.38M ops/sec | 295.92 ns |
| Optimized_tokenizer_complete | 256.50 ns | 3.90M ops/sec | 256.50 ns |
| Optimized_tokenizer_only | 147.45 ns | 6.78M ops/sec | 147.45 ns |

#### Tokenizer

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 211.16 ns | 4.74M ops/sec | 211.24 ns |
| Medium | 169.53 ns | 5.90M ops/sec | 164.73 ns |
| Simple | 71.36 ns | 14.01M ops/sec | 71.05 ns |

### Performance Summary

**Key Metrics:**
- **Tokenizer**: Processes FHIRPath expressions into tokens
- **Parser**: Builds AST from tokens using Pratt parsing
- **Evaluator**: Executes FHIRPath expressions against data
- **Full Pipeline**: Complete tokenize → parse → evaluate workflow

### Detailed Results

For detailed benchmark results, charts, and statistical analysis, see the HTML reports in `target/criterion/`.

### Running Benchmarks

```bash
# Run core benchmarks
just bench

# Run full benchmark suite
just bench-full

# Update this documentation
just bench-update-docs
```

### Benchmark Infrastructure

- **Framework**: Criterion.rs v0.7
- **Statistical Analysis**: Includes confidence intervals, outlier detection
- **Sample Sizes**: Adaptive sampling for statistical significance
- **Measurement**: Wall-clock time with warm-up cycles

