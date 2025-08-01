# ADR-005: Model Provider for Type Checking and Type Conversion

## Status
Proposed

## Context

The current FHIRPath implementation has basic type checking capabilities but lacks a comprehensive model provider system for accurate type checking and type conversion as specified in the FHIRPath specification. This limitation affects:

1. **Type Safety**: The FHIRPath specification (§12) emphasizes the importance of type safety and strict evaluation, allowing implementations to choose different levels of compile-time vs. runtime type checking.

2. **Polymorphic Types**: FHIR resources contain many polymorphic elements (choice types like `value[x]`) that require model-aware type resolution.

3. **Type Reflection**: The specification (§11) defines type reflection capabilities with `SimpleTypeInfo`, `ClassInfo`, `ListTypeInfo`, and `TupleTypeInfo` structures.

4. **Model Context**: Different FHIR versions (R4, R4B, R5) and other healthcare standards (CDA) require different type models.

5. **Performance**: Type checking at compile-time can prevent runtime errors and improve performance.

### Current Implementation Analysis

**Strengths:**
- Basic `TypeInfo` enum covering primitive and collection types (src/model/types.rs:7-68)
- `ModelProvider` trait with essential methods (src/model/provider.rs:39-152)
- `FhirTypeRegistry` with type hierarchy and polymorphic element support (src/types.rs:11-467)
- Type compatibility checking and conversion support

**Limitations:**
- No integration between the type registry and model provider
- Limited type reflection capabilities
- No version-specific model data
- Incomplete polymorphic type resolution
- No compile-time type checking pipeline

### FHIRPath.js Analysis

The reference JavaScript implementation provides several key insights:

1. **Model Data Structure**: Uses objects with `choiceTypePaths`, `pathsDefinedElsewhere`, `type2Parent`, and `path2Type` properties
2. **Version Support**: Separate model files for different FHIR versions (R4, R5, etc.)
3. **Type Validation**: Parameter type validation in user-defined functions
4. **Internal Types**: Handles FHIR-specific internal types with conversion capabilities

## Decision

We will implement a comprehensive Model Provider system with the following components:

### 1. Enhanced Model Provider Architecture

```rust
pub trait ModelProvider: Send + Sync {
    // Core type information
    fn get_type_info(&self, type_name: &str) -> Option<TypeInfo>;
    fn get_property_type(&self, parent_type: &str, property: &str) -> Option<TypeInfo>;
    
    // Polymorphic type support
    fn resolve_polymorphic_property(&self, parent_type: &str, property: &str, value: &FhirPathValue) -> Option<String>;
    fn get_choice_type_paths(&self) -> &HashMap<String, Vec<String>>;
    
    // Type hierarchy and validation
    fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool;
    fn validate_type_assignment(&self, value_type: &TypeInfo, target_type: &TypeInfo) -> TypeCheckResult;
    
    // Reflection support
    fn get_simple_type_info(&self, type_name: &str) -> Option<SimpleTypeInfo>;
    fn get_class_info(&self, type_name: &str) -> Option<ClassInfo>;
    fn get_list_type_info(&self, element_type: &str) -> Option<ListTypeInfo>;
    
    // Context information
    fn get_fhir_version(&self) -> FhirVersion;
    fn get_namespace(&self) -> &str;
}
```

### 2. Type Checking Integration

- **Compile-time checking**: Optional strict type checking during expression parsing
- **Runtime validation**: Type compatibility checking during evaluation
- **Error reporting**: Enhanced diagnostics with type mismatch information

### 3. Version-Specific Model Providers

```rust
pub struct FhirR4ModelProvider {
    type_registry: FhirTypeRegistry,
    structure_definitions: HashMap<String, StructureDefinition>,
    choice_type_paths: HashMap<String, Vec<String>>,
    path_to_type_map: HashMap<String, String>,
}

pub struct FhirR5ModelProvider {
    // Similar structure with R5-specific data
}
```

### 4. Type Reflection System

Implement the FHIRPath specification type reflection structures:

```rust
#[derive(Debug, Clone)]
pub struct SimpleTypeInfo {
    pub namespace: String,
    pub name: String,
    pub base_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub namespace: String,
    pub name: String,
    pub base_type: Option<String>,
    pub elements: Vec<ClassInfoElement>,
}

#[derive(Debug, Clone)]  
pub struct ClassInfoElement {
    pub name: String,
    pub type_specifier: String,
    pub is_one_based: bool,
}
```

### 5. Polymorphic Type Resolution

Enhanced support for FHIR choice types:

```rust
impl ModelProvider for FhirModelProvider {
    fn resolve_polymorphic_property(&self, parent_type: &str, property: &str, value: &FhirPathValue) -> Option<String> {
        // Resolve actual property name based on value type
        // e.g., "value" + value type -> "valueString", "valueQuantity", etc.
    }
}
```

## Implementation Plan

### Phase 1: Core Model Provider Enhancement
- [ ] Extend `ModelProvider` trait with reflection and validation methods
- [ ] Implement type checking result structures and error types
- [ ] Add compile-time type checking configuration options

### Phase 2: FHIR Model Provider Implementation  
- [ ] Create FHIR R4 model provider with structure definitions
- [ ] Implement choice type resolution logic
- [ ] Add type hierarchy validation
- [ ] Support for polymorphic property resolution

### Phase 3: Type Reflection System
- [ ] Implement `SimpleTypeInfo`, `ClassInfo`, `ListTypeInfo` structures
- [ ] Add `type()` function support for reflection
- [ ] Integrate reflection with existing type system

### Phase 4: Integration and Testing
- [ ] Integrate model provider with expression evaluator
- [ ] Add compile-time type checking to parser
- [ ] Implement comprehensive test suite
- [ ] Performance optimization and benchmarking

### Phase 5: Multi-Version Support
- [ ] Add FHIR R5 model provider
- [ ] Support for version-specific type checking
- [ ] Model provider registry and selection mechanism

## Consequences

### Positive
- **Improved Type Safety**: Compile-time and runtime type validation prevents common errors
- **Better Performance**: Early type checking reduces runtime validation overhead
- **Spec Compliance**: Full implementation of FHIRPath type system and reflection
- **Extensibility**: Support for multiple FHIR versions and other healthcare standards
- **Developer Experience**: Better error messages and IDE support

### Negative
- **Complexity**: Increased codebase complexity with multiple model providers
- **Memory Usage**: Type registry and structure definitions require additional memory
- **Build Time**: Compile-time type checking may increase parsing time
- **Maintenance**: Need to maintain multiple FHIR version definitions

### Risks
- **Performance Impact**: Type checking overhead during expression evaluation
- **Compatibility**: Changes may affect existing API consumers
- **Data Accuracy**: Type definitions must be kept in sync with FHIR specifications

## Alternatives Considered

### 1. Minimal Type Checking
Continue with current basic type checking without model provider enhancement.
**Rejected**: Doesn't meet FHIRPath specification requirements for type safety.

### 2. Runtime-Only Validation
Implement comprehensive type checking only at runtime.
**Rejected**: Misses opportunity for compile-time error detection and performance optimization.

### 3. External Type Definition Files
Use external JSON/YAML files for type definitions.
**Rejected**: Adds dependency management complexity and runtime file I/O overhead.

## References

- [FHIRPath Specification §11: Types and Reflection](specs/§11-types-and-reflection.md)
- [FHIRPath Specification §12: Type Safety and Strict Evaluation](specs/§12-type-safety-and-strict-evaluation.md) 
- [FHIRPath.js Model Provider Implementation](https://github.com/hl7/fhirpath.js)
- [FHIR Structure Definitions](https://www.hl7.org/fhir/structuredefinition.html)
- [Current Implementation: src/model/provider.rs](src/model/provider.rs)
- [Current Implementation: src/types.rs](src/types.rs)