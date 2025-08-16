# FHIRPath Type Reflection System Analysis & Implementation Plan

## Current State Analysis

### Existing Type System
- Basic `type()` function exists in `crates/fhirpath-registry/src/operations/types/type_func.rs`
- Returns simple `TypeInfoObject` with `namespace` and `name`
- Basic type mapping for System types (Boolean, Integer, String, etc.) and FHIR resources
- No formal reflection system for complex types

### FHIRPath Specification Requirements
Based on § 11 Types and Reflection:

1. **SimpleTypeInfo** for primitive types:
   ```
   SimpleTypeInfo { namespace: string, name: string, baseType: TypeSpecifier }
   ```

2. **ClassInfo** for complex types:
   ```
   ClassInfoElement { name: string, type: TypeSpecifier, isOneBased: Boolean }
   ClassInfo { namespace: string, name: string, baseType: TypeSpecifier, element: List<ClassInfoElement> }
   ```

3. **TupleInfo** for tuples:
   ```
   TupleTypeInfoElement { name: string, type: TypeSpecifier }
   TupleTypeInfo { element: List<TupleTypeInfoElement> }
   ```

## Minimal Implementation Design

### Core Types to Add

```rust
// Enhanced type info objects for FHIRPath reflection
#[derive(Clone, PartialEq)]
pub enum TypeInfoVariant {
    Simple(SimpleTypeInfo),
    Class(ClassInfo),
    Tuple(TupleInfo),
}

#[derive(Clone, PartialEq)]
pub struct SimpleTypeInfo {
    pub namespace: Arc<str>,
    pub name: Arc<str>,
    pub base_type: Option<Arc<str>>, // e.g. "System.Any"
}

#[derive(Clone, PartialEq)]
pub struct ClassInfoElement {
    pub name: Arc<str>,
    pub type_specifier: Arc<str>,
    pub is_one_based: bool,
}

#[derive(Clone, PartialEq)]
pub struct ClassInfo {
    pub namespace: Arc<str>,
    pub name: Arc<str>,
    pub base_type: Option<Arc<str>>,
    pub elements: Vec<ClassInfoElement>,
}

#[derive(Clone, PartialEq)]
pub struct TupleTypeInfoElement {
    pub name: Arc<str>,
    pub type_specifier: Arc<str>,
}

#[derive(Clone, PartialEq)]
pub struct TupleInfo {
    pub elements: Vec<TupleTypeInfoElement>,
}
```

### Enhanced FhirPathValue

Add new variant to `FhirPathValue`:
```rust
/// Enhanced type information object with full reflection support
TypeInfoVariant(TypeInfoVariant),
```

### ModelProvider Integration

Extend `ModelProvider` trait to support type reflection:
```rust
async fn get_type_reflection(&self, type_name: &str) -> Result<Option<TypeInfoVariant>, ModelError>;
async fn get_element_info(&self, type_name: &str, element_name: &str) -> Result<Option<ClassInfoElement>, ModelError>;
```

### Performance Optimizations

1. **Lazy Loading**: Only resolve detailed type info when `type()` function is called
2. **Caching**: Cache type reflection results in ModelProvider
3. **Arena Allocation**: Use arena for temporary type info objects during evaluation

## Implementation Strategy

### Phase 1: Core Type System
- Add new type info variants to `FhirPathValue`
- Enhance existing `type()` function to return detailed type info
- Implement basic System type reflections

### Phase 2: FHIR Schema Integration
- Extend `FhirSchemaModelProvider` with type reflection capabilities
- Map FHIRSchema elements to `ClassInfoElement`
- Support complex FHIR resource type reflection

### Phase 3: Optimization & Testing
- Add comprehensive test coverage
- Performance benchmarking
- Memory usage optimization

## Minimal Implementation Focus

To achieve maximum spec compliance with minimal complexity:

1. **System Types**: Full `SimpleTypeInfo` support for all primitive types
2. **FHIR Resources**: Basic `ClassInfo` with major elements from FHIRSchema
3. **Collections**: Proper type reflection for collection elements
4. **Caching**: Simple in-memory cache for type info objects

## Expected Compliance Improvement

Current: Basic type names only
Target: Full FHIRPath § 11 compliance with SimpleTypeInfo and ClassInfo support

This minimal implementation will significantly improve specification compliance while maintaining performance and avoiding over-engineering.