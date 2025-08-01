# ADR-006: Performance Optimization for Parser and Evaluator

## Status
Proposed

## Context

Analysis of the current FHIRPath implementation reveals significant performance opportunities in both the parser and evaluator components. Based on benchmark results and code analysis, the system shows:

### Current Performance Baseline
- **Tokenizer**: 10-40M ops/sec (excellent performance)
- **Parser**: 1.4-6M ops/sec (good performance with optimization opportunities)  
- **Evaluator**: 7-9K ops/sec (performance bottleneck)

The evaluator is the primary performance bottleneck, running 100-1000x slower than the parser components.

### Key Performance Issues Identified

#### Parser Issues
1. **String allocations**: Heavy use of `String::new()` and string formatting in error paths
2. **Recursive call overhead**: Deep method call chains for complex expressions
3. **Token discrimination**: Using `std::mem::discriminant()` for token matching is suboptimal
4. **Memory layout**: Large enum variants cause cache misses

#### Evaluator Issues  
1. **Excessive cloning**: `FhirPathValue` cloning throughout evaluation chains
2. **Context switching overhead**: Frequent `EvaluationContext` creation and manipulation
3. **HashMap lookups**: Variable and function resolution through hash maps
4. **Collection flattening**: Repeated collection unwrapping/wrapping operations
5. **Dynamic dispatch**: Heavy use of trait objects and dynamic function calls

## Decision

Implement a multi-phase performance optimization strategy targeting both parser and evaluator components.

## Detailed Performance Optimization Plan

### Phase 1: Parser Optimizations (Expected 15-30% improvement)

#### 1.1 Token Matching Optimization
- Replace `std::mem::discriminant()` with direct pattern matching
- Use const lookup tables for operator precedence
- Implement token interning for frequently used identifiers

```rust
// Current: std::mem::discriminant() comparison
// Optimized: Direct enum discriminant comparison
#[repr(u8)]
enum TokenKind {
    Identifier = 1,
    String = 2,
    // ... other variants
}
```

#### 1.2 Memory Layout Optimization
- Reduce enum variant sizes through boxing of large variants
- Use `SmallVec` for argument lists (most functions have ≤4 args)
- Implement string interning for identifiers and operators

#### 1.3 Error Path Optimization
- Pre-allocate common error messages
- Use `Cow<'static, str>` for error strings
- Lazy error message formatting

### Phase 2: Evaluator Core Optimizations (Expected 40-60% improvement)

#### 2.1 Value System Redesign
```rust
// Current: Heavy cloning
pub enum FhirPathValue {
    Collection(Arc<[FhirPathValue]>), // Use Arc for zero-copy collections
    // ...
}

// Add copy-on-write semantics
pub struct ValueRef<'a> {
    value: Cow<'a, FhirPathValue>,
}
```

#### 2.2 Context Management Optimization
- Pool `EvaluationContext` instances to avoid allocations
- Use stack-allocated contexts for simple expressions
- Implement copy-on-write for variable scopes

#### 2.3 Function Call Optimization
- Pre-compile function signatures for faster dispatch
- Implement specialized fast paths for common functions (`where`, `select`, `count`)
- Use const generics for arity-specific function implementations

### Phase 3: Advanced Optimizations (Expected 30-50% additional improvement)

#### 3.1 Expression Compilation
- Implement bytecode compilation for complex expressions
- Create specialized evaluators for common expression patterns
- Add constant folding for literal expressions

#### 3.2 Memory Management
- Custom arena allocator for temporary values during evaluationЫздше ФВК 
- Object pooling for frequently allocated types
- Stack-based evaluation for simple expressions

#### 3.3 Algorithmic Improvements
- Implement lazy evaluation for collection operations
- Add streaming evaluation for large datasets
- Use SIMD instructions for numeric operations where applicable

### Phase 4: Specialization and Caching (Expected 20-40% additional improvement)

#### 4.1 Expression Analysis and Specialization
```rust
enum ExpressionComplexity {
    Simple,    // Single property access: Patient.name
    Medium,    // Method chains: Patient.name.where(use='official')
    Complex,   // Complex logic: Patient.name.where(use='official').select(given + ' ' + family)
}

// Generate specialized evaluators based on complexity
trait SpecializedEvaluator {
    fn evaluate_simple(&self, expr: &SimpleExpr, input: &FhirPathValue) -> Result<FhirPathValue>;
    fn evaluate_medium(&self, expr: &MediumExpr, input: &FhirPathValue) -> Result<FhirPathValue>;
    fn evaluate_complex(&self, expr: &ComplexExpr, context: &EvaluationContext) -> Result<FhirPathValue>;
}
```

#### 4.2 Result Caching
- Implement LRU cache for frequently evaluated expressions
- Cache parsed expression trees
- Add memoization for pure function calls

#### 4.3 Hot Path Optimization
- Profile-guided optimization for most common expression patterns
- Branch prediction hints for frequent code paths
- Inline critical functions aggressively

## Implementation Strategy

### Phase 1: Foundation (Weeks 1-2)
1. Implement token optimization and memory layout improvements
2. Add benchmarking infrastructure for regression testing
3. Create performance baseline measurements

### Phase 2: Core Optimizations (Weeks 3-5)
1. Redesign value system with copy-on-write semantics
2. Optimize context management and function dispatch
3. Implement specialized evaluators for common patterns

### Phase 3: Advanced Features (Weeks 6-8)
1. Add expression compilation and bytecode generation
2. Implement memory management optimizations
3. Add algorithmic improvements (lazy evaluation, streaming)

### Phase 4: Specialization (Weeks 9-10)
1. Implement expression complexity analysis
2. Add result caching and memoization
3. Profile-guided optimization based on real-world usage

## Success Metrics

### Performance Targets
- **Parser**: 2-8M ops/sec (25-35% improvement)
- **Evaluator**: 15-30K ops/sec (100-250% improvement)
- **Memory usage**: 30-50% reduction in allocations
- **Latency**: Sub-millisecond evaluation for simple expressions

### Quality Metrics
- Maintain 100% compatibility with FHIRPath specification
- All existing tests must pass
- No regressions in error reporting quality
- Memory safety preserved (no unsafe code unless absolutely necessary)

## Risk Mitigation

### Technical Risks
1. **Complexity increase**: Mitigate through extensive testing and documentation
2. **Memory safety**: Use Rust's type system and avoid unsafe code where possible
3. **Maintenance burden**: Implement optimizations incrementally with fallback paths

### Compatibility Risks
1. **API changes**: Maintain backwards compatibility through adapter layers
2. **Behavior changes**: Extensive regression testing with official test suites
3. **Performance regressions**: Continuous benchmarking and performance monitoring

## Alternative Approaches Considered

### 1. Complete Rewrite
- **Pros**: Clean slate, optimal design
- **Cons**: High risk, long timeline, potential compatibility issues
- **Decision**: Rejected due to risk and timeline constraints

### 2. JIT Compilation
- **Pros**: Maximum performance potential
- **Cons**: Complexity, compile-time overhead, memory usage
- **Decision**: Deferred to future consideration after core optimizations

### 3. Multi-threading
- **Pros**: Parallelization of collection operations
- **Cons**: Complexity, synchronization overhead, API changes
- **Decision**: Considered for Phase 5 after core optimizations prove successful

## Consequences

### Positive
- Significant performance improvements across all components
- Better resource utilization and lower memory footprint
- Improved user experience with faster query execution
- Stronger foundation for future optimizations

### Negative
- Increased code complexity requiring careful maintenance
- Longer implementation timeline (10 weeks estimated)
- Risk of introducing subtle bugs during optimization
- Higher barrier to entry for new contributors

### Neutral
- No changes to public API during optimization phases
- Existing functionality preserved throughout implementation
- Performance monitoring becomes critical part of development process

## Implementation Notes

### Testing Strategy
- Maintain existing test suite with 100% pass rate
- Add performance regression tests
- Implement fuzzing for edge cases
- Create stress tests for memory usage patterns

### Documentation Updates
- Update architecture documentation with new designs
- Create performance tuning guide for users
- Document optimization techniques for future development

### Monitoring and Metrics
- Add performance telemetry collection
- Create automated benchmark runs in CI/CD
- Set up alerting for performance regressions

This ADR provides a comprehensive roadmap for achieving significant performance improvements while maintaining the reliability and correctness of the FHIRPath implementation.