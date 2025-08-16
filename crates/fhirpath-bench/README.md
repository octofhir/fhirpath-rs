# FHIRPath Benchmarking Suite

A comprehensive benchmarking suite for FHIRPath-rs using the `divan` library.

## Features

- **Performance Benchmarking**: Measures tokenizer, parser, and evaluation performance in ops/sec
- **Expression Categories**: Simple, medium, and complex expressions including bundle operations
- **Profiling Support**: Generate flamegraphs for detailed performance analysis
- **Automated Reporting**: Generate benchmark.md files with results

## Expression Categories

### Simple Expressions
Basic field access and operations:
- `Patient.active`
- `Patient.name.family`
- `Patient.birthDate`
- `1 + 2`
- `Patient.name.count()`

### Medium Expressions
Filtered queries and functions:
- `Patient.name.where(use = 'official').family`
- `Patient.telecom.where(system = 'phone').value`
- `Patient.birthDate > @1980-01-01`
- `Patient.name.family.substring(0, 3)`

### Complex Expressions
Bundle operations and resolve() calls (from resolve.json):
- `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`
- `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`
- `Bundle.entry.resource.descendants().where($this is Reference).reference`

## Usage

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --package fhirpath-bench

# Run specific benchmark category
cargo bench --package fhirpath-bench -- "simple"
cargo bench --package fhirpath-bench -- "medium" 
cargo bench --package fhirpath-bench -- "complex"
```

### CLI Commands

```bash
# List available expressions
fhirpath-bench list

# Profile a specific expression
fhirpath-bench profile "Patient.active" --output ./profile_output --iterations 1000

# Profile complex expression with bundle data
fhirpath-bench profile "Bundle.entry.resource.count()" --bundle --iterations 500

# Generate benchmark report
fhirpath-bench benchmark --output benchmark.md --run

# Generate template (without running benchmarks)
fhirpath-bench benchmark --output benchmark.md
```

### Profiling Options

- `--output, -o`: Output directory for flamegraph files (default: `./profile_output`)
- `--iterations, -i`: Number of iterations for profiling (default: 1000)
- `--bundle, -b`: Use bundle data instead of patient data

## Data Sources

- **Patient Data**: Sample Patient resource for simple/medium expressions
- **Bundle Data**: Uses `bundle-medium.json` from the specs directory for complex expressions
- **Fallback**: Gracefully handles missing files with sample data

## Benchmark Structure

The suite measures three key operations:

1. **Tokenization** (`bench_tokenize_*`): Converting expressions to tokens
2. **Parsing** (`bench_parse_*`): Building AST from tokens  
3. **Evaluation** (`bench_evaluate_*`): Executing expressions against FHIR data

Each operation is benchmarked across all expression categories to provide comprehensive performance insights.

## Output Files

### Profiling Output
- `{expression}_tokenize.svg`: Tokenization flamegraph
- `{expression}_parse.svg`: Parsing flamegraph
- `{expression}_evaluate.svg`: Evaluation flamegraph

### Benchmark Reports
- `benchmark.md`: Comprehensive benchmark results with performance tables

## Implementation Details

- Built on `divan` for high-precision benchmarking
- Uses `pprof` for flamegraph generation  
- Async-aware evaluation benchmarking
- Thread-safe design for concurrent benchmarking
- Configurable iterations and data sources

## Dependencies

- `divan`: Benchmarking framework
- `pprof`: Profiling and flamegraph generation
- `clap`: CLI interface
- `tokio`: Async runtime for evaluation benchmarks
- FHIRPath crates: Core functionality being benchmarked
