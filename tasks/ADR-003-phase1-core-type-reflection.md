# ADR-003 Phase 1: Core Type Reflection Infrastructure

**Related ADR:** [ADR-003: Enhanced FHIRPath Type Reflection System](../docs/adr/ADR-003-fhirpath-type-reflection-system.md)  
**Phase:** 1 of 5  
**Status:** Planned  
**Priority:** Critical  
**Estimated Effort:** 3-4 weeks  

## Objective

Implement the foundational type reflection infrastructure that provides specification-compliant FHIRPath TypeInfo structures while establishing the performance and architectural foundation for subsequent phases.

## Scope

### In Scope
1. **Core Type Reflection Data Structures**
   - `FhirPathTypeInfo` enum with all specification-required variants
   - `ClassInfoElement` and `TupleTypeInfoElement` structures
   - Healthcare-specific extensions (ChoiceTypeInfo, ReferenceTypeInfo)
   - Memory-efficient design using Arc<str> and interned strings

2. **String Interning System**
   - High-performance string interner for zero-allocation type operations
   - Thread-safe intern table with concurrent access optimization
   - Memory usage tracking and cleanup strategies

3. **Fast Element Lookup System**
   - Perfect hash table implementation for O(1) element access
   - Pre-computed hash values for type and element names
   - Cache-friendly data layout optimization

4. **Type Constraint System Foundation**
   - Basic constraint types (TypeConstraints, ElementConstraints)
   - Cardinality representation with performance optimization
   - Foundation for profile constraint integration

5. **Error Handling Framework**
   - `TypeReflectionError` comprehensive error type
   - Error context preservation for debugging
   - Performance-optimized error paths

### Out of Scope
- Complex constraint validation logic (Phase 4)
- Multi-version FHIR support (Phase 3)
- IDE integration features (Phase 4)
- Advanced type inference (Phase 4)

## Technical Implementation

### 1. Core Data Structures

**File:** `crates/fhirpath-model/src/reflection/types.rs`

```rust
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::intern::StringInterner;

/// Specification-compliant FHIRPath type information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FhirPathTypeInfo {
    /// Primitive types: Boolean, Integer, String, etc.
    SimpleTypeInfo {
        namespace: Arc<str>,
        name: Arc<str>,
        base_type: Option<Arc<str>>,
        constraints: Option<Arc<TypeConstraints>>,
    },
    
    /// Complex types: Patient, Observation, etc.
    ClassInfo {
        namespace: Arc<str>,
        name: Arc<str>,
        base_type: Option<Arc<str>>,
        elements: Arc<[ClassInfoElement]>,
        element_index: Arc<FastElementLookup>,
        profile_constraints: Option<Arc<ProfileConstraints>>,
    },
    
    /// Collection types: List<Patient>, etc.
    ListTypeInfo {
        element_type: Box<FhirPathTypeInfo>,
        cardinality: Cardinality,
    },
    
    /// Anonymous types from backbone elements
    TupleTypeInfo {
        elements: Arc<[TupleTypeInfoElement]>,
        is_backbone_element: bool,
    },
    
    /// FHIR choice types: value[x]
    ChoiceTypeInfo {
        base_name: Arc<str>,
        choices: Arc<[FhirPathTypeInfo]>,
        choice_index: Arc<ChoiceTypeLookup>,
    },
    
    /// FHIR reference types with target constraints
    ReferenceTypeInfo {
        target_types: Arc<[Arc<str>]>,
        reference_type: ReferenceKind,
    },
}
```

### 2. String Interning System

**File:** `crates/fhirpath-model/src/intern.rs`

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use dashmap::DashMap;

/// High-performance string interner for type system
pub struct StringInterner {
    strings: DashMap<String, Arc<str>>,
    stats: Arc<RwLock<InternStats>>,
}

impl StringInterner {
    pub fn intern(&self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            return interned.clone();
        }
        
        let arc_str: Arc<str> = Arc::from(s);
        self.strings.insert(s.to_string(), arc_str.clone());
        arc_str
    }
    
    pub fn get_stats(&self) -> InternStats {
        *self.stats.read().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InternStats {
    pub total_strings: usize,
    pub memory_usage: usize,
    pub hit_rate: f64,
}
```

### 3. Fast Element Lookup

**File:** `crates/fhirpath-model/src/reflection/lookup.rs`

```rust
use std::sync::Arc;

/// O(1) element lookup using perfect hashing
#[derive(Debug)]
pub struct FastElementLookup {
    hash_table: Box<[Option<u16>]>,
    elements_count: u16,
    hash_function: HashFunction,
}

impl FastElementLookup {
    pub fn new(element_names: &[Arc<str>]) -> Self {
        let hash_function = HashFunction::create_perfect_hash(element_names);
        let table_size = hash_function.table_size();
        let mut hash_table = vec![None; table_size].into_boxed_slice();
        
        for (idx, name) in element_names.iter().enumerate() {
            let hash = hash_function.hash(name);
            hash_table[hash as usize] = Some(idx as u16);
        }
        
        Self {
            hash_table,
            elements_count: element_names.len() as u16,
            hash_function,
        }
    }
    
    pub fn find_element_index(&self, name: &str) -> Option<usize> {
        let hash = self.hash_function.hash(name);
        self.hash_table.get(hash as usize)?.map(|&idx| idx as usize)
    }
}
```

## Implementation Tasks

### Week 1: Foundation Infrastructure
- [ ] Create `crates/fhirpath-model/src/reflection/` module structure
- [ ] Implement string interning system with performance tests
- [ ] Design and implement basic FhirPathTypeInfo enum
- [ ] Create comprehensive unit tests for data structures

### Week 2: Performance Optimizations
- [ ] Implement FastElementLookup with perfect hashing
- [ ] Add memory-efficient Arc<str> usage throughout
- [ ] Implement cardinality representation optimization
- [ ] Create performance benchmarks for core operations

### Week 3: Constraint System Foundation
- [ ] Design TypeConstraints and ElementConstraints structures
- [ ] Implement profile constraint placeholder system
- [ ] Add constraint serialization/deserialization
- [ ] Create constraint validation framework foundation

### Week 4: Integration and Testing
- [ ] Integrate with existing ModelProvider trait
- [ ] Comprehensive testing with real FHIR schemas
- [ ] Performance validation against current benchmarks

## Success Criteria

### Functional Requirements
- [ ] All FHIRPath TypeInfo variants properly implemented
- [ ] String interning provides measurable memory reduction
- [ ] Element lookup achieves O(1) performance in benchmarks

### Performance Requirements
- [ ] Type lookup operations complete in <100ns average
- [ ] Memory usage reduced by >30% compared to current implementation
- [ ] Zero allocation during steady-state type operations
- [ ] Thread contention minimized in concurrent scenarios

### Quality Requirements
- [ ] 100% test coverage for core type reflection code
- [ ] Zero compiler warnings
- [ ] Documentation coverage >95%
- [ ] All code reviewed and approved

## Dependencies

### Internal Dependencies
- Current ModelProvider trait interface
- Existing FhirPathValue type system
- String handling utilities

### External Dependencies
- `dashmap` crate for concurrent hash maps
- `serde` for serialization support
- `arc-swap` for lock-free atomic operations

## Risks and Mitigations

### Risk: Memory Usage Increase
**Probability:** Medium  
**Impact:** High  
**Mitigation:** Comprehensive memory profiling and optimization of Arc usage

### Risk: Performance Regression
**Probability:** Low  
**Impact:** High  
**Mitigation:** Continuous benchmarking and performance testing throughout development

### Risk: Integration Complexity
**Probability:** Medium  
**Impact:** Medium  
**Mitigation:** Incremental integration with thorough testing at each step

## Testing Strategy

### Unit Tests
- All data structure operations
- String interning functionality
- Element lookup performance
- Serialization/deserialization

### Integration Tests
- ModelProvider compatibility
- Real FHIR schema processing
- Memory usage validation
- Concurrent access patterns

### Performance Tests
- Type lookup benchmarks
- Memory allocation profiling
- Cache hit rate measurement
- Scalability testing

## Deliverables

1. **Core Type Reflection Module** - Complete implementation of FhirPathTypeInfo system
2. **String Interning System** - High-performance string deduplication
3. **Fast Lookup Infrastructure** - O(1) element access implementation
4. **Test Suite** - Comprehensive testing coverage
5. **Performance Benchmarks** - Baseline measurements for subsequent phases
6. **Documentation** - API documentation and usage examples

## Next Phase Dependencies

This phase provides the foundation for:
- **Phase 2:** Enhanced Type Function implementation
- **Phase 3:** FHIRSchema integration enhancements
- **Phase 4:** Advanced features and validation
- **Phase 5:** Compliance testing and optimization