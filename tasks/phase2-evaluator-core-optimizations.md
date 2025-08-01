# Phase 2: Evaluator Core Optimizations

**Goal**: Achieve 40-60% evaluator performance improvement  
**Timeline**: Weeks 3-5  
**Status**: Pending

## Task 2.1: Value System Redesign

### Task 2.1.1: Implement Copy-on-Write for FhirPathValue
- **File**: `src/model/value.rs`
- **Description**: Redesign FhirPathValue with Arc for zero-copy collections
- **Estimated Effort**: 12 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  pub enum FhirPathValue {
      Collection(Arc<[FhirPathValue]>), // Use Arc for zero-copy
      String(Arc<str>), // Interned strings
      // ... other variants
  }
  ```
- **Acceptance Criteria**:
  - Collections use Arc for zero-copy operations
  - String values are interned where beneficial
  - Cloning becomes O(1) for large collections
  - All existing functionality preserved

### Task 2.1.2: Add ValueRef with Cow Semantics
- **File**: `src/model/value.rs`
- **Description**: Create `ValueRef<'a>` for borrowed value operations
- **Estimated Effort**: 10 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  pub struct ValueRef<'a> {
      value: Cow<'a, FhirPathValue>,
  }
  
  impl<'a> ValueRef<'a> {
      pub fn borrowed(value: &'a FhirPathValue) -> Self;
      pub fn owned(value: FhirPathValue) -> Self;
  }
  ```
- **Acceptance Criteria**:
  - ValueRef supports both borrowed and owned values
  - Evaluator can work with borrowed values where possible
  - Significant reduction in cloning operations

### Task 2.1.3: Optimize Collection Operations
- **File**: `src/model/value.rs`
- **Description**: Implement efficient collection operations without unnecessary cloning
- **Estimated Effort**: 8 hours
- **Status**: Pending
- **Operations to optimize**:
  - Collection flattening
  - Item filtering
  - Collection concatenation
  - Duplicate removal
- **Acceptance Criteria**:
  - Collection operations avoid unnecessary cloning
  - Memory usage reduction for large collections
  - Performance improvement in collection-heavy expressions

## Task 2.2: Context Management Optimization

### Task 2.2.1: Implement Context Pooling
- **File**: `src/evaluator/context.rs`
- **Description**: Pool `EvaluationContext` instances to avoid allocations
- **Estimated Effort**: 8 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  struct ContextPool {
      pool: Vec<EvaluationContext>,
      max_size: usize,
  }
  
  impl ContextPool {
      fn acquire(&mut self) -> EvaluationContext;
      fn release(&mut self, context: EvaluationContext);
  }
  ```
- **Acceptance Criteria**:
  - Context instances are pooled and reused
  - Allocation reduction for context creation
  - Thread-safe implementation

### Task 2.2.2: Stack-Allocated Contexts for Simple Expressions
- **File**: `src/evaluator/context.rs`
- **Description**: Use stack allocation for simple expression contexts
- **Estimated Effort**: 6 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  enum ContextStorage {
      Stack(StackContext),
      Heap(Box<HeapContext>),
  }
  ```
- **Acceptance Criteria**:
  - Simple expressions use stack-allocated contexts
  - Complex expressions fall back to heap allocation
  - Performance improvement for simple expressions

### Task 2.2.3: Copy-on-Write Variable Scopes
- **File**: `src/evaluator/context.rs`
- **Description**: Implement CoW semantics for variable scopes
- **Estimated Effort**: 8 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  struct VariableScope {
      variables: Cow<'a, HashMap<String, FhirPathValue>>,
      parent: Option<&'a VariableScope>,
  }
  ```
- **Acceptance Criteria**:
  - Variable scopes use CoW semantics
  - Memory reduction for scope inheritance
  - Efficient variable resolution

## Task 2.3: Function Call Optimization

### Task 2.3.1: Pre-compile Function Signatures
- **File**: `src/registry/function.rs`
- **Description**: Pre-compile function signatures for faster dispatch
- **Estimated Effort**: 10 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  struct CompiledSignature {
      name: InternedString,
      arity: u8,
      dispatch_fn: fn(&[FhirPathValue], &EvaluationContext) -> Result<FhirPathValue>,
  }
  ```
- **Acceptance Criteria**:
  - Function signatures are pre-compiled
  - Function dispatch is faster
  - Reduced overhead in function calls

### Task 2.3.2: Implement Fast Paths for Common Functions
- **File**: `src/registry/functions/`
- **Description**: Create specialized implementations for common functions
- **Estimated Effort**: 16 hours
- **Status**: Pending
- **Functions to optimize**:
  - `where()` - filtering operations
  - `select()` - transformation operations  
  - `count()` - counting operations
  - `exists()` - existence checking
  - `first()`, `last()` - element access
- **Implementation**:
  ```rust
  trait FastPathFunction {
      fn can_fast_path(&self, args: &[ExpressionNode]) -> bool;
      fn fast_evaluate(&self, args: &[ExpressionNode], context: &EvaluationContext) -> Result<FhirPathValue>;
  }
  ```
- **Acceptance Criteria**:
  - Common functions have optimized fast paths
  - 50%+ performance improvement for common operations
  - Fall back to general implementation when needed

### Task 2.3.3: Use Const Generics for Arity-Specific Functions
- **File**: `src/registry/function.rs`
- **Description**: Implement const generic function wrappers for different arities
- **Estimated Effort**: 8 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  trait Function<const ARITY: usize> {
      fn evaluate(&self, args: [FhirPathValue; ARITY], context: &EvaluationContext) -> Result<FhirPathValue>;
  }
  ```
- **Acceptance Criteria**:
  - Functions are specialized by arity
  - Compile-time argument count checking
  - Performance improvement in function dispatch

## Task 2.4: Algorithm Improvements

### Task 2.4.1: Implement Expression Complexity Analysis
- **File**: `src/evaluator/analyzer.rs`
- **Description**: Analyze expression complexity to choose optimal evaluation strategy
- **Estimated Effort**: 12 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  enum ExpressionComplexity {
      Simple,    // Single property access
      Medium,    // Method chains
      Complex,   // Complex logic with multiple operations
  }
  
  fn analyze_complexity(expr: &ExpressionNode) -> ExpressionComplexity;
  ```
- **Acceptance Criteria**:
  - Expression complexity analysis implemented
  - Different evaluation strategies for different complexities
  - Performance improvement for simple expressions

### Task 2.4.2: Specialized Evaluators by Complexity
- **File**: `src/evaluator/specialized.rs`
- **Description**: Create specialized evaluators for different expression complexities
- **Estimated Effort**: 16 hours
- **Status**: Pending
- **Implementation**:
  ```rust
  trait SpecializedEvaluator {
      fn evaluate_simple(&self, expr: &SimpleExpr, input: &FhirPathValue) -> Result<FhirPathValue>;
      fn evaluate_medium(&self, expr: &MediumExpr, input: &FhirPathValue) -> Result<FhirPathValue>;
      fn evaluate_complex(&self, expr: &ComplexExpr, context: &EvaluationContext) -> Result<FhirPathValue>;
  }
  ```
- **Acceptance Criteria**:
  - Specialized evaluators for each complexity level
  - Automatic selection based on analysis
  - Performance improvement across all complexity levels

## Performance Validation

### Task 2.5: Add Evaluator Benchmarks
- **File**: `benches/evaluator_optimized_benchmark.rs`
- **Description**: Comprehensive benchmarks for evaluator optimizations
- **Estimated Effort**: 8 hours
- **Status**: Pending
- **Benchmarks**:
  - Value system operations (cloning, collection ops)
  - Context management overhead
  - Function call performance
  - End-to-end evaluation performance
  - Memory allocation patterns
- **Acceptance Criteria**:
  - Baseline measurements established
  - 40-60% improvement demonstrated
  - Memory usage reduction verified

## Success Metrics
- **Target**: 40-60% evaluator performance improvement
- **Baseline**: 7-9K ops/sec
- **Target**: 15-30K ops/sec  
- **Memory**: 30-50% reduction in allocations
- **Quality**: All existing tests pass, no regressions

## Dependencies
- Phase 1 parser optimizations completed
- `smallvec` crate added to dependencies
- Benchmark infrastructure in place

## Risk Mitigation
- Implement optimizations behind feature flags
- Extensive testing with official FHIRPath test suite
- Memory safety verification
- Performance regression monitoring
- Gradual rollout with fallback mechanisms