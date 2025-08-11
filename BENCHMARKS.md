# FHIRPath Benchmark Results

Last updated: Mon Aug 11 17:11:59 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| String_interning_baseline | 22.37 μs | 44.70K ops/sec | 22.27 μs |
| String_interning_hit_rate | 42.60 μs | 23.47K ops/sec | 42.40 μs |
| Evaluator_throughput | 206.67 μs | 4.84K ops/sec | 204.03 μs |
| Parser_throughput | 572.31 ns | 1.75M ops/sec | 570.96 ns |
| Tokenizer_throughput | 280.69 ns | 3.56M ops/sec | 278.38 ns |
| Interner_stats | 104.60 ns | 9.56M ops/sec | 104.00 ns |
| With_interning | 2.47 μs | 405.20K ops/sec | 2.45 μs |
| Without_interning | 1.71 μs | 585.93K ops/sec | 1.70 μs |
| Is_keyword_str | 53.07 ns | 18.84M ops/sec | 52.76 ns |
| Keyword_lookup | 2.22 μs | 449.66K ops/sec | 2.15 μs |
| Keyword_stats | 1.54 ns | 649.92M ops/sec | 1.50 ns |
| Memory_estimation | 0.67 ns | 1483.66M ops/sec | 0.67 ns |

#### Evaluator

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex_chained | 213.47 μs | 4.68K ops/sec | 210.45 μs |
| Extremely_complex_multi_lambda | 214.35 μs | 4.67K ops/sec | 210.65 μs |
| Medium_function | 224.18 μs | 4.46K ops/sec | 208.31 μs |
| Medium_where | 207.83 μs | 4.81K ops/sec | 204.58 μs |
| Simple_literal | 235.93 μs | 4.24K ops/sec | 213.07 μs |
| Simple_property | 216.72 μs | 4.61K ops/sec | 207.35 μs |

#### Parser

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex_chained | 859.76 ns | 1.16M ops/sec | 849.67 ns |
| Extremely_complex_multi_lambda | 3.12 μs | 320.94K ops/sec | 2.96 μs |
| Medium_function | 377.90 ns | 2.65M ops/sec | 370.46 ns |
| Medium_where | 605.65 ns | 1.65M ops/sec | 600.55 ns |
| Simple_literal | 33.10 ns | 30.21M ops/sec | 32.96 ns |
| Simple_property | 245.31 ns | 4.08M ops/sec | 228.31 ns |

#### Tokenizer

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Complex_chained | 399.61 ns | 2.50M ops/sec | 398.10 ns |
| Extremely_complex_multi_lambda | 1.34 μs | 745.85K ops/sec | 1.33 μs |
| Medium_function | 213.00 ns | 4.69M ops/sec | 213.01 ns |
| Medium_where | 297.07 ns | 3.37M ops/sec | 296.29 ns |
| Simple_literal | 44.45 ns | 22.50M ops/sec | 44.30 ns |
| Simple_property | 184.53 ns | 5.42M ops/sec | 166.55 ns |

#### Tokenizer_streaming

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| 26000 | 269.07 μs | 3.72K ops/sec | 267.78 μs |
| 2600 | 25.49 μs | 39.23K ops/sec | 25.36 μs |
| 260 | 2.68 μs | 373.23K ops/sec | 2.68 μs |
| 26000 | 300.00 μs | 3.33K ops/sec | 298.17 μs |
| 2600 | 28.51 μs | 35.08K ops/sec | 28.32 μs |
| 260 | 2.87 μs | 348.17K ops/sec | 2.86 μs |

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

