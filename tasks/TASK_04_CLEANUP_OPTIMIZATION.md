# Task 4: Cleanup and Optimization

**Status:** Not Started  
**Estimated Time:** 1-2 weeks  
**Priority:** Medium  
**Dependencies:** Task 3 (Complete Migration)

## Objective

Finalize the unified registry system by removing all legacy code, optimizing performance based on benchmarks, cleaning up the API surface, and preparing comprehensive documentation.

## Deliverables

### 1. Legacy Code Removal and Cleanup

```rust
// Final cleanup of crates/fhirpath-registry/src/lib.rs

//! Unified FHIRPath Registry
//!
//! This crate provides a high-performance, async-first unified registry for all
//! FHIRPath functions and operators. The registry combines previously separate
//! function and operator registries into a single, optimized system.

// Core unified system
pub use unified_registry_v2::FhirPathRegistry;
pub use unified_operation::{FhirPathOperation, OperationType};
pub use unified_metadata::{OperationMetadata, OperationCategory};
pub use standard_registry::{create_standard_registry, StandardRegistryBuilder};

// Operation implementations  
pub mod operations;

// Utilities
pub use async_cache::AsyncLruCache;
pub use migration_utils::{RegistryMigrationHelper, MigrationStats};

// Re-exports from workspace crates
pub use octofhir_fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator};
pub use octofhir_fhirpath_core::{FhirPathError, Result};
pub use octofhir_fhirpath_model::{FhirPathValue, ModelProvider};

// Performance and metrics
pub use registry_metrics::{RegistryMetrics, PerformanceStats};

// Error types
pub use errors::{EvaluationError, RegistryError, MigrationError};

// REMOVED: All legacy exports and compatibility layers
// REMOVED: function.rs, unified_registry.rs, unified_operator_registry.rs
// REMOVED: unified_implementations/, unified_operators/, functions/, operators/
```

### 2. Performance Optimization

```rust
// Enhanced file: crates/fhirpath-registry/src/performance_optimizer.rs

/// Performance optimization utilities for the unified registry
pub struct PerformanceOptimizer {
    metrics: Arc<RegistryMetrics>,
    config: OptimizationConfig,
}

impl PerformanceOptimizer {
    /// Optimize registry based on usage patterns
    pub async fn optimize_registry(
        &self,
        registry: &mut FhirPathRegistry,
        usage_stats: &UsageStatistics,
    ) -> OptimizationResult {
        let mut results = OptimizationResult::new();
        
        // Optimize dispatch cache based on usage patterns
        self.optimize_dispatch_cache(registry, usage_stats).await?;
        
        // Precompile frequently used operations
        self.precompile_hot_operations(registry, usage_stats).await?;
        
        // Optimize metadata storage
        self.optimize_metadata_storage(registry).await?;
        
        // Configure async pools for optimal performance
        self.configure_async_pools(registry).await?;
        
        results.add_improvement("dispatch_cache", "Optimized for top 95% operations");
        results.add_improvement("precompilation", "Hot path operations precompiled");
        results.add_improvement("metadata", "Reduced metadata memory footprint by 40%");
        results.add_improvement("async_pools", "Configured optimal async task pools");
        
        results
    }
    
    /// Optimize dispatch cache for hot operations
    async fn optimize_dispatch_cache(
        &self,
        registry: &mut FhirPathRegistry,
        usage_stats: &UsageStatistics,
    ) -> Result<(), OptimizationError> {
        // Identify most frequently used operations
        let hot_operations = usage_stats.get_top_operations(0.95);
        
        // Pre-warm cache with hot operations
        for operation in hot_operations {
            registry.warm_cache(&operation.name, &operation.common_arg_types).await;
        }
        
        // Configure cache size based on operation count and memory constraints
        let optimal_size = self.calculate_optimal_cache_size(usage_stats);
        registry.resize_cache(optimal_size).await;
        
        Ok(())
    }
    
    /// Precompile hot path operations for maximum performance
    async fn precompile_hot_operations(
        &self,
        registry: &mut FhirPathRegistry,
        usage_stats: &UsageStatistics,
    ) -> Result<(), OptimizationError> {
        let critical_operations = usage_stats.get_critical_path_operations();
        
        for operation in critical_operations {
            // Create specialized fast-path implementations
            if let Some(optimized) = self.create_optimized_implementation(&operation).await {
                registry.register_optimized_implementation(
                    &operation.name, 
                    optimized
                ).await;
            }
        }
        
        Ok(())
    }
    
    /// Create specialized optimized implementations for critical operations
    async fn create_optimized_implementation(
        &self,
        operation: &OperationUsageInfo,
    ) -> Option<Arc<dyn FhirPathOperation>> {
        match operation.name.as_str() {
            "count" => Some(Arc::new(OptimizedCountOperation::new())),
            "+" if operation.is_mostly_numeric() => Some(Arc::new(OptimizedNumericAddition::new())),
            "length" => Some(Arc::new(OptimizedLengthOperation::new())),
            "empty" => Some(Arc::new(OptimizedEmptyOperation::new())),
            _ => None
        }
    }
}

/// Specialized optimized count operation
pub struct OptimizedCountOperation;

#[async_trait]
impl FhirPathOperation for OptimizedCountOperation {
    fn identifier(&self) -> &str { "count" }
    
    fn operation_type(&self) -> OperationType { OperationType::Function }
    
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, EvaluationError> {
        // Ultra-fast count implementation with minimal allocations
        match &context.input {
            FhirPathValue::Collection(items) => Ok(FhirPathValue::Integer(items.len() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        }
    }
    
    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue, EvaluationError>> {
        // Always sync for count
        Some(match &context.input {
            FhirPathValue::Collection(items) => Ok(FhirPathValue::Integer(items.len() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        })
    }
    
    fn supports_sync(&self) -> bool { true }
}
```

### 3. Memory Usage Optimization

```rust
// New file: crates/fhirpath-registry/src/memory_optimizer.rs

/// Memory usage optimization for registry
pub struct MemoryOptimizer;

impl MemoryOptimizer {
    /// Optimize memory usage across the registry
    pub async fn optimize_memory_usage(
        registry: &mut FhirPathRegistry,
    ) -> MemoryOptimizationResult {
        let initial_usage = self.measure_memory_usage(registry).await;
        
        // Optimize metadata storage
        self.optimize_metadata_storage(registry).await;
        
        // Implement copy-on-write for operation metadata  
        self.implement_cow_metadata(registry).await;
        
        // Optimize string interning
        self.optimize_string_interning(registry).await;
        
        // Compact cache structures
        self.compact_cache_structures(registry).await;
        
        let final_usage = self.measure_memory_usage(registry).await;
        
        MemoryOptimizationResult {
            initial_usage,
            final_usage,
            savings: initial_usage - final_usage,
            optimizations_applied: vec![
                "metadata_deduplication",
                "cow_metadata", 
                "string_interning",
                "cache_compaction"
            ],
        }
    }
    
    /// Implement copy-on-write for metadata to reduce duplication
    async fn implement_cow_metadata(&self, registry: &mut FhirPathRegistry) {
        // Use Arc<T> and Cow<'static, T> for metadata sharing
        // Deduplicate common metadata patterns
        // Intern frequently used strings
    }
    
    /// Optimize string storage using interning
    async fn optimize_string_interning(&self, registry: &mut FhirPathRegistry) {
        // Create global string interner for operation names, descriptions
        // Share common strings across metadata
        // Use static string references where possible
    }
}
```

### 4. API Documentation and Examples

```rust
// Enhanced file: crates/fhirpath-registry/src/lib.rs documentation

//! # FHIRPath Registry
//!
//! A high-performance, async-first unified registry for FHIRPath functions and operators.
//!
//! ## Features
//!
//! - **Unified System**: Single registry for all FHIRPath operations
//! - **Async-First**: Non-blocking evaluation with optimal async/await patterns  
//! - **High Performance**: <100ns operation dispatch, extensive caching
//! - **Memory Efficient**: 50%+ reduction vs previous dual registry system
//! - **Extensible**: Clean API for custom function/operator registration
//!
//! ## Quick Start
//!
//! ```rust
//! use fhirpath_registry::{FhirPathRegistry, create_standard_registry};
//! use fhirpath_model::{FhirPathValue, EvaluationContext};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create standard registry with all built-in operations
//!     let registry = create_standard_registry().await;
//!     
//!     // Create evaluation context
//!     let context = EvaluationContext::new(
//!         FhirPathValue::collection(vec![
//!             FhirPathValue::String("hello".into()),
//!             FhirPathValue::String("world".into()),
//!         ])
//!     );
//!     
//!     // Evaluate function
//!     let result = registry.evaluate("count", &[], &context).await?;
//!     assert_eq!(result, FhirPathValue::Integer(2));
//!     
//!     // Try synchronous path for performance  
//!     if let Some(sync_result) = registry.try_evaluate_sync("count", &[], &context) {
//!         let result = sync_result?;
//!         assert_eq!(result, FhirPathValue::Integer(2));
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Custom Operations
//!
//! ```rust
//! use async_trait::async_trait;
//! use fhirpath_registry::{FhirPathOperation, OperationType, OperationMetadata};
//!
//! struct CustomUpperFunction;
//!
//! #[async_trait]
//! impl FhirPathOperation for CustomUpperFunction {
//!     fn identifier(&self) -> &str { "customUpper" }
//!     
//!     fn operation_type(&self) -> OperationType {
//!         OperationType::Function
//!     }
//!     
//!     async fn evaluate(
//!         &self,
//!         args: &[FhirPathValue],
//!         context: &EvaluationContext,
//!     ) -> Result<FhirPathValue, EvaluationError> {
//!         match &context.input {
//!             FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase().into())),
//!             _ => Ok(FhirPathValue::Empty)
//!         }
//!     }
//! }
//!
//! // Register custom function
//! let mut registry = FhirPathRegistry::new();
//! registry.register(CustomUpperFunction).await;
//! ```
//!
//! ## Performance
//!
//! The unified registry is optimized for performance:
//!
//! - **Fast Dispatch**: <50ns for cached operations  
//! - **Sync Fast Paths**: Synchronous evaluation for simple operations
//! - **Memory Efficient**: Shared metadata, optimized caching
//! - **Async Optimized**: Non-blocking evaluation with minimal overhead
//!
//! ## Architecture
//!
//! The registry uses a unified approach where both functions (`count`, `length`) 
//! and operators (`+`, `=`) implement the same `FhirPathOperation` trait. This 
//! simplifies the API while maintaining high performance through specialized
//! optimizations.
```

### 5. Comprehensive Benchmarking

```rust
// Enhanced file: crates/fhirpath-benchmarks/src/unified_registry_benchmarks.rs

/// Comprehensive benchmarks for unified registry performance
pub struct UnifiedRegistryBenchmarks;

impl UnifiedRegistryBenchmarks {
    /// Benchmark operation dispatch performance
    pub fn benchmark_dispatch_performance(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let registry = rt.block_on(create_standard_registry());
        
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2), 
            FhirPathValue::Integer(3),
        ]));
        
        // Benchmark most common operations
        c.bench_function("unified_count_dispatch", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(registry.evaluate("count", &[], &context).await.unwrap())
            })
        });
        
        c.bench_function("unified_count_sync", |b| {
            b.iter(|| {
                black_box(registry.try_evaluate_sync("count", &[], &context).unwrap().unwrap())
            })
        });
        
        // Benchmark arithmetic operations
        let args = vec![FhirPathValue::Integer(42), FhirPathValue::Integer(8)];
        c.bench_function("unified_addition", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(registry.evaluate("+", &args, &context).await.unwrap())
            })
        });
        
        c.bench_function("unified_addition_sync", |b| {
            b.iter(|| {
                black_box(registry.try_evaluate_sync("+", &args, &context).unwrap().unwrap())
            })
        });
    }
    
    /// Benchmark memory usage and allocation patterns
    pub fn benchmark_memory_usage(c: &mut Criterion) {
        c.bench_function("registry_creation", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                black_box(create_standard_registry().await)
            })
        });
        
        c.bench_function("operation_registration", |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.to_async(&rt).iter(|| async {
                let mut registry = FhirPathRegistry::new();
                black_box(registry.register(TestOperation::new()).await);
                registry
            })
        });
    }
    
    /// Benchmark vs legacy system performance
    pub fn benchmark_vs_legacy(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        // Legacy dual registry system
        let (legacy_func_registry, legacy_op_registry) = rt.block_on(async {
            // This would use the old system for comparison
            todo!("Create legacy registries for comparison")
        });
        
        // New unified registry
        let unified_registry = rt.block_on(create_standard_registry());
        
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        
        // Compare function evaluation
        let mut group = c.benchmark_group("function_evaluation");
        
        group.bench_function("legacy_count", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    legacy_func_registry
                        .evaluate_function("count", &[], &context)
                        .await
                        .unwrap()
                )
            })
        });
        
        group.bench_function("unified_count", |b| {
            b.to_async(&rt).iter(|| async {
                black_box(unified_registry.evaluate("count", &[], &context).await.unwrap())
            })
        });
        
        group.finish();
    }
}
```

### 6. Final Documentation

```markdown
# FHIRPath Registry Migration Guide

## Overview

The FHIRPath registry has been simplified from a dual registry system (separate function and operator registries) to a unified, high-performance system. This guide covers the migration process and API changes.

## Breaking Changes

### Registry Creation

**Before (v0.4.x):**
```rust
let (func_registry, op_registry) = create_standard_registries();
```

**After (v0.5.0+):**
```rust
let registry = create_standard_registry().await;
```

### Function Evaluation

**Before:**
```rust
let result = func_registry.evaluate_function("count", &[], &context).await?;
```

**After:**
```rust
let result = registry.evaluate("count", &[], &context).await?;
```

### Operator Evaluation

**Before:**
```rust
let result = op_registry.evaluate_binary("+", left, right, &context).await?;
```

**After:**
```rust
let result = registry.evaluate("+", &[left, right], &context).await?;
```

## Performance Improvements

- 20%+ improvement in evaluation throughput
- 50%+ reduction in memory usage
- <100ns operation dispatch (cached)
- Zero-copy operation metadata

## Migration Assistance

For complex migrations, use the migration helper:

```rust
use fhirpath_registry::migration_utils::RegistryMigrationHelper;

// Validate migration
let validation = RegistryMigrationHelper::validate_migration(
    &legacy_func_registry,
    &legacy_op_registry, 
    &unified_registry
).await;

if !validation.is_successful() {
    eprintln!("Migration validation failed: {}", validation.summary());
}
```
```

## Implementation Steps

### Step 1: Remove Legacy Code
- [ ] Delete all legacy registry implementation files
- [ ] Remove legacy traits and interfaces
- [ ] Clean up unused dependencies
- [ ] Update module structure

### Step 2: Performance Optimization  
- [ ] Implement performance profiling and optimization
- [ ] Add specialized fast-path implementations
- [ ] Optimize memory usage and allocation patterns
- [ ] Configure optimal async execution

### Step 3: API Cleanup
- [ ] Remove deprecated APIs and compatibility layers
- [ ] Simplify public API surface  
- [ ] Update all documentation and examples
- [ ] Clean up error types and messages

### Step 4: Documentation and Examples
- [ ] Write comprehensive API documentation
- [ ] Create migration guide and examples
- [ ] Add performance benchmarking documentation
- [ ] Update README and getting started guides

### Step 5: Final Testing and Validation
- [ ] Run complete test suite validation
- [ ] Performance regression testing
- [ ] Memory usage validation
- [ ] Integration testing with dependent crates

### Step 6: Release Preparation
- [ ] Version number updates
- [ ] Changelog preparation  
- [ ] Release note drafting
- [ ] Final QA and review

## Success Criteria

1. **Code Quality**: Zero compiler warnings, clean API surface
2. **Performance**: Meet or exceed all performance targets
3. **Memory Usage**: Achieve 50%+ memory reduction
4. **Documentation**: Complete API documentation and examples
5. **Test Coverage**: Maintain >95% test coverage
6. **Migration**: Smooth migration path for existing users

## Performance Targets (Final)

### Operation Performance
- Registry creation: <20ms  
- Function dispatch: <50ns (cached)
- Operator dispatch: <25ns (cached)
- Sync evaluation overhead: <5ns

### Memory Efficiency
- Registry base memory: <5MB
- Operation metadata: <512B per operation
- Cache hit rate: >95% for common operations
- Total memory reduction: >50% vs legacy system

## Files to Remove

1. `crates/fhirpath-registry/src/function.rs`
2. `crates/fhirpath-registry/src/unified_registry.rs`  
3. `crates/fhirpath-registry/src/unified_operator_registry.rs`
4. Entire `crates/fhirpath-registry/src/unified_implementations/` directory
5. Entire `crates/fhirpath-registry/src/unified_operators/` directory
6. Legacy trait implementations and helper modules

## Files to Finalize

1. `crates/fhirpath-registry/src/lib.rs` - Final API
2. `crates/fhirpath-registry/src/unified_registry_v2.rs` - Rename to `unified_registry.rs`
3. `crates/fhirpath-registry/README.md` - Updated documentation
4. `MIGRATION_GUIDE.md` - Comprehensive migration guide
5. Performance benchmark documentation

## Risk Mitigation

### Performance Risk
- **Mitigation**: Comprehensive benchmarking before/after cleanup
- **Fallback**: Keep optimization tools for post-release tuning

### API Risk  
- **Mitigation**: Extensive API review and validation
- **Fallback**: Rapid patch release capability for critical issues

### Migration Risk
- **Mitigation**: Comprehensive migration guide and tooling
- **Fallback**: Extended support period for legacy APIs

## Timeline

### Week 1: Legacy Removal and Optimization
- Remove legacy code and dependencies
- Implement performance optimizations
- Memory usage optimization

### Week 2: Documentation and Final Testing  
- Complete API documentation
- Migration guide creation
- Final performance validation and testing

This task completes the unified registry migration, delivering a clean, high-performance, async-first FHIRPath registry system.