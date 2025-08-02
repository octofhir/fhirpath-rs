# FHIRPath Benchmark Results

Last updated: Sat Aug 02 15:08:47 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Evaluator_throughput | 200.71 μs | 4.98K ops/sec | 192.63 μs |
| Parser_throughput | 610.77 ns | 1.64M ops/sec | 592.11 ns |
| Tokenizer_throughput | 158.54 ns | 6.31M ops/sec | 157.94 ns |

#### Evaluator

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 199.29 μs | 5.02K ops/sec | 195.22 μs |
| Medium | 196.57 μs | 5.09K ops/sec | 191.06 μs |
| Simple | 188.54 μs | 5.30K ops/sec | 187.67 μs |

#### Parser

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 862.28 ns | 1.16M ops/sec | 877.73 ns |
| Medium | 599.31 ns | 1.67M ops/sec | 600.05 ns |
| Simple | 173.13 ns | 5.78M ops/sec | 177.05 ns |

#### Tokenizer

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 211.73 ns | 4.72M ops/sec | 206.48 ns |
| Medium | 157.45 ns | 6.35M ops/sec | 156.95 ns |
| Simple | 69.97 ns | 14.29M ops/sec | 69.20 ns |

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

