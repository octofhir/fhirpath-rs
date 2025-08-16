# FHIRPath Performance Optimization Plan

## Analysis Summary

**Target Expression:** `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`

**Profiling Command:** `just profile-bundle "Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value"`

**Date:** 2025-08-16

## Performance Bottlenecks Identified

### 1. Excessive Memory Cloning
**Location:** `crates/fhirpath-registry/src/operations/lambda/where_fn.rs`
- **Lines 110, 113, 137:** Multiple `item.clone()` calls for each filtered item
- **Lines 265, 268, 278:** Redundant cloning in lambda context building
- **Impact:** Each `FhirPathValue::Collection(Arc<[FhirPathValue]>)` clone creates new Arc references

### 2. Context Creation Overhead
- Lambda context rebuilding for each collection item (lines 264-269)
- Redundant `context.clone()` followed by `with_input(item.clone())`
- Variable setting overhead with string allocations (`$this`, `$index`, `$total`)

### 3. Nested Collection Processing
- Multiple nested `where()` calls create cascading filter operations
- Each filter step processes entire collections independently
- No predicate pushdown or optimization across chained operations

## Memory Usage Patterns

### Current Architecture
1. **Collections:** `Collection(Arc<[FhirPathValue]>)` - Good for sharing but expensive to modify
2. **Strings:** `String(Arc<str>)` - Efficient for repeated strings
3. **JSON Values:** `ArcJsonValue` adds indirection overhead
4. **Resources:** `Resource(Arc<FhirResource>)` - Proper shared ownership

### Memory Layout Issues
- Deep cloning in filter chains
- Temporary collection allocations for intermediate results
- Lambda context recreation for each iteration

## Optimization Recommendations

### Priority 1: Memory Optimization (Immediate Impact)

#### 1.1 Eliminate Unnecessary Clones in `where_fn.rs`
```rust
// Current (lines 137, 278):
filtered_items.push(item.clone());

// Optimized approach:
// Use iterator-based filtering to avoid intermediate Vec allocation
// Implement copy-on-write semantics for filtered results
```

#### 1.2 Optimize Lambda Context Creation
```rust
// Current (lines 264-269):
let lambda_context = LambdaContextBuilder::new(context)
    .with_this(item.clone())        // <- Clone here
    .with_index(index as i64)
    .with_total(FhirPathValue::Integer(items.len() as i64))
    .with_input(item.clone())       // <- Clone again
    .build();

// Optimized:
// 1. Pre-allocate context builder and reuse
// 2. Cache variable names as Arc<str> to avoid string allocations
// 3. Use references where possible
```

#### 1.3 Implement Copy-on-Write Collections
```rust
// Replace Vec<FhirPathValue> with Cow<[FhirPathValue]>
// Defer cloning until mutation is needed
// Share immutable collections between operations
```

### Priority 2: Algorithm Optimization (25-35% Performance Gain)

#### 2.1 Predicate Pushdown
```rust
// Current:
Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile')

// Optimized:
// Combine multiple where() clauses into single filter pass
Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.where(system='phone' and use='mobile').exists())
```

#### 2.2 Early Termination for Boolean Operations
- Short-circuit evaluation for `and`/`or` operations
- Skip remaining items when result is deterministic
- Implement lazy evaluation chains

#### 2.3 Streaming Evaluation for Large Collections
```rust
// Replace eager collection materialization with iterators
// Process items on-demand rather than building intermediate collections
// Implement iterator fusion for chained operations
```

### Priority 3: Data Structure Optimization

#### 3.1 Value Interning System
```rust
// Intern common values to reduce memory usage:
// - Resource types: "Patient", "Observation", etc.
// - System codes: "phone", "email", etc.
// - Use codes: "home", "work", "mobile", etc.

static VALUE_INTERNER: LazyLock<ValueInterner> = LazyLock::new(ValueInterner::new);
```

#### 3.2 Lazy Collection Materialization
```rust
pub enum FhirPathValue {
    // ... existing variants
    LazyCollection(Box<dyn Iterator<Item = FhirPathValue> + Send>),
}
```

#### 3.3 SIMD Optimizations for Bulk Operations
- Vectorize string comparisons for primitive filtering
- Parallel processing for large collections (rayon integration)
- Specialized bulk operations for common patterns

### Priority 4: Caching and Memoization

#### 4.1 Expression-Level Caching
```rust
// Cache parsed expressions with weak references
// Memoize frequent subexpression results
// LRU cache for evaluation results of pure expressions
```

#### 4.2 Type-Aware Fast Paths
```rust
// Specialized handlers for common FHIR patterns:
// - Resource.resourceType comparisons
// - Primitive field access
// - Collection size operations
```

## Implementation Roadmap

### Phase 1: Memory Optimization (Week 1-2)
- [ ] Fix clone issues in `where_fn.rs`
- [ ] Implement COW collections
- [ ] Optimize lambda context creation
- [ ] Add memory usage benchmarks

### Phase 2: Algorithm Improvements (Week 3-4)
- [ ] Implement predicate pushdown
- [ ] Add early termination logic
- [ ] Streaming evaluation framework
- [ ] Performance regression tests

### Phase 3: Advanced Optimizations (Week 5-6)
- [ ] Value interning system
- [ ] Lazy evaluation implementation
- [ ] SIMD integration for bulk ops
- [ ] Comprehensive benchmarking

### Phase 4: Caching Layer (Week 7-8)
- [ ] Expression caching
- [ ] Result memoization
- [ ] Type-aware optimizations
- [ ] Production validation

## Expected Performance Gains

### Memory Usage
- **40-60% reduction** in memory allocations
- **30-50% lower** peak memory usage
- **Reduced GC pressure** from fewer temporary objects

### Execution Speed
- **25-35% faster** evaluation for complex nested queries
- **50-70% improvement** for repeated similar expressions
- **80%+ cache hit rate** for common patterns

### Scalability
- **Linear scaling** maintained for large collections
- **Reduced latency** for real-time applications
- **Better throughput** for batch processing

## Validation Strategy

### Benchmarking
```bash
# Before optimization
just profile-bundle "Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value"

# Micro-benchmarks for specific optimizations
cargo bench --bench where_function_optimization
cargo bench --bench memory_usage_comparison
cargo bench --bench collection_processing

# Memory profiling
just profile-memory "complex_expression"
```

### Testing
- Maintain 100% compatibility with existing test suite
- Add performance regression tests
- Memory leak detection with valgrind/heaptrack
- Load testing with large FHIR bundles

## Risk Assessment

### Low Risk
- Memory optimization (Priority 1)
- Early termination improvements
- Caching layer additions

### Medium Risk
- Algorithm changes (predicate pushdown)
- Data structure modifications
- SIMD integration

### High Risk
- Lazy evaluation system changes
- Major architectural refactoring
- Breaking API changes

## Success Metrics

### Performance Targets
- [ ] Memory usage < 60% of current baseline
- [ ] Evaluation speed > 130% of current performance
- [ ] Cache hit rate > 80% for repeated expressions
- [ ] Zero performance regressions in test suite

### Quality Targets
- [ ] Maintain 100% test coverage
- [ ] Zero new compiler warnings
- [ ] Documentation updated for new features
- [ ] Benchmark suite integrated into CI

---

**Generated on:** 2025-08-16  
**Next Review:** Weekly during implementation phases  
**Owner:** FHIRPath Core Team