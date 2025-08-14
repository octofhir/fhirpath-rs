# Task 2: Core Operations Migration

**Status:** Not Started  
**Estimated Time:** 3-4 weeks  
**Priority:** Critical  
**Dependencies:** Task 1 (Foundation)

## Objective

Migrate the most critical and commonly used functions and operators to the new unified registry system, ensuring performance and compatibility while maintaining all existing functionality.

## Deliverables

### 1. Core Arithmetic Operators Migration

```rust
// New file: crates/fhirpath-registry/src/operations/arithmetic.rs

/// Unified arithmetic operation implementations
pub struct ArithmeticOperations;

impl ArithmeticOperations {
    pub fn register_all(registry: &mut FhirPathRegistry) {
        registry.register(AdditionOperation::new()).await;
        registry.register(SubtractionOperation::new()).await;
        registry.register(MultiplicationOperation::new()).await;
        registry.register(DivisionOperation::new()).await;
        registry.register(ModuloOperation::new()).await;
    }
}

/// Addition operation (+) - supports both binary and unary
pub struct AdditionOperation;

#[async_trait]
impl FhirPathOperation for AdditionOperation {
    fn identifier(&self) -> &str { "+" }
    
    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            precedence: 12, 
            associativity: Associativity::Left 
        }
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        // High-performance async implementation
        match args.len() {
            2 => self.evaluate_binary(&args[0], &args[1], context).await,
            1 => self.evaluate_unary(&args[0], context).await,
            _ => Err(EvaluationError::InvalidArgumentCount { 
                expected: "1 or 2", 
                actual: args.len() 
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue, EvaluationError>> {
        // Synchronous fast path for numeric types
        match args {
            [FhirPathValue::Integer(a), FhirPathValue::Integer(b)] => {
                Some(Ok(FhirPathValue::Integer(a + b)))
            }
            [FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)] => {
                Some(Ok(FhirPathValue::Decimal(a + b)))
            }
            _ => None // Fallback to async for complex cases
        }
    }
    
    fn supports_sync(&self) -> bool { true }
}
```

### 2. Core Collection Functions Migration

```rust
// New file: crates/fhirpath-registry/src/operations/collection.rs

/// High-frequency collection operations
pub struct CollectionOperations;

impl CollectionOperations {
    pub async fn register_all(registry: &mut FhirPathRegistry) {
        // Most critical functions first
        registry.register(CountFunction::new()).await;
        registry.register(EmptyFunction::new()).await;
        registry.register(ExistsFunction::new()).await;
        registry.register(FirstFunction::new()).await;
        registry.register(LastFunction::new()).await;
        registry.register(SingleFunction::new()).await;
    }
}

/// Count function - most frequently used collection function
pub struct CountFunction;

#[async_trait]
impl FhirPathOperation for CountFunction {
    fn identifier(&self) -> &str { "count" }
    
    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::OnceLock<OperationMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| {
            OperationMetadata {
                basic: BasicOperationInfo {
                    name: "count".to_string(),
                    description: "Returns the count of elements in a collection".to_string(),
                    category: OperationCategory::Collection,
                    pure: true,
                },
                types: TypeConstraints {
                    input_types: vec![TypePattern::Collection(Box::new(TypePattern::Any))],
                    output_type: TypePattern::Exact(TypeInfo::Integer),
                    min_args: 0,
                    max_args: Some(0),
                },
                performance: PerformanceMetadata {
                    complexity: PerformanceComplexity::O1,
                    supports_sync: true,
                    cacheable: true,
                },
                lsp: LspMetadata {
                    completion_priority: 1,
                    snippet: "count()".to_string(),
                    examples: vec!["Patient.name.count()".to_string()],
                },
                specific: OperationSpecificMetadata::Function(FunctionMetadata {
                    supports_lambda: false,
                    lambda_args: vec![],
                })
            }
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArgumentCount { 
                expected: "0", 
                actual: args.len() 
            });
        }
        
        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        
        Ok(FhirPathValue::Integer(count as i64))
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue, EvaluationError>> {
        // Count is always synchronous
        Some(tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                self.evaluate(args, context)
            )
        }))
    }
    
    fn supports_sync(&self) -> bool { true }
}
```

### 3. Core String Functions Migration

```rust
// New file: crates/fhirpath-registry/src/operations/string.rs

/// Essential string operations
pub struct StringOperations;

impl StringOperations {
    pub async fn register_all(registry: &mut FhirPathRegistry) {
        registry.register(LengthFunction::new()).await;
        registry.register(ContainsFunction::new()).await;
        registry.register(StartsWithFunction::new()).await;
        registry.register(EndsWithFunction::new()).await;
        registry.register(SubstringFunction::new()).await;
    }
}

/// Length function - high-frequency string operation
pub struct LengthFunction;

#[async_trait]
impl FhirPathOperation for LengthFunction {
    fn identifier(&self) -> &str { "length" }
    
    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        if !args.is_empty() {
            return Err(EvaluationError::InvalidArgumentCount { 
                expected: "0", 
                actual: args.len() 
            });
        }
        
        match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Integer(s.chars().count() as i64))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::InvalidInputType { 
                expected: "String", 
                actual: context.input.type_name() 
            })
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue], 
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue, EvaluationError>> {
        // String length is synchronous
        Some(tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                self.evaluate(args, context)
            )
        }))
    }
    
    fn supports_sync(&self) -> bool { true }
}
```

### 4. Evaluator Engine Integration

```rust
// Modified file: crates/fhirpath-evaluator/src/engine.rs

impl FhirPathEvaluator {
    /// Create evaluator with unified registry
    pub async fn new_unified() -> Result<Self, EvaluationError> {
        let registry = FhirPathRegistry::new_standard().await;
        Ok(Self {
            registry,
            // ... other fields
        })
    }
    
    /// Evaluate function using unified registry
    async fn evaluate_function_call(
        &self,
        name: &str,
        args: Vec<FhirPathValue>,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        // Try sync path first for performance
        if let Some(result) = self.registry.try_evaluate_sync(name, &args, context) {
            return result;
        }
        
        // Fallback to async evaluation
        self.registry.evaluate(name, &args, context).await
    }
    
    /// Evaluate binary operator using unified registry
    async fn evaluate_binary_operation(
        &self,
        operator: &str,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        let args = vec![left, right];
        
        // Try sync path for numeric operations
        if let Some(result) = self.registry.try_evaluate_sync(operator, &args, context) {
            return result;
        }
        
        // Async evaluation for complex cases
        self.registry.evaluate(operator, &args, context).await
    }
}
```

### 5. Standard Registry Builder

```rust
// New file: crates/fhirpath-registry/src/standard_registry.rs

/// Builder for standard FHIRPath registry with all built-in operations
pub struct StandardRegistryBuilder {
    registry: FhirPathRegistry,
    config: RegistryConfig,
}

impl StandardRegistryBuilder {
    /// Create new builder with default configuration
    pub fn new() -> Self {
        Self {
            registry: FhirPathRegistry::new(),
            config: RegistryConfig::default(),
        }
    }
    
    /// Configure async cache size
    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.cache_size = size;
        self
    }
    
    /// Enable/disable sync fast paths
    pub fn enable_sync_fastpath(mut self, enabled: bool) -> Self {
        self.config.sync_fastpath = enabled;
        self
    }
    
    /// Build registry with all standard operations
    pub async fn build(mut self) -> FhirPathRegistry {
        // Configure registry
        self.registry.configure(self.config).await;
        
        // Register core operations in priority order
        ArithmeticOperations::register_all(&mut self.registry).await;
        CollectionOperations::register_all(&mut self.registry).await;
        StringOperations::register_all(&mut self.registry).await;
        ComparisonOperations::register_all(&mut self.registry).await;
        LogicalOperations::register_all(&mut self.registry).await;
        
        // Additional operations
        TypeOperations::register_all(&mut self.registry).await;
        MathOperations::register_all(&mut self.registry).await;
        DateTimeOperations::register_all(&mut self.registry).await;
        FhirOperations::register_all(&mut self.registry).await;
        
        self.registry
    }
}

/// Convenience function for creating standard registry
pub async fn create_standard_registry() -> FhirPathRegistry {
    StandardRegistryBuilder::new().build().await
}
```

## Implementation Steps

### Step 1: Migrate Core Arithmetic Operators
- [ ] Implement addition, subtraction, multiplication, division
- [ ] Add sync fast paths for numeric operations  
- [ ] Comprehensive test coverage
- [ ] Performance benchmarking

### Step 2: Migrate Essential Collection Functions
- [ ] Implement count, empty, exists, first, last, single
- [ ] Optimize for common collection types
- [ ] Add async/sync hybrid evaluation
- [ ] Integration with existing tests

### Step 3: Migrate Core String Functions  
- [ ] Implement length, contains, startsWith, endsWith, substring
- [ ] Unicode-aware string operations
- [ ] Performance optimization for string operations
- [ ] Comprehensive string handling tests

### Step 4: Update Evaluator Engine
- [ ] Modify evaluator to use unified registry
- [ ] Implement hybrid sync/async dispatch
- [ ] Update function call evaluation
- [ ] Update operator evaluation

### Step 5: Create Standard Registry Builder
- [ ] Implement configurable registry builder
- [ ] Add convenience functions
- [ ] Performance tuning and optimization
- [ ] Documentation and examples

### Step 6: Compatibility and Migration
- [ ] Create migration utilities from old systems
- [ ] Ensure API compatibility where possible
- [ ] Update dependent crates
- [ ] Comprehensive integration testing

## Performance Targets

### Operation Performance
- Arithmetic operators: <25ns (sync path)
- Collection functions: <50ns (cached)
- String functions: <100ns average
- Registry lookup: <50ns (cached)

### Memory Efficiency
- Registry creation: <5MB base memory
- Operation metadata: <1KB per operation
- Cache efficiency: >90% hit rate for common operations
- Total memory: <50% of current dual system

### Async Performance
- No blocking operations in hot paths
- Minimal async overhead (<10ns)
- Efficient task scheduling
- Lock-free reads where possible

## Testing Strategy

### Unit Tests
- Individual operation implementations
- Sync/async evaluation paths
- Error handling and edge cases
- Performance benchmarks

### Integration Tests
- Evaluator integration
- Registry migration from old systems
- Compatibility with existing FHIRPath expressions
- Performance regression tests

### Performance Tests
- Operation dispatch benchmarks
- Memory allocation profiling
- Cache hit rate measurements
- Async overhead analysis

## Success Criteria

1. **Functional**: All migrated operations maintain exact same behavior
2. **Performance**: No regression in evaluation performance  
3. **Memory**: <20% memory usage increase during migration
4. **Compatibility**: Existing code continues to work with adapters
5. **Test Coverage**: >95% coverage for all migrated operations
6. **Async Support**: All operations support async evaluation

## Risks and Mitigation

### Performance Risk
- **Risk**: Unified system slower than specialized implementations
- **Mitigation**: Extensive benchmarking, sync fast paths, caching

### Behavioral Risk  
- **Risk**: Subtle differences in operation behavior
- **Mitigation**: Comprehensive test suites, behavioral validation

### Integration Risk
- **Risk**: Breaking changes to evaluator engine
- **Mitigation**: Adapter patterns, gradual migration, parallel systems

## Files to Create/Modify

### New Files
1. `crates/fhirpath-registry/src/operations/arithmetic.rs`
2. `crates/fhirpath-registry/src/operations/collection.rs`  
3. `crates/fhirpath-registry/src/operations/string.rs`
4. `crates/fhirpath-registry/src/operations/comparison.rs`
5. `crates/fhirpath-registry/src/operations/mod.rs`
6. `crates/fhirpath-registry/src/standard_registry.rs`

### Modified Files
1. `crates/fhirpath-evaluator/src/engine.rs`
2. `crates/fhirpath-registry/src/lib.rs`
3. `crates/octofhir-fhirpath/src/lib.rs`

## Dependencies Added

- High-performance string operations
- Async-aware numeric computations
- Advanced caching mechanisms

## Next Task Dependencies  

This task enables:
- Task 3: Complete Migration (remaining operations)
- Task 4: Cleanup and Optimization
- Performance validation and tuning