# FHIRPath Benchmark Results

Last updated: Thu Jul 31 19:21:22 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Parser | 703.04 ns | 1.42M ops/sec | 701.24 ns |
| Tokenizer | 84.08 ns | 11.89M ops/sec | 83.27 ns |
| Parser | 274.37 ns | 3.64M ops/sec | 267.49 ns |
| Tokenizer | 37.88 ns | 26.40M ops/sec | 37.71 ns |
| Parser | 159.29 ns | 6.28M ops/sec | 160.13 ns |
| Tokenizer | 24.77 ns | 40.38M ops/sec | 24.64 ns |
| Engine_creation | 109.80 μs | 9.11K ops/sec | 106.72 μs |
| Evaluation_simple | 107.19 μs | 9.33K ops/sec | 107.27 μs |
| Evaluator_ops_per_sec | 122.16 μs | 8.19K ops/sec | 114.55 μs |
| Parser_ops_per_sec | 709.68 ns | 1.41M ops/sec | 700.51 ns |
| Tokenizer_ops_per_sec | 84.00 ns | 11.90M ops/sec | 82.86 ns |
| Parser_1m_ops_target | 710.95 ns | 1.41M ops/sec | 726.19 ns |
| Tokenizer_11m_ops_target | 84.06 ns | 11.90M ops/sec | 82.85 ns |
| Parser_1m_target | 714.19 ns | 1.40M ops/sec | 635.02 ns |
| Tokenizer_10m_target | 79.97 ns | 12.50M ops/sec | 75.11 ns |

#### Evaluation_types

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Boolean | 120.18 μs | 8.32K ops/sec | 118.39 μs |
| Number | 112.34 μs | 8.90K ops/sec | 113.48 μs |
| String | 110.31 μs | 9.06K ops/sec | 109.33 μs |

#### Evaluator

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 125.83 μs | 7.95K ops/sec | 122.04 μs |
| Medium | 115.95 μs | 8.62K ops/sec | 112.21 μs |
| Simple | 114.79 μs | 8.71K ops/sec | 114.27 μs |

#### Expression_complexity

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Arithmetic | 117.63 μs | 8.50K ops/sec | 114.99 μs |
| Literal | 119.18 μs | 8.39K ops/sec | 117.88 μs |
| Simple_path | 118.06 μs | 8.47K ops/sec | 115.69 μs |

#### Full_pipeline

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 127.79 μs | 7.83K ops/sec | 117.16 μs |
| Medium | 123.54 μs | 8.09K ops/sec | 119.16 μs |
| Simple | 122.56 μs | 8.16K ops/sec | 118.73 μs |

#### Parser

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 832.94 ns | 1.20M ops/sec | 835.58 ns |
| Medium | 595.10 ns | 1.68M ops/sec | 592.14 ns |
| Simple | 168.19 ns | 5.95M ops/sec | 167.29 ns |

#### Target

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Expr_0_parser | 162.85 ns | 6.14M ops/sec | 161.65 ns |
| Expr_0_tokenizer | 85.46 ns | 11.70M ops/sec | 84.96 ns |
| Expr_1_parser | 281.48 ns | 3.55M ops/sec | 283.02 ns |
| Expr_1_tokenizer | 124.96 ns | 8.00M ops/sec | 122.83 ns |
| Expr_2_parser | 593.54 ns | 1.68M ops/sec | 599.08 ns |
| Expr_2_tokenizer | 168.69 ns | 5.93M ops/sec | 166.78 ns |
| Expr_3_parser | 730.52 ns | 1.37M ops/sec | 742.13 ns |
| Expr_3_tokenizer | 188.85 ns | 5.30M ops/sec | 187.01 ns |
| Expr_4_parser | 711.35 ns | 1.41M ops/sec | 723.26 ns |
| Expr_4_tokenizer | 188.94 ns | 5.29M ops/sec | 187.48 ns |
| Expr_5_parser | 436.17 ns | 2.29M ops/sec | 386.74 ns |
| Expr_5_tokenizer | 142.13 ns | 7.04M ops/sec | 139.80 ns |
| Expr_6_parser | 162.61 ns | 6.15M ops/sec | 162.11 ns |
| Expr_6_tokenizer | 86.94 ns | 11.50M ops/sec | 86.39 ns |
| Expr_7_parser | 279.79 ns | 3.57M ops/sec | 275.43 ns |
| Expr_7_tokenizer | 126.31 ns | 7.92M ops/sec | 124.58 ns |
| Optimized_tokenizer_complete | 222.86 ns | 4.49M ops/sec | 216.76 ns |
| Optimized_tokenizer_only | 85.04 ns | 11.76M ops/sec | 84.13 ns |
| Parser | 748.17 ns | 1.34M ops/sec | 730.34 ns |
| Parser_complete | 743.45 ns | 1.35M ops/sec | 743.05 ns |
| Tokenizer_complete | 187.91 ns | 5.32M ops/sec | 186.65 ns |
| Tokenizer_only | 83.53 ns | 11.97M ops/sec | 82.91 ns |

#### Tokenizer

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 218.32 ns | 4.58M ops/sec | 217.09 ns |
| Medium | 177.86 ns | 5.62M ops/sec | 167.54 ns |
| Simple | 94.00 ns | 10.64M ops/sec | 85.59 ns |

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

