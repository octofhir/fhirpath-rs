# FHIRPath Benchmark Results

Last updated: Fri Aug 01 12:30:50 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Parser | 690.68 ns | 1.45M ops/sec | 682.84 ns |
| Tokenizer | 90.91 ns | 11.00M ops/sec | 93.07 ns |
| Parser | 288.99 ns | 3.46M ops/sec | 277.69 ns |
| Tokenizer | 38.74 ns | 25.81M ops/sec | 38.64 ns |
| Parser | 166.09 ns | 6.02M ops/sec | 168.12 ns |
| Tokenizer | 25.24 ns | 39.62M ops/sec | 25.19 ns |
| Parse_batch | 344.31 ns | 2.90M ops/sec | 341.53 ns |
| Parse_batch | 1.07 μs | 934.12K ops/sec | 1.02 μs |
| Parse_function_calls | 8.27 μs | 120.89K ops/sec | 8.02 μs |
| Evaluator_ops_per_sec | 109.94 μs | 9.10K ops/sec | 108.45 μs |
| Parser_ops_per_sec | 676.93 ns | 1.48M ops/sec | 684.07 ns |
| Tokenizer_ops_per_sec | 84.70 ns | 11.81M ops/sec | 84.30 ns |
| Parse_complex_operations | 3.90 μs | 256.46K ops/sec | 3.90 μs |
| Interning_tokenizer | 1.44 μs | 695.82K ops/sec | 1.46 μs |
| Standard_tokenizer | 244.85 ns | 4.08M ops/sec | 240.22 ns |

#### Evaluator

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 112.93 μs | 8.85K ops/sec | 111.43 μs |
| Medium | 109.28 μs | 9.15K ops/sec | 109.43 μs |
| Simple | 109.86 μs | 9.10K ops/sec | 107.89 μs |

#### Full_pipeline

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 111.77 μs | 8.95K ops/sec | 110.37 μs |
| Medium | 108.43 μs | 9.22K ops/sec | 108.28 μs |
| Simple | 109.06 μs | 9.17K ops/sec | 107.86 μs |

#### Memory_layout

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 0 | 1.15 μs | 869.83K ops/sec | 1.15 μs |
| 1 | 1.24 μs | 807.67K ops/sec | 1.09 μs |
| 2 | 923.57 ns | 1.08M ops/sec | 934.21 ns |
| 3 | 1.42 μs | 706.20K ops/sec | 1.33 μs |
| 4 | 1.56 μs | 639.33K ops/sec | 1.43 μs |

#### Parser

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 812.05 ns | 1.23M ops/sec | 805.52 ns |
| Medium | 533.93 ns | 1.87M ops/sec | 542.30 ns |
| Simple | 155.38 ns | 6.44M ops/sec | 154.76 ns |

#### Target

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Expr_0_parser | 157.19 ns | 6.36M ops/sec | 158.90 ns |
| Expr_0_tokenizer | 83.48 ns | 11.98M ops/sec | 83.47 ns |
| Expr_1_parser | 275.62 ns | 3.63M ops/sec | 274.90 ns |
| Expr_1_tokenizer | 121.46 ns | 8.23M ops/sec | 121.38 ns |
| Expr_2_parser | 546.08 ns | 1.83M ops/sec | 541.95 ns |
| Expr_2_tokenizer | 169.19 ns | 5.91M ops/sec | 167.82 ns |
| Expr_3_parser | 679.76 ns | 1.47M ops/sec | 684.89 ns |
| Expr_3_tokenizer | 190.19 ns | 5.26M ops/sec | 189.98 ns |
| Expr_4_parser | 693.13 ns | 1.44M ops/sec | 687.34 ns |
| Expr_4_tokenizer | 198.74 ns | 5.03M ops/sec | 191.64 ns |
| Expr_5_parser | 389.45 ns | 2.57M ops/sec | 386.68 ns |
| Expr_5_tokenizer | 140.64 ns | 7.11M ops/sec | 139.69 ns |
| Expr_6_parser | 162.47 ns | 6.16M ops/sec | 162.42 ns |
| Expr_6_tokenizer | 84.20 ns | 11.88M ops/sec | 84.14 ns |
| Expr_7_parser | 272.19 ns | 3.67M ops/sec | 273.05 ns |
| Expr_7_tokenizer | 127.13 ns | 7.87M ops/sec | 128.65 ns |
| Optimized_tokenizer_complete | 224.33 ns | 4.46M ops/sec | 220.59 ns |
| Optimized_tokenizer_only | 92.90 ns | 10.76M ops/sec | 92.12 ns |
| Parser | 720.79 ns | 1.39M ops/sec | 721.43 ns |
| Tokenizer_complete | 188.28 ns | 5.31M ops/sec | 187.23 ns |
| Tokenizer_only | 87.11 ns | 11.48M ops/sec | 85.75 ns |

#### Tokenizer

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex | 220.64 ns | 4.53M ops/sec | 220.18 ns |
| Medium | 174.33 ns | 5.74M ops/sec | 172.19 ns |
| Simple | 86.67 ns | 11.54M ops/sec | 84.21 ns |

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

