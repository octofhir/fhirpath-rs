# FHIRPath Registry Simplification Tasks

This directory contains the detailed task breakdown for simplifying and unifying the fhirpath-registry crate. The goal is to combine the separate function and operator registries into a single, high-performance, async-first system.

## Overview

The current fhirpath-registry has evolved into a complex dual registry system with significant code duplication, performance overhead, and API complexity. This task series will create a unified registry that provides:

- **Simplicity**: Single registry API instead of two separate systems
- **Performance**: 20%+ throughput improvement, 50%+ memory reduction
- **Async-First**: Non-blocking operations with optimal async/await patterns
- **Maintainability**: Cleaner codebase with reduced complexity

## Task Sequence

### [Task 1: Foundation](TASK_01_FOUNDATION.md)
**Status:** Not Started | **Time:** 2-3 weeks | **Priority:** Critical

Create the core unified registry architecture with:
- Unified `FhirPathRegistry` struct
- `FhirPathOperation` trait for all operations
- High-performance async caching system
- Migration utilities and compatibility layer

**Key Deliverables:**
- Core unified registry implementation
- Async-optimized dispatch and caching
- Migration infrastructure
- Performance benchmarking framework

### [Task 2: Core Migration](TASK_02_CORE_MIGRATION.md)  
**Status:** Not Started | **Time:** 3-4 weeks | **Priority:** Critical

Migrate the most critical functions and operators:
- Core arithmetic operators (+, -, *, /, %)
- Essential collection functions (count, exists, first, last)  
- Core string functions (length, contains, startsWith)
- Evaluator engine integration

**Key Deliverables:**
- High-frequency operations migrated
- Evaluator integration complete
- Standard registry builder
- Performance validation

### [Task 3: Complete Migration](TASK_03_COMPLETE_MIGRATION.md)
**Status:** Not Started | **Time:** 2-3 weeks | **Priority:** High

Complete migration of all remaining operations:
- Advanced functions (FHIR-specific, mathematical, utility)
- Lambda-supporting functions (where, select, aggregate)  
- All remaining operators
- Legacy system removal

**Key Deliverables:**
- All operations migrated to unified system
- Lambda expression support
- Legacy code removal
- Complete API migration

### [Task 4: Cleanup and Optimization](TASK_04_CLEANUP_OPTIMIZATION.md)
**Status:** Not Started | **Time:** 1-2 weeks | **Priority:** Medium  

Final optimization and cleanup:
- Performance optimization based on benchmarks
- Memory usage optimization
- API cleanup and documentation
- Final testing and validation

**Key Deliverables:**
- Optimized performance implementation  
- Comprehensive documentation
- Migration guide
- Release preparation

## Success Metrics

### Performance Targets
- **Dispatch Performance**: <100ns per operation (cached)
- **Memory Usage**: 50%+ reduction vs current dual system
- **Throughput**: 20%+ improvement in evaluation performance
- **Registry Creation**: <20ms for full standard registry

### API Simplicity
- **Single Registry**: One `FhirPathRegistry` instead of two separate systems
- **Unified API**: Same interface for functions and operators
- **Async-First**: All operations support async evaluation
- **Clean Migration**: Clear migration path from legacy system

### Code Quality  
- **Reduced Complexity**: <50% lines of code in registry module
- **Zero Warnings**: Clean compilation with no warnings
- **Test Coverage**: Maintain >95% coverage throughout migration
- **Documentation**: Complete API docs and examples

## Breaking Changes

This migration involves breaking changes to achieve the simplification goals:

### Registry Creation
```rust
// OLD: Dual registry system
let (func_registry, op_registry) = create_standard_registries();

// NEW: Single unified registry
let registry = FhirPathRegistry::new_standard().await;
```

### Operation Evaluation
```rust
// OLD: Separate evaluation methods
func_registry.evaluate_function("count", &[], &context).await?;
op_registry.evaluate_binary("+", left, right, &context).await?;

// NEW: Unified evaluation
registry.evaluate("count", &[], &context).await?;
registry.evaluate("+", &[left, right], &context).await?;
```

## Migration Support

### Compatibility Layer
During the transition, compatibility adapters will be provided:
- Legacy API wrappers for gradual migration
- Migration validation tools
- Performance comparison utilities
- Automated migration scripts where possible

### Documentation
Comprehensive migration documentation including:
- Step-by-step migration guide  
- API change reference
- Performance optimization guide
- Troubleshooting common issues

## Timeline

**Total Estimated Time:** 8-12 weeks

- **Phase 1** (Tasks 1-2): 5-7 weeks - Foundation and core migration
- **Phase 2** (Task 3): 2-3 weeks - Complete migration and legacy removal  
- **Phase 3** (Task 4): 1-2 weeks - Optimization and finalization

## Dependencies

### External Dependencies
- `tokio` - Async runtime support
- `async-trait` - Async trait definitions
- `lru` - High-performance caching
- `rustc-hash` - Fast hashing for cache keys

### Internal Dependencies
- All tasks depend on the previous task completion
- Evaluator integration requires Task 1 foundation
- Performance optimization requires benchmarking from Task 2

## Risk Mitigation

### Performance Risk
- **Risk**: Unified system could be slower than specialized registries
- **Mitigation**: Extensive benchmarking, specialized fast paths, performance-first design

### Migration Risk  
- **Risk**: Difficult migration from existing dual registry system
- **Mitigation**: Comprehensive migration tools, parallel system support, clear documentation

### Complexity Risk
- **Risk**: Unified system becomes more complex than individual systems
- **Mitigation**: Simple core design, extensive testing, iterative refinement

## Getting Started

1. **Review Plan**: Read the [main plan document](../FHIRPATH_REGISTRY_SIMPLIFICATION_PLAN.md)
2. **Start with Task 1**: Begin with foundation implementation
3. **Follow Dependencies**: Complete tasks in sequence due to dependencies
4. **Monitor Progress**: Track performance and compatibility throughout

## Questions and Support

For questions about this migration plan:
1. Review the detailed task documents
2. Check the main simplification plan
3. Consult the existing registry implementation for context
4. Consider performance implications of proposed changes

This migration will significantly improve the FHIRPath registry system's maintainability, performance, and usability while providing a clean async-first API for modern Rust applications.