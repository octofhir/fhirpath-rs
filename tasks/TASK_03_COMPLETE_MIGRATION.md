# Task 3: Complete Migration

**Status:** Not Started  
**Estimated Time:** 2-3 weeks  
**Priority:** High  
**Dependencies:** Task 2 (Core Migration)

## Objective

Complete the migration of all remaining functions and operators to the unified registry system, remove legacy registry systems, and update all external usage points to use the new unified API.

## Deliverables

### 1. Advanced Function Categories Migration

```rust
// New file: crates/fhirpath-registry/src/operations/advanced.rs

/// Advanced operations including FHIR-specific and utility functions
pub struct AdvancedOperations;

impl AdvancedOperations {
    pub async fn register_all(registry: &mut FhirPathRegistry) {
        // FHIR-specific operations
        FhirOperations::register_all(registry).await;
        
        // Mathematical functions
        MathOperations::register_all(registry).await;
        
        // Date/time operations  
        DateTimeOperations::register_all(registry).await;
        
        // Utility functions
        UtilityOperations::register_all(registry).await;
        
        // Lambda-supporting functions
        LambdaOperations::register_all(registry).await;
    }
}

/// FHIR-specific operations
pub struct FhirOperations;

impl FhirOperations {
    pub async fn register_all(registry: &mut FhirPathRegistry) {
        registry.register(ResolveFunction::new()).await;
        registry.register(ExtensionFunction::new()).await;
        registry.register(ConformsToFunction::new()).await;
        registry.register(ComparableFunction::new()).await;
    }
}

/// Resolve function - requires ModelProvider, must be async
pub struct ResolveFunction;

#[async_trait]
impl FhirPathOperation for ResolveFunction {
    fn identifier(&self) -> &str { "resolve" }
    
    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        // Async-only operation due to ModelProvider requirement
        let model_provider = context.model_provider
            .as_ref()
            .ok_or(EvaluationError::MissingModelProvider)?;
            
        // Reference resolution logic using async ModelProvider
        match &context.input {
            FhirPathValue::String(reference) => {
                model_provider.resolve_reference(reference).await
                    .map_err(|e| EvaluationError::ResolutionError(e))
            }
            _ => Ok(FhirPathValue::Empty)
        }
    }
    
    // No sync support - requires async ModelProvider calls
    fn supports_sync(&self) -> bool { false }
}
```

### 2. Lambda-Supporting Functions Migration

```rust
// New file: crates/fhirpath-registry/src/operations/lambda.rs

/// Lambda-supporting operations (where, select, aggregate, etc.)
pub struct LambdaOperations;

impl LambdaOperations {
    pub async fn register_all(registry: &mut FhirPathRegistry) {
        registry.register(WhereFunction::new()).await;
        registry.register(SelectFunction::new()).await;
        registry.register(AggregateFunction::new()).await;
        registry.register(AllFunction::new()).await;
        registry.register(AnyFunction::new()).await;
    }
}

/// Where function - requires lambda evaluation
pub struct WhereFunction;

#[async_trait]
impl FhirPathOperation for WhereFunction {
    fn identifier(&self) -> &str { "where" }
    
    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }
    
    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::OnceLock<OperationMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| {
            OperationMetadata {
                basic: BasicOperationInfo {
                    name: "where".to_string(),
                    description: "Filters collection based on boolean expression".to_string(),
                    category: OperationCategory::Collection,
                    pure: true,
                },
                types: TypeConstraints {
                    input_types: vec![TypePattern::Collection(Box::new(TypePattern::Any))],
                    output_type: TypePattern::Collection(Box::new(TypePattern::Any)),
                    min_args: 1,
                    max_args: Some(1),
                },
                performance: PerformanceMetadata {
                    complexity: PerformanceComplexity::Linear,
                    supports_sync: false, // Requires async lambda evaluation
                    cacheable: false, // Context-dependent
                },
                lsp: LspMetadata {
                    completion_priority: 2,
                    snippet: "where($this)".to_string(),
                    examples: vec!["Patient.telecom.where(system='email')".to_string()],
                },
                specific: OperationSpecificMetadata::Function(FunctionMetadata {
                    supports_lambda: true,
                    lambda_args: vec![0], // First argument is lambda
                })
            }
        })
    }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        if args.len() != 1 {
            return Err(EvaluationError::InvalidArgumentCount { 
                expected: "1", 
                actual: args.len() 
            });
        }
        
        // Extract lambda expression
        let lambda_expr = match &args[0] {
            FhirPathValue::LambdaExpression(expr) => expr,
            _ => return Err(EvaluationError::InvalidArgumentType { 
                expected: "LambdaExpression", 
                actual: args[0].type_name() 
            })
        };
        
        // Filter collection using async lambda evaluation
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut filtered = Vec::new();
                
                for item in items.iter() {
                    let item_context = context.with_input(item.clone());
                    let result = context.lambda_evaluator
                        .as_ref()
                        .ok_or(EvaluationError::MissingLambdaEvaluator)?
                        .evaluate(lambda_expr, &item_context).await?;
                    
                    if result.is_truthy() {
                        filtered.push(item.clone());
                    }
                }
                
                Ok(FhirPathValue::collection(filtered))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            single_item => {
                // Evaluate lambda on single item
                let result = context.lambda_evaluator
                    .as_ref()
                    .ok_or(EvaluationError::MissingLambdaEvaluator)?
                    .evaluate(lambda_expr, context).await?;
                
                if result.is_truthy() {
                    Ok(single_item.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }
    
    // Lambda functions are async-only
    fn supports_sync(&self) -> bool { false }
}
```

### 3. Legacy System Removal

```rust
// Modified file: crates/fhirpath-registry/src/lib.rs

// Remove legacy exports
// pub use function::FunctionRegistry; // REMOVED
// pub use unified_registry::UnifiedFunctionRegistry; // REMOVED  
// pub use unified_operator_registry::UnifiedOperatorRegistry; // REMOVED

// New unified exports
pub use unified_registry_v2::FhirPathRegistry;
pub use unified_operation::FhirPathOperation;
pub use unified_metadata::{OperationMetadata, OperationType};
pub use standard_registry::{create_standard_registry, StandardRegistryBuilder};

// Migration compatibility (deprecated)
#[deprecated(since = "0.5.0", note = "Use FhirPathRegistry instead")]
pub type UnifiedFunctionRegistry = FhirPathRegistry;

#[deprecated(since = "0.5.0", note = "Use FhirPathRegistry instead")]
pub type UnifiedOperatorRegistry = FhirPathRegistry;

/// Create standard registries - now returns single unified registry
#[deprecated(since = "0.5.0", note = "Use create_standard_registry() instead")]
pub async fn create_standard_registries() -> (FhirPathRegistry, FhirPathRegistry) {
    let registry = create_standard_registry().await;
    (registry.clone(), registry)
}
```

### 4. External Usage Updates

```rust
// Modified file: crates/octofhir-fhirpath/src/lib.rs

/// FHIRPath engine with unified registry
pub struct FhirPathEngine {
    registry: FhirPathRegistry,
    evaluator: FhirPathEvaluator,
    model_provider: Option<Arc<dyn ModelProvider>>,
}

impl FhirPathEngine {
    /// Create engine with default configuration
    pub async fn new() -> Result<Self, FhirPathError> {
        let registry = create_standard_registry().await;
        let evaluator = FhirPathEvaluator::new_unified(registry.clone()).await?;
        
        Ok(Self {
            registry,
            evaluator,
            model_provider: None,
        })
    }
    
    /// Create engine with custom model provider
    pub async fn with_model_provider(
        model_provider: Arc<dyn ModelProvider>
    ) -> Result<Self, FhirPathError> {
        let mut engine = Self::new().await?;
        engine.model_provider = Some(model_provider);
        Ok(engine)
    }
    
    /// Evaluate FHIRPath expression (async)
    pub async fn evaluate(
        &self,
        expression: &str,
        context: FhirPathValue,
    ) -> Result<FhirPathValue, FhirPathError> {
        let evaluation_context = EvaluationContext {
            input: context.clone(),
            root: context,
            variables: FxHashMap::default(),
            model_provider: self.model_provider.clone(),
            lambda_evaluator: Some(&self.evaluator),
        };
        
        self.evaluator.evaluate(expression, &evaluation_context).await
    }
    
    /// Try synchronous evaluation (for performance)
    pub fn try_evaluate_sync(
        &self,
        expression: &str,
        context: FhirPathValue,
    ) -> Option<Result<FhirPathValue, FhirPathError>> {
        // Try sync path for simple expressions
        if let Some(simple_result) = self.evaluator.try_simple_sync(expression, &context) {
            Some(simple_result)
        } else {
            None // Fallback to async
        }
    }
}
```

### 5. Migration Validation and Testing

```rust
// New file: crates/fhirpath-registry/src/migration_validation.rs

/// Validation utilities to ensure migration correctness
pub struct MigrationValidator;

impl MigrationValidator {
    /// Validate that unified registry provides same results as legacy registries
    pub async fn validate_migration(
        legacy_func_registry: &function::FunctionRegistry,
        legacy_op_registry: &unified_operator_registry::UnifiedOperatorRegistry,
        unified_registry: &FhirPathRegistry,
    ) -> ValidationResult {
        let mut results = ValidationResult::new();
        
        // Test all functions
        for func_name in legacy_func_registry.function_names() {
            if let Err(e) = self.validate_function(&func_name, 
                legacy_func_registry, unified_registry).await {
                results.add_error(ValidationError::FunctionMismatch {
                    name: func_name,
                    error: e,
                });
            }
        }
        
        // Test all operators
        for op_symbol in legacy_op_registry.binary_operator_symbols() {
            if let Err(e) = self.validate_operator(&op_symbol,
                legacy_op_registry, unified_registry).await {
                results.add_error(ValidationError::OperatorMismatch {
                    symbol: op_symbol.to_string(),
                    error: e,
                });
            }
        }
        
        results
    }
    
    /// Validate specific function behavior
    async fn validate_function(
        &self,
        func_name: &str,
        legacy_registry: &function::FunctionRegistry,
        unified_registry: &FhirPathRegistry,
    ) -> Result<(), ValidationError> {
        // Test with various input combinations
        let test_cases = self.generate_test_cases_for_function(func_name);
        
        for test_case in test_cases {
            let legacy_result = legacy_registry
                .evaluate_function(func_name, &test_case.args, &test_case.context)
                .await;
            let unified_result = unified_registry
                .evaluate(func_name, &test_case.args, &test_case.context)
                .await;
                
            if !self.results_equivalent(&legacy_result, &unified_result) {
                return Err(ValidationError::ResultMismatch {
                    test_case,
                    legacy_result,
                    unified_result,
                });
            }
        }
        
        Ok(())
    }
}
```

## Implementation Steps

### Step 1: Complete Function Migration
- [ ] Migrate all remaining mathematical functions
- [ ] Migrate all date/time functions  
- [ ] Migrate all utility functions (iif, trace, defineVariable, etc.)
- [ ] Migrate all FHIR-specific functions

### Step 2: Complete Operator Migration  
- [ ] Migrate remaining logical operators
- [ ] Migrate type checking operators (is, as)
- [ ] Migrate string concatenation operator
- [ ] Migrate collection operators (union, in, contains)

### Step 3: Lambda Function Support
- [ ] Implement lambda-supporting functions (where, select, aggregate)
- [ ] Add lambda expression evaluation support
- [ ] Integrate with evaluator engine
- [ ] Test complex lambda scenarios

### Step 4: Legacy System Removal
- [ ] Remove old `FunctionRegistry` code
- [ ] Remove old `UnifiedOperatorRegistry` code
- [ ] Remove legacy traits and implementations
- [ ] Clean up unused modules and files

### Step 5: External API Updates
- [ ] Update `octofhir-fhirpath` crate to use unified registry
- [ ] Update CLI tools and utilities  
- [ ] Update benchmarking infrastructure
- [ ] Update integration test runner

### Step 6: Migration Validation
- [ ] Comprehensive behavioral validation  
- [ ] Performance regression testing
- [ ] Integration test validation
- [ ] Documentation and migration guide updates

## Performance Targets

### Migration Performance
- No regression in function/operator evaluation performance
- <10% overhead during transition period with compatibility layers
- Registry creation time <20ms for full standard registry
- Memory usage increase <30% during migration

### Final Performance Goals
- 20%+ improvement in evaluation throughput after cleanup
- 50%+ reduction in memory footprint vs dual registry system
- <100ns operation dispatch including cache lookup
- Async overhead <5ns for async-capable operations

## Testing Strategy

### Migration Validation
- Behavioral equivalence testing for all functions/operators
- Performance regression testing  
- Integration test validation
- Edge case and error condition testing

### Compatibility Testing
- Legacy API compatibility via adapters
- Existing codebase integration testing
- Third-party usage validation
- Migration script validation

### Performance Testing
- Throughput benchmarks before/after migration
- Memory allocation profiling
- Cache effectiveness measurement  
- Async overhead analysis

## Success Criteria

1. **Functional Equivalence**: 100% behavioral compatibility with legacy systems
2. **Performance**: No regression, target 20%+ improvement post-cleanup  
3. **Memory Efficiency**: 50%+ reduction in registry memory footprint
4. **API Simplicity**: Single registry for all operations
5. **Test Coverage**: >95% coverage maintained
6. **Migration Success**: All external usage successfully migrated

## Risks and Mitigation

### Compatibility Risk
- **Risk**: Breaking changes to existing APIs
- **Mitigation**: Compatibility adapters, gradual deprecation, migration tools

### Performance Risk  
- **Risk**: Regression during transition period
- **Mitigation**: Comprehensive benchmarking, performance monitoring

### Complexity Risk
- **Risk**: Lambda functions and async operations more complex than expected  
- **Mitigation**: Incremental implementation, extensive testing

## Files to Create/Modify

### New Files
1. `crates/fhirpath-registry/src/operations/advanced.rs`
2. `crates/fhirpath-registry/src/operations/lambda.rs`  
3. `crates/fhirpath-registry/src/operations/math.rs`
4. `crates/fhirpath-registry/src/operations/datetime.rs`
5. `crates/fhirpath-registry/src/operations/utility.rs`
6. `crates/fhirpath-registry/src/migration_validation.rs`

### Files to Remove
1. `crates/fhirpath-registry/src/function.rs` (legacy)
2. `crates/fhirpath-registry/src/unified_registry.rs` (legacy)
3. `crates/fhirpath-registry/src/unified_operator_registry.rs` (legacy)
4. All old function implementation files in `unified_implementations/`
5. All old operator implementation files in `unified_operators/`

### Modified Files  
1. `crates/fhirpath-registry/src/lib.rs` - Updated exports
2. `crates/octofhir-fhirpath/src/lib.rs` - Use unified registry
3. `crates/fhirpath-evaluator/src/engine.rs` - Remove legacy registry support
4. `crates/fhirpath-tools/src/lib.rs` - Update tooling

## Migration Timeline

### Week 1: Advanced Functions
- Mathematical, date/time, utility functions migration
- FHIR-specific functions migration  
- Performance validation

### Week 2: Lambda Support & Remaining Operators
- Lambda-supporting functions implementation
- Remaining operator migration
- Complex evaluation scenarios

### Week 3: Legacy Removal & Validation
- Remove legacy registry systems
- Update external API usage
- Comprehensive migration validation

## Next Task Dependencies

This task enables:
- Task 4: Cleanup and Optimization  
- Performance optimization phase
- Final documentation updates
- Release preparation