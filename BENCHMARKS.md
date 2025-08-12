# FHIRPath Benchmark Results

Last updated: Tue Aug 12 21:59:15 UTC 2025

## Performance Overview

This document contains the latest benchmark results for the FHIRPath implementation.

### Core Components

The following components have been benchmarked with their current performance metrics:

#### Criterion

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Patient.identifier.where(system = 'http___example.org') | 7.88 ns | 126.96M ops/sec | 7.78 ns |
| Patient.name | 7.96 ns | 125.61M ops/sec | 7.78 ns |
| Patient.name.given | 7.84 ns | 127.51M ops/sec | 7.77 ns |
| Patient.name.given[0] | 8.46 ns | 118.21M ops/sec | 7.77 ns |

#### Evaluate

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Bundle.entry.resource.where(resourcetype='medicationrequest').me | 411.15 μs | 2.43K ops/sec | 381.61 μs |
| Patient.name | 211.17 μs | 4.74K ops/sec | 197.74 μs |

#### Parse

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Bundle.entry.resource.where(resourcetype='medicationrequest').me | 7.71 μs | 129.63K ops/sec | 7.65 μs |
| Patient.identifier.where(system = 'http___example.org') | 4.65 μs | 214.85K ops/sec | 4.38 μs |
| Patient.name | 2.05 μs | 487.68K ops/sec | 2.02 μs |
| Patient.name.given | 3.24 μs | 308.92K ops/sec | 3.22 μs |
| Patient.name.given[0] | 3.77 μs | 265.12K ops/sec | 3.74 μs |

#### Tokenize

| Complexity | Mean Time | Throughput | Median Time |
|------------|-----------|------------|-------------|
| Bundle.entry.resource.where(resourcetype='medicationrequest').me | 4.73 μs | 211.37K ops/sec | 4.59 μs |
| Patient.name | 1.50 μs | 665.77K ops/sec | 1.49 μs |
| Patient.name.given[0] | 2.81 μs | 355.39K ops/sec | 2.71 μs |

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

