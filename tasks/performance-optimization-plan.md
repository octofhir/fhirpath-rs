# FHIRPath Performance Optimization Plan

**Generated from flamegraph analysis on:** 2025-08-11  
**Test expression:** `Bundle.entry.resource.where(resourceType='Encounter' and meta.profile.contains('http://fhir.mimic.mit.edu/StructureDefinition/mimic-encounter-icu')).partOf.reference`  
**Dataset:** `benches/fixtures/large.json`  
**Flamegraph:** `opt/flamegraphs/baseline_flamegraph.svg`  

## 🔥 Performance Analysis Results

### Current Performance Characteristics
- **Total execution time:** ~3.87 seconds for single evaluation
- **CPU utilization:** 189% (multi-core usage)
- **Expression correctness:** ✅ Returns expected 2 reference values
- **Primary bottleneck:** Memory management (~75% of execution time)

### Top Performance Bottlenecks Identified

**1. Memory Management & Cleanup (~75% of total time)**
- `core::ptr::drop_in_place` operations: **~24.6% combined**
- `alloc::sync::Arc::drop_slow`: **~22.6% combined** 
- `alloc::collections::btree::map::BTreeMap` cloning: **~12.8%**
- `serde_json::Value` drops: **~15% combined**

**2. Data Structure Operations (~25% of time)**
- `alloc::collections::btree::map::IntoIter::dying_next`: **~8.5% combined**
- BTreeMap clone operations for evaluation contexts

## 🚀 Optimization Recommendations

### Priority 1: Reduce Memory Allocations & Cloning (High Impact)

**Problem:** Excessive cloning of evaluation contexts and JSON values  
**Expected gain:** 40-60% improvement

**Implementation locations:**
- `src/evaluator/context.rs`
- `src/model/value.rs`

**Specific changes:**
```rust
// Current: Heavy Arc<EvaluationContext> cloning
// Solution: Implement copy-on-write or reference sharing

// Replace excessive Arc usage with:
- Use Rc<RefCell<>> instead of Arc<> for single-threaded contexts
- Implement lazy cloning - only clone when context actually changes  
- Add context pooling to reuse allocated contexts
```

### Priority 2: Optimize JSON Value Handling (High Impact)

**Problem:** Heavy serde_json::Value drops indicate excessive JSON manipulation  
**Expected gain:** 20-30% improvement

**Implementation locations:**
- `src/model/value.rs`
- `src/evaluator/engine.rs`

**Specific changes:**
```rust
// Current: Full JSON materialization for all operations
// Solution: Streaming evaluation with minimal JSON materialization

- Use serde_json::RawValue for unprocessed JSON segments
- Implement zero-copy JSON navigation where possible
- Cache frequently accessed JSON paths
```

### Priority 3: Reduce Arc Drop Overhead (Medium Impact)

**Problem:** Arc::drop_slow indicates high reference counting overhead  
**Expected gain:** 25-35% improvement

**Implementation locations:**
- `src/model/arc_pool.rs` (already exists but underutilized)
- `src/evaluator/context.rs`

**Specific changes:**
```rust
// Current: Excessive Arc reference counting
// Solution: Use object pooling and weak references

- Implement custom arena allocator for short-lived objects
- Use Weak<> references for non-owning relationships  
- Pool and reuse Arc-wrapped objects via existing arc_pool
```

### Priority 4: Optimize BTreeMap Operations (Medium Impact)

**Problem:** Frequent BTreeMap iteration and cloning  
**Expected gain:** 15-25% improvement

**Implementation locations:**
- `src/model/value.rs:Collection`
- `src/evaluator/context.rs`

**Specific changes:**
```rust
// Current: BTreeMap for all key-value operations
// Solution: Use more efficient data structures for specific use cases

- Replace BTreeMap with FxHashMap for small collections
- Use SmallVec for collections with predictable small sizes
- Implement copy-on-write for rarely-modified maps
```

### Priority 5: Expression-Specific Optimizations (Low-Medium Impact)

**Problem:** Full traversal of all Bundle entries  
**Expected gain:** 10-20% improvement

**Implementation locations:**
- `src/evaluator/engine.rs`
- `src/registry/functions/filtering/where.rs`

**Specific changes:**
```rust
// Current: Full traversal for filtering operations
// Solution: Early filtering and short-circuit evaluation

- Add fast-path for common resource type filters
- Implement streaming evaluation for large collections  
- Cache compiled expressions to avoid re-parsing
```

## 📊 Implementation Plan

### Phase 1: Immediate (High ROI)
- [ ] Context pooling and lazy cloning
- [ ] Replace excessive Arc usage with lighter alternatives
- [ ] Expected improvement: 30-40%

### Phase 2: Short-term (2-4 weeks)
- [ ] JSON value optimization and streaming
- [ ] BTreeMap replacement for hot paths
- [ ] Expected additional improvement: 20-30%

### Phase 3: Medium-term (1-2 months)
- [ ] Expression-specific fast paths
- [ ] Advanced caching strategies
- [ ] Expected additional improvement: 10-15%

## 🎯 Success Metrics

**Target performance goals:**
- **Overall improvement:** 50-70% faster execution
- **Memory usage reduction:** 40-60% less allocations
- **Specific target:** Reduce 3.87s execution to <1.5s

**Benchmark tracking:**
- Re-run flamegraph analysis after each phase
- Use `just bench` to track performance regression
- Monitor memory allocation patterns

## 🔍 Verification Steps

1. **After each phase:**
   ```bash
   # Regenerate flamegraph
   RUSTFLAGS='-C force-frame-pointers=yes' cargo build --release --bin flamegraph_baseline --features profiling
   ./target/release/flamegraph_baseline
   
   # Run performance benchmarks
   just bench
   
   # Test correctness
   just test-official
   ```

2. **Profile memory usage:**
   ```bash
   # Use memory profiling tools
   cargo build --release --features profiling
   # Add memory profiling to flamegraph_baseline.rs
   ```

## 📋 Implementation Notes

- **Maintain compatibility:** All optimizations must pass existing test suite
- **Incremental approach:** Implement one optimization at a time to isolate performance gains
- **Profile-guided:** Use flamegraph analysis to validate each optimization
- **Documentation:** Update performance characteristics in README.md after completion

## 🚨 Risks & Mitigations

1. **Breaking changes in memory model**
   - Mitigation: Comprehensive testing with official FHIRPath test suites
   
2. **Complexity increase in codebase**
   - Mitigation: Document new patterns and provide examples
   
3. **Performance regression in other expressions**
   - Mitigation: Expand benchmark coverage before starting optimizations

---

**Status:** Planned  
**Next action:** Begin Phase 1 implementation  
**Owner:** Development team  
**Review date:** After each phase completion