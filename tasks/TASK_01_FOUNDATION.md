# Task 1: Foundation - Unified Registry Architecture

**Status:** Not Started  
**Estimated Time:** 2-3 weeks  
**Priority:** Critical  
**Dependencies:** None

## Objective

Create the foundational unified registry architecture that combines function and operator registries into a single, async-first, high-performance system.

## Deliverables

### 1. Core Unified Registry Structure

```rust
// New file: crates/fhirpath-registry/src/unified_registry_v2.rs

/// Single unified registry for all FHIRPath operations
#[derive(Clone)]
pub struct FhirPathRegistry {
    /// All callable items (functions + operators) indexed by symbol
    operations: Arc<RwLock<FxHashMap<String, Arc<dyn FhirPathOperation>>>>,
    
    /// Enhanced metadata with unified type information
    metadata: Arc<RwLock<FxHashMap<String, OperationMetadata>>>,
    
    /// Performance-optimized async dispatch cache
    dispatch_cache: Arc<AsyncLruCache<DispatchKey, Arc<dyn FhirPathOperation>>>,
    
    /// LSP and tooling support
    lsp_provider: Arc<OperationLspProvider>,
    
    /// Performance statistics and metrics
    metrics: Arc<RegistryMetrics>,
}
```

### 2. Unified Operation Trait

```rust
// New file: crates/fhirpath-registry/src/unified_operation.rs

/// Unified trait for all FHIRPath callable operations
#[async_trait]
pub trait FhirPathOperation: Send + Sync {
    /// Operation identifier (function name or operator symbol)
    fn identifier(&self) -> &str;
    
    /// Operation type (Function, BinaryOperator, UnaryOperator)
    fn operation_type(&self) -> OperationType;
    
    /// Enhanced metadata for the operation
    fn metadata(&self) -> &OperationMetadata;
    
    /// Async evaluation - primary interface (non-blocking)
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError>;
    
    /// Optional sync evaluation for performance-critical paths
    /// Returns None if sync evaluation is not supported
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue, EvaluationError>> {
        None // Default: async-only
    }
    
    /// Check if operation supports sync evaluation
    fn supports_sync(&self) -> bool {
        false
    }
    
    /// Validate arguments before evaluation
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<(), EvaluationError>;
}
```

### 3. Unified Metadata System

```rust
// New file: crates/fhirpath-registry/src/unified_metadata.rs

/// Unified metadata for operations (functions and operators)
#[derive(Debug, Clone)]
pub struct OperationMetadata {
    /// Basic operation information
    pub basic: BasicOperationInfo,
    
    /// Type constraints and signatures
    pub types: TypeConstraints,
    
    /// Performance characteristics
    pub performance: PerformanceMetadata,
    
    /// LSP support information
    pub lsp: LspMetadata,
    
    /// Operation-specific metadata
    pub specific: OperationSpecificMetadata,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Function,
    BinaryOperator { precedence: u8, associativity: Associativity },
    UnaryOperator,
}

#[derive(Debug, Clone)]
pub enum OperationSpecificMetadata {
    Function(FunctionMetadata),
    Operator(OperatorMetadata),
}
```

### 4. High-Performance Async Cache

```rust
// New file: crates/fhirpath-registry/src/async_cache.rs

/// High-performance async LRU cache for operation dispatch
pub struct AsyncLruCache<K, V> {
    inner: Arc<RwLock<LruCache<K, V>>>,
    metrics: Arc<CacheMetrics>,
}

impl<K: Hash + Eq + Clone, V: Clone> AsyncLruCache<K, V> {
    /// Get item from cache (async-friendly)
    pub async fn get(&self, key: &K) -> Option<V> {
        // Lock-free read when possible
        self.inner.read().await.get(key).cloned()
    }
    
    /// Insert item into cache
    pub async fn insert(&self, key: K, value: V) {
        let mut cache = self.inner.write().await;
        cache.put(key, value);
        self.metrics.record_insertion();
    }
    
    /// Clear cache
    pub async fn clear(&self) {
        self.inner.write().await.clear();
        self.metrics.record_clear();
    }
}
```

### 5. Migration Utilities

```rust
// New file: crates/fhirpath-registry/src/migration_utils.rs

/// Utilities for migrating from old registry system to unified system
pub struct RegistryMigrationHelper;

impl RegistryMigrationHelper {
    /// Convert old function registry to unified registry
    pub async fn migrate_function_registry(
        old_registry: &UnifiedFunctionRegistry,
        unified_registry: &mut FhirPathRegistry,
    ) -> Result<MigrationStats, MigrationError> {
        // Migration logic
    }
    
    /// Convert old operator registry to unified registry
    pub async fn migrate_operator_registry(
        old_registry: &UnifiedOperatorRegistry,
        unified_registry: &mut FhirPathRegistry,
    ) -> Result<MigrationStats, MigrationError> {
        // Migration logic
    }
    
    /// Create standard registry with all built-in operations
    pub async fn create_standard_registry() -> FhirPathRegistry {
        // Build unified registry with all standard operations
    }
}
```

## Implementation Steps

### Step 1: Create Core Data Structures
- [ ] Implement `FhirPathRegistry` struct
- [ ] Implement `OperationMetadata` and related types
- [ ] Create async-optimized cache implementation
- [ ] Add comprehensive error types

### Step 2: Implement Unified Operation Trait
- [ ] Define `FhirPathOperation` trait with async-first design
- [ ] Create operation type enumeration
- [ ] Implement validation framework
- [ ] Add support for both sync and async execution paths

### Step 3: Build Registration System
- [ ] Implement operation registration methods
- [ ] Add bulk registration support
- [ ] Create metadata builder pattern
- [ ] Add validation for duplicate registrations

### Step 4: Create Migration Infrastructure
- [ ] Build migration utilities for existing registries
- [ ] Create compatibility adapters
- [ ] Implement registry introspection tools
- [ ] Add migration validation

### Step 5: Performance Optimizations
- [ ] Implement lock-free reads where possible
- [ ] Add operation dispatch caching
- [ ] Optimize metadata storage
- [ ] Implement lazy loading for operations

### Step 6: Testing Infrastructure
- [ ] Create comprehensive unit tests
- [ ] Add performance benchmarks
- [ ] Build integration tests
- [ ] Create migration validation tests

## Performance Requirements

### Dispatch Performance
- Function lookup: <50ns (cached)
- Operation dispatch: <100ns total
- Registry creation: <10ms for standard registry
- Memory usage: <50% vs current dual system

### Async Performance
- Zero blocking operations in hot paths
- Minimal async overhead for sync-capable operations
- Efficient await points
- Lock contention minimization

## Testing Strategy

### Unit Tests
- Operation registration and lookup
- Metadata validation
- Cache performance
- Error handling

### Integration Tests  
- Migration from old registries
- Compatibility with existing evaluator
- Performance benchmarks
- Memory usage validation

### Performance Tests
- Dispatch latency benchmarks
- Throughput measurements
- Memory allocation profiling
- Async overhead analysis

## Success Criteria

1. **API Simplicity**: Single registry creation function
2. **Performance**: Dispatch <100ns, registry creation <10ms
3. **Memory Efficiency**: <50% memory usage vs dual system
4. **Async-First**: All operations non-blocking
5. **Migration Support**: Clean migration from existing systems
6. **Test Coverage**: >95% code coverage maintained

## Risks and Mitigation

### Performance Risk
- **Risk**: Unified registry slower than specialized registries
- **Mitigation**: Comprehensive benchmarking, performance-first design

### Complexity Risk
- **Risk**: Unified system more complex than expected
- **Mitigation**: Iterative design, simple core with extensions

### Migration Risk
- **Risk**: Difficult migration from existing systems
- **Mitigation**: Comprehensive migration tools, parallel system support

## Files to Create

1. `crates/fhirpath-registry/src/unified_registry_v2.rs` - Core registry
2. `crates/fhirpath-registry/src/unified_operation.rs` - Operation trait
3. `crates/fhirpath-registry/src/unified_metadata.rs` - Metadata system
4. `crates/fhirpath-registry/src/async_cache.rs` - Performance cache
5. `crates/fhirpath-registry/src/migration_utils.rs` - Migration helpers

## Dependencies

- `tokio` - Async runtime support
- `lru` - LRU cache implementation  
- `async-trait` - Async trait support
- `rustc-hash` - Fast hashing
- `thiserror` - Error handling

## Next Task Dependencies

This task is a prerequisite for:
- Task 2: Core Operations Migration
- Task 3: Complete Migration  
- Task 4: Cleanup and Optimization