# ADR-003 Phase 2: Enhanced Type Function Implementation

**Related ADR:** [ADR-003: Enhanced FHIRPath Type Reflection System](../docs/adr/ADR-003-fhirpath-type-reflection-system.md)  
**Phase:** 2 of 5  
**Status:** Planned  
**Priority:** Critical  
**Estimated Effort:** 2-3 weeks  
**Prerequisites:** Phase 1 completed

## Objective

Implement a specification-compliant `type()` function that returns proper FHIRPath TypeInfo structures, enabling advanced type introspection and meta-programming capabilities while maintaining industry-leading performance.

## Scope

### In Scope
1. **Specification-Compliant Type Function**
   - Return proper SimpleTypeInfo, ClassInfo, ListTypeInfo, TupleTypeInfo
   - Full compliance with FHIRPath §11 specification
   - Support for all FhirPathValue variants

2. **Healthcare-Specific Type Resolution**
   - FHIR resource type detection and classification
   - Choice type handling (value[x] patterns)
   - Reference type resolution with target constraints
   - Backbone element support (anonymous types)

3. **Performance Optimization**
   - Zero-allocation type operations during steady state
   - Intelligent caching of computed type information
   - Lazy evaluation of complex type structures
   - SIMD-optimized type comparison operations

4. **Advanced Type Inference**
   - Collection element type inference from contents
   - Polymorphic type resolution for union types
   - Context-aware type resolution
   - Type compatibility scoring

### Out of Scope
- Full constraint validation (Phase 4)
- Multi-version FHIR support (Phase 3)
- Profile-specific type information (Phase 3)
- IDE integration features (Phase 4)

## Technical Implementation

### 1. Enhanced Type Function Core

**File:** `crates/fhirpath-registry/src/operations/types/type_func.rs`

```rust
use crate::{FhirPathOperation, EvaluationContext};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection, reflection::FhirPathTypeInfo};
use async_trait::async_trait;
use std::sync::Arc;

/// Specification-compliant type() function implementation
pub struct EnhancedTypeFunction {
    type_cache: Arc<TypeInformationCache>,
    interner: Arc<StringInterner>,
}

impl EnhancedTypeFunction {
    pub fn new() -> Self {
        Self {
            type_cache: Arc::new(TypeInformationCache::new()),
            interner: Arc::new(StringInterner::global()),
        }
    }
    
    /// Create specification-compliant type information for any FhirPathValue
    async fn create_type_info(
        &self,
        value: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathTypeInfo> {
        match value {
            // System primitive types - SimpleTypeInfo
            FhirPathValue::Boolean(_) => Ok(self.create_simple_type_info("System", "Boolean", Some("System.Any"))),
            FhirPathValue::Integer(_) => Ok(self.create_simple_type_info("System", "Integer", Some("System.Any"))),
            FhirPathValue::Decimal(_) => Ok(self.create_simple_type_info("System", "Decimal", Some("System.Any"))),
            FhirPathValue::String(_) => Ok(self.create_simple_type_info("System", "String", Some("System.Any"))),
            FhirPathValue::Date(_) => Ok(self.create_simple_type_info("System", "Date", Some("System.Any"))),
            FhirPathValue::DateTime(_) => Ok(self.create_simple_type_info("System", "DateTime", Some("System.Any"))),
            FhirPathValue::Time(_) => Ok(self.create_simple_type_info("System", "Time", Some("System.Any"))),
            FhirPathValue::Quantity(_) => Ok(self.create_simple_type_info("System", "Quantity", Some("System.Any"))),
            
            // FHIR resources and complex types - ClassInfo
            FhirPathValue::Resource(resource) => {
                self.create_resource_class_info(resource, context).await
            },
            FhirPathValue::JsonValue(json) => {
                self.create_json_class_info(json, context).await
            },
            
            // Collections - ListTypeInfo
            FhirPathValue::Collection(collection) => {
                self.create_list_type_info(collection, context).await
            },
            
            // Other complex types
            _ => Ok(self.create_simple_type_info("System", "Any", None)),
        }
    }
    
    fn create_simple_type_info(
        &self,
        namespace: &str,
        name: &str,
        base_type: Option<&str>,
    ) -> FhirPathTypeInfo {
        FhirPathTypeInfo::SimpleTypeInfo {
            namespace: self.interner.intern(namespace),
            name: self.interner.intern(name),
            base_type: base_type.map(|bt| self.interner.intern(bt)),
            constraints: None,
        }
    }
    
    async fn create_resource_class_info(
        &self,
        resource: &dyn ResourceReflection,
        context: &EvaluationContext,
    ) -> Result<FhirPathTypeInfo> {
        let resource_type = resource.resource_type()
            .unwrap_or("Resource");
        
        // Check cache first
        if let Some(cached) = self.type_cache.get_class_info(resource_type).await {
            return Ok(cached);
        }
        
        // Get type information from ModelProvider
        let model_provider = context.model_provider();
        let type_reflection = model_provider
            .get_type_reflection(resource_type)
            .await
            .ok_or_else(|| FhirPathError::type_error(
                format!("Unknown resource type: {}", resource_type)
            ))?;
        
        // Convert to FhirPathTypeInfo
        let class_info = self.convert_reflection_to_class_info(type_reflection, resource_type).await?;
        
        // Cache the result
        self.type_cache.cache_class_info(resource_type, class_info.clone()).await;
        
        Ok(class_info)
    }
    
    async fn create_list_type_info(
        &self,
        collection: &Collection,
        context: &EvaluationContext,
    ) -> Result<FhirPathTypeInfo> {
        if collection.is_empty() {
            // Empty collection - return List<Any>
            return Ok(FhirPathTypeInfo::ListTypeInfo {
                element_type: Box::new(self.create_simple_type_info("System", "Any", None)),
                cardinality: Cardinality { min: 0, max: None },
            });
        }
        
        // Infer element type from collection contents
        let element_type = self.infer_collection_element_type(collection, context).await?;
        
        Ok(FhirPathTypeInfo::ListTypeInfo {
            element_type: Box::new(element_type),
            cardinality: Cardinality {
                min: collection.len() as u32,
                max: None, // Collections are unbounded by default
            },
        })
    }
    
    async fn infer_collection_element_type(
        &self,
        collection: &Collection,
        context: &EvaluationContext,
    ) -> Result<FhirPathTypeInfo> {
        // Get type information for all elements
        let mut element_types = Vec::new();
        for item in collection.iter().take(10) { // Sample first 10 items for performance
            let item_type = self.create_type_info(item, context).await?;
            element_types.push(item_type);
        }
        
        // Find common type or create union
        self.compute_common_type(&element_types).await
    }
    
    async fn compute_common_type(
        &self,
        types: &[FhirPathTypeInfo],
    ) -> Result<FhirPathTypeInfo> {
        if types.is_empty() {
            return Ok(self.create_simple_type_info("System", "Any", None));
        }
        
        if types.len() == 1 {
            return Ok(types[0].clone());
        }
        
        // Check if all types are the same
        let first_type = &types[0];
        if types.iter().all(|t| t == first_type) {
            return Ok(first_type.clone());
        }
        
        // For now, return the first type
        // TODO: Implement proper type union logic
        Ok(first_type.clone())
    }
}

#[async_trait]
impl FhirPathOperation for EnhancedTypeFunction {
    fn identifier(&self) -> &str {
        "type"
    }
    
    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let input = &context.input;
        
        match input {
            FhirPathValue::Collection(collection) => {
                let mut result_types = Vec::new();
                
                for item in collection.iter() {
                    let type_info = self.create_type_info(item, context).await?;
                    result_types.push(FhirPathValue::TypeReflectionObject(type_info));
                }
                
                Ok(FhirPathValue::Collection(Collection::from_vec(result_types)))
            },
            _ => {
                let type_info = self.create_type_info(input, context).await?;
                Ok(FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::TypeReflectionObject(type_info)
                ])))
            }
        }
    }
}
```

### 2. Type Information Cache

**File:** `crates/fhirpath-model/src/reflection/cache.rs`

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;
use std::num::NonZeroUsize;

/// High-performance cache for type information
pub struct TypeInformationCache {
    class_info_cache: Arc<RwLock<LruCache<String, FhirPathTypeInfo>>>,
    simple_type_cache: Arc<RwLock<LruCache<String, FhirPathTypeInfo>>>,
    inference_cache: Arc<RwLock<LruCache<String, FhirPathTypeInfo>>>,
    cache_stats: Arc<RwLock<CacheStats>>,
}

impl TypeInformationCache {
    pub fn new() -> Self {
        Self {
            class_info_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(1000).unwrap())
            )),
            simple_type_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(100).unwrap())
            )),
            inference_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(500).unwrap())
            )),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    pub async fn get_class_info(&self, type_name: &str) -> Option<FhirPathTypeInfo> {
        let mut cache = self.class_info_cache.write().await;
        let result = cache.get(type_name).cloned();
        
        // Update stats
        let mut stats = self.cache_stats.write().await;
        if result.is_some() {
            stats.class_info_hits += 1;
        } else {
            stats.class_info_misses += 1;
        }
        
        result
    }
    
    pub async fn cache_class_info(&self, type_name: &str, type_info: FhirPathTypeInfo) {
        let mut cache = self.class_info_cache.write().await;
        cache.put(type_name.to_string(), type_info);
    }
    
    pub async fn get_stats(&self) -> CacheStats {
        *self.cache_stats.read().await
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    pub class_info_hits: u64,
    pub class_info_misses: u64,
    pub simple_type_hits: u64,
    pub simple_type_misses: u64,
    pub inference_hits: u64,
    pub inference_misses: u64,
}
```

## Implementation Tasks

### Week 1: Core Type Function Implementation
- [ ] Implement EnhancedTypeFunction with basic type resolution
- [ ] Add support for all FhirPathValue variants
- [ ] Create specification-compliant type info structures
- [ ] Implement caching infrastructure

### Week 2: Advanced Type Resolution
- [ ] Add FHIR resource type detection and classification
- [ ] Implement choice type handling (value[x] patterns)
- [ ] Add reference type resolution with target constraints
- [ ] Implement collection element type inference

### Week 3: Performance and Integration
- [ ] Optimize type operations for zero allocation
- [ ] Add SIMD-optimized type comparison operations
- [ ] Integrate with existing registry system
- [ ] Create comprehensive test suite

## Success Criteria

### Functional Requirements
- [ ] All FHIRPath official type() tests pass
- [ ] Specification compliance verified against §11 requirements
- [ ] Support for all healthcare-specific type patterns

### Performance Requirements
- [ ] Type() function completes in <1μs average
- [ ] Zero allocations during steady-state operation
- [ ] Cache hit rate >95% for common type patterns
- [ ] Memory usage optimized with intelligent caching

### Quality Requirements
- [ ] 100% test coverage for type function implementation
- [ ] Zero compiler warnings
- [ ] Documentation coverage >95%
- [ ] Performance benchmarks established

## Dependencies

### Phase 1 Dependencies
- FhirPathTypeInfo data structures
- String interning system
- Fast element lookup infrastructure
- Type constraint system foundation

### External Dependencies
- LRU cache implementation
- SIMD optimization libraries
- Performance profiling tools

## Testing Strategy

### Unit Tests
- Type function for all FhirPathValue variants
- Cache functionality and performance
- Type inference algorithms
- Error handling scenarios

### Integration Tests
- FHIRPath official test suite compliance
- Real FHIR resource type detection
- Complex type scenarios
- Performance validation

### Specification Compliance Tests
- All examples from §11 Types and Reflection
- Edge cases and error conditions
- Type hierarchy verification
- Cardinality handling

## Deliverables

1. **Enhanced Type Function** - Specification-compliant implementation
2. **Type Information Cache** - High-performance caching system
3. **Type Inference Engine** - Advanced type resolution capabilities
4. **Test Suite** - Comprehensive testing coverage
5. **Performance Benchmarks** - Optimized operation metrics
6. **Documentation** - API documentation and usage examples

## Next Phase Integration

This phase enables:
- **Phase 3:** FHIRSchema integration for complete type information
- **Phase 4:** Advanced constraint validation and type checking
- **Phase 5:** IDE integration and developer tooling support