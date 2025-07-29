# FHIRPath Performance Benchmarks

This document describes the benchmark suite for the FHIRPath parser and tokenizer, providing performance metrics and optimization targets.

## 🚀 Quick Start

### Run Compact Benchmark (Recommended)
```bash
./run_benchmarks.sh
```

### Run Full Benchmark Suite
```bash
./run_benchmarks.sh --full
```

### Run Specific Benchmarks
```bash
# New compact benchmark (best overview)
cargo bench --bench compact_performance_benchmark

# Individual benchmarks
cargo bench --bench tokenizer_only_benchmark
cargo bench --bench parser_benchmark
cargo bench --bench parser_only
```

## 📊 Performance Targets & Results

### Current Performance (as of latest update)

| Component | Target | Achieved | Status |
|-----------|--------|----------|---------|
| **Tokenizer** | 10M ops/sec | **13M ops/sec** | ✅ **Exceeded** |
| **Parser** | 1M ops/sec | **1.3M ops/sec** | ✅ **Exceeded** |

### Test Expression
Standard benchmark expression: `"Patient.name.where(use = 'official')"`

## 🏗️ Architecture Improvements

### Pratt Parser Implementation
- **Performance Gain**: 1.16x improvement over recursive descent
- **Precedence Levels**: 12 levels (Implies to Invocation)
- **Optimizations**: Zero-allocation parsing, aggressive inlining, branch prediction optimization

### Tokenizer Optimizations
- **Zero-copy**: String slices with lifetime parameters
- **Hot Path**: Most common tokens optimized for branch prediction
- **Memory**: Minimal allocations during tokenization

## 📈 Benchmark Descriptions

### 1. Compact Performance Benchmark (`compact_performance_benchmark.rs`)
**🔧 Primary benchmark for development**
- **Multi-complexity**: Tests simple to very complex expressions
- **Direct Comparisons**: Tokenizer vs Parser performance
- **Throughput Targets**: Validates performance goals
- **Results Display**: Clear ops/second metrics

### 2. Tokenizer Only Benchmark (`tokenizer_only_benchmark.rs`)
- **Focused**: Pure tokenization performance
- **Expressions**: Multiple complexity levels
- **Metrics**: Token count and operations/second

### 3. Parser Benchmark (`parser_benchmark.rs`)
- **Combined**: Both tokenizer and parser performance
- **Comprehensive**: Multiple expression types
- **Comparison**: Direct tokenizer vs parser comparison

### 4. Parser Only Benchmark (`parser_only.rs`)
- **Pure Parser**: Complete parsing pipeline
- **Validation**: Performance target verification

## 🎯 Performance Optimization Guidelines

### For Tokenizer Performance
- Use zero-copy string slices
- Optimize hot paths (identifiers, operators)
- Minimize allocations in tight loops
- Pre-allocate buffers where possible

### For Parser Performance
- Leverage Pratt parsing for operator precedence
- Use compile-time precedence tables (`#[repr(u8)]`)
- Aggressive inlining on hot functions (`#[inline(always)]`)
- Branch prediction friendly code organization

## 🧪 Expression Complexity Levels

| Level | Expression | Use Case |
|-------|------------|----------|
| **Simple** | `Patient.name` | Basic property access |
| **Medium** | `Patient.name.where(use = 'official')` | Filtered access |
| **Complex** | `Patient.name.where(use = 'official').given.first()` | Chained operations |
| **Very Complex** | `Bundle.entry.resource.where($this is Patient).name.where(use = 'official').given` | Real-world complexity |

## 🔧 Running Custom Benchmarks

### Add New Benchmark
1. Create benchmark file in `benches/` directory
2. Add `[[bench]]` section to `Cargo.toml`
3. Use `std::hint::black_box()` (not deprecated `criterion::black_box`)
4. Follow the pattern in `compact_performance_benchmark.rs`

### Example Custom Benchmark
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput}; 
use std::hint::black_box;
use fhirpath_parser::parse_expression;

fn my_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_group");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("my_test", |b| {
        b.iter(|| {
            black_box(parse_expression(black_box("Patient.name")))
        })
    });
    
    group.finish();
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

## 🎯 Performance Monitoring

### Continuous Integration
- Benchmarks run on every significant parser change
- Performance regression detection
- Target validation (must meet minimum thresholds)

### Local Development
- Use `./run_benchmarks.sh` for quick validation
- Profile with `cargo bench --bench compact_performance_benchmark`
- HTML reports available in `target/criterion/`

## 📝 Notes

### Dependencies
- **Fixed Deprecation**: Replaced `criterion::black_box` with `std::hint::black_box`
- **Updated Libraries**: All dependencies updated to latest versions
- **Clean Compilation**: No deprecation warnings

### Best Practices
- Always use `black_box()` to prevent compiler optimizations
- Test with realistic expression complexity
- Monitor both absolute performance and relative comparisons
- Use appropriate sample sizes for statistical significance

---

*For questions about benchmarks, see the implementation in `benches/` directory or run `./run_benchmarks.sh` for quick performance overview.*