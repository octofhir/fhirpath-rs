# ADR-003: Enhanced FHIRPath Type Reflection System

**Status:** Proposed  
**Date:** 2025-01-16  
**Authors:** Claude Code (AI Assistant)  
**Reviewers:** TBD  

## Context

The current FHIRSchema-based ModelProvider implementation provides basic type information but falls short of the comprehensive type reflection system required for a best-in-class FHIRPath library. After analyzing the FHIRPath specification §11 Types and Reflection alongside production healthcare systems requirements, several critical gaps emerge that prevent our library from achieving industry-leading status.

### Current State Analysis

**Strengths:**
- Async-first architecture with proper ModelProvider trait
- Schema caching and performance optimization
- Basic type reflection via `TypeReflectionInfo` enum
- Integration with FHIRSchema package management
- Zero-copy optimizations in critical paths

**Critical Gaps for Best-in-Class Status:**

1. **Specification Compliance Gaps:**
   - Current `type()` function returns simplified type objects instead of spec-compliant reflection structures
   - Missing support for anonymous types (TupleTypeInfo) essential for complex healthcare data modeling
   - Limited inheritance hierarchy traversal preventing proper polymorphic type handling
   - No support for type element metadata (isOneBased, cardinality) required for robust validation

2. **Enterprise Healthcare Requirements:**
   - Lacks multi-version FHIR support (R4, R4B, R5 simultaneous operation)
   - No support for custom implementation guides and profiles
   - Missing constraint validation integration with type reflection
   - Insufficient performance for real-time clinical decision support systems

3. **Developer Experience Deficiencies:**
   - No type safety guarantees at compile time for FHIRPath expressions
   - Missing intelligent code completion and validation
   - Lack of comprehensive error reporting with type context
   - No support for type-aware expression optimization

4. **Ecosystem Integration Limitations:**
   - Cannot support Language Server Protocol (LSP) for IDE integration
   - Missing support for schema evolution and migration
   - No integration with formal verification tools
   - Lacks support for cross-resource type resolution in Bundle contexts

### FHIRPath Specification Requirements

According to §11, the `type()` function must return concrete TypeInfo subtypes:

1. **SimpleTypeInfo** for primitives: `{ namespace: string, name: string, baseType: TypeSpecifier }`
2. **ClassInfo** for classes: `{ namespace: string, name: string, baseType: TypeSpecifier, element: List<ClassInfoElement> }`
3. **ListTypeInfo** for collections: `{ elementType: TypeSpecifier }`
4. **TupleTypeInfo** for anonymous types: `{ element: List<TupleTypeInfoElement> }`

## Decision

We will implement a revolutionary type reflection system that not only achieves full FHIRPath specification compliance but establishes new industry standards for healthcare interoperability libraries. This system will serve as the foundation for next-generation clinical decision support, schema validation, and developer tooling.

### Best-in-Class Design Principles

1. **Zero-Compromise Specification Compliance**: 100% implementation of FHIRPath TypeInfo with extensions for healthcare-specific requirements
2. **Performance Leadership**: Sub-microsecond type lookups with intelligent multi-level caching and lock-free data structures
3. **Type Safety Guarantees**: Compile-time verification of FHIRPath expressions through advanced type inference
4. **Healthcare-Grade Reliability**: Formal verification, exhaustive testing, and production-proven error handling
5. **Ecosystem Enablement**: Foundation for IDE tools, LSP servers, and automated validation systems
6. **Future-Proof Architecture**: Support for FHIR evolution, custom profiles, and emerging standards

### Strategic Competitive Advantages

1. **Industry-First Features**: 
   - Multi-version FHIR support with automatic schema translation
   - Real-time constraint validation during type reflection
   - Intelligent type inference for incomplete expressions
   - Cross-resource type resolution in complex Bundle scenarios

2. **Performance Innovation**:
   - Lock-free type cache with sub-microsecond access times
   - Lazy evaluation with intelligent prefetching
   - Memory-efficient type representations using arena allocation
   - SIMD-optimized type comparison operations

3. **Developer Experience Revolution**:
   - IDE-grade code completion with contextual type hints
   - Compile-time FHIRPath expression validation
   - Automatic error recovery and suggestion system
   - Visual type hierarchy exploration tools

## Detailed Design

### 1. Industry-Leading Type Reflection Architecture

Create a comprehensive type system that exceeds specification requirements while enabling advanced healthcare use cases:

```rust
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::intern::StringInterner;

/// Zero-allocation type information using interned strings for memory efficiency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FhirPathTypeInfo {
    SimpleTypeInfo {
        namespace: Arc<str>,
        name: Arc<str>,
        base_type: Option<Arc<str>>,
        /// Healthcare extension: UCUM unit constraints for quantities
        constraints: Option<Arc<TypeConstraints>>,
    },
    ClassInfo {
        namespace: Arc<str>,
        name: Arc<str>,
        base_type: Option<Arc<str>>,
        elements: Arc<[ClassInfoElement]>,
        /// Healthcare extension: FHIR profile constraints
        profile_constraints: Option<Arc<ProfileConstraints>>,
        /// Performance optimization: Pre-computed element lookup table
        element_index: Arc<FastElementLookup>,
    },
    ListTypeInfo {
        element_type: Box<FhirPathTypeInfo>,
        /// Specification compliance: Collection constraints
        cardinality: Cardinality,
    },
    TupleTypeInfo {
        elements: Arc<[TupleTypeInfoElement]>,
        /// Healthcare extension: Support for backbone elements
        is_backbone_element: bool,
    },
    /// Healthcare extension: Choice types (e.g., value[x])
    ChoiceTypeInfo {
        base_name: Arc<str>,
        choices: Arc<[FhirPathTypeInfo]>,
        /// Optimized lookup for choice resolution
        choice_index: Arc<ChoiceTypeLookup>,
    },
    /// Healthcare extension: Reference types with target constraints
    ReferenceTypeInfo {
        target_types: Arc<[Arc<str>]>,
        reference_type: ReferenceKind,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassInfoElement {
    pub name: Arc<str>,
    pub type_specifier: Arc<str>,
    pub is_one_based: bool,
    pub cardinality: Cardinality,
    /// Healthcare extension: Element-level constraints from profiles
    pub constraints: Option<Arc<ElementConstraints>>,
    /// Performance: Pre-computed hash for fast comparison
    pub name_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TupleTypeInfoElement {
    pub name: Arc<str>,
    pub type_specifier: Arc<str>,
    pub is_one_based: bool,
    pub cardinality: Cardinality,
}

/// Healthcare-grade cardinality with performance optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cardinality {
    pub min: u32,
    pub max: Option<u32>, // None = unbounded
}

/// Fast element lookup using perfect hashing for O(1) access
#[derive(Debug)]
pub struct FastElementLookup {
    hash_table: Box<[Option<u16>]>, // Indices into elements array
    elements_count: u16,
}

/// Advanced constraint system for healthcare compliance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeConstraints {
    pub value_constraints: Vec<ValueConstraint>,
    pub fhirpath_constraints: Vec<FhirPathConstraint>,
    pub terminology_bindings: Vec<TerminologyBinding>,
}

/// Profile-level constraints from FHIR Implementation Guides
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileConstraints {
    pub profile_url: Arc<str>,
    pub must_support: Vec<Arc<str>>,
    pub slicing_rules: Vec<SlicingRule>,
    pub extension_constraints: Vec<ExtensionConstraint>,
}
```

### 2. Next-Generation ModelProvider Architecture

Revolutionary ModelProvider design that enables unprecedented performance and functionality:

```rust
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait AdvancedModelProvider: Send + Sync + std::fmt::Debug {
    // Core type reflection - specification compliant
    async fn get_fhirpath_type_info(&self, type_name: &str) -> Result<FhirPathTypeInfo, TypeReflectionError>;
    
    async fn get_element_type_info(
        &self,
        type_name: &str,
        element_path: &str,
    ) -> Result<FhirPathTypeInfo, TypeReflectionError>;
    
    // Healthcare-specific enhancements
    async fn get_type_hierarchy(&self, type_name: &str) -> Result<TypeHierarchy, TypeReflectionError>;
    
    async fn resolve_type_specifier(&self, specifier: &str) -> Result<ResolvedType, TypeReflectionError>;
    
    async fn get_choice_type_info(
        &self,
        base_type: &str,
        choices: &[String],
    ) -> Result<ChoiceTypeInfo, TypeReflectionError>;
    
    // Multi-version FHIR support
    async fn get_type_info_for_version(
        &self,
        type_name: &str,
        fhir_version: FhirVersion,
    ) -> Result<FhirPathTypeInfo, TypeReflectionError>;
    
    // Profile and implementation guide support
    async fn get_profiled_type_info(
        &self,
        type_name: &str,
        profile_url: &str,
    ) -> Result<ProfiledTypeInfo, TypeReflectionError>;
    
    // Advanced constraint validation
    async fn validate_type_constraints(
        &self,
        value: &FhirPathValue,
        type_info: &FhirPathTypeInfo,
        context: &ValidationContext,
    ) -> Result<ConstraintValidationResult, TypeReflectionError>;
    
    // Performance optimizations
    async fn batch_get_type_info(
        &self,
        type_names: &[&str],
    ) -> Result<HashMap<String, FhirPathTypeInfo>, TypeReflectionError>;
    
    // Type inference for incomplete expressions
    async fn infer_expression_type(
        &self,
        expression: &str,
        context_type: &str,
    ) -> Result<TypeInferenceResult, TypeReflectionError>;
    
    // Cross-resource type resolution for Bundles
    async fn resolve_bundle_reference_type(
        &self,
        reference: &str,
        bundle_context: &BundleContext,
    ) -> Result<FhirPathTypeInfo, TypeReflectionError>;
    
    // IDE and tooling support
    async fn get_completion_candidates(
        &self,
        partial_path: &str,
        context_type: &str,
    ) -> Result<Vec<CompletionCandidate>, TypeReflectionError>;
    
    // Schema evolution support
    async fn compare_type_versions(
        &self,
        type_name: &str,
        version_a: FhirVersion,
        version_b: FhirVersion,
    ) -> Result<TypeEvolutionReport, TypeReflectionError>;
}

/// Comprehensive type hierarchy with inheritance and composition relationships
#[derive(Debug, Clone)]
pub struct TypeHierarchy {
    pub type_name: Arc<str>,
    pub ancestors: Vec<Arc<str>>, // Inheritance chain
    pub descendants: Vec<Arc<str>>, // Known subtypes
    pub composition_relationships: Vec<CompositionRelationship>,
    pub interface_implementations: Vec<Arc<str>>, // For type unions
}

/// Resolved type with full context information
#[derive(Debug, Clone)]
pub struct ResolvedType {
    pub type_info: FhirPathTypeInfo,
    pub resolution_context: ResolutionContext,
    pub confidence_score: f32, // For ambiguous resolutions
    pub alternatives: Vec<AlternativeResolution>,
}

/// Profiled type with implementation guide constraints
#[derive(Debug, Clone)]
pub struct ProfiledTypeInfo {
    pub base_type: FhirPathTypeInfo,
    pub profile_constraints: ProfileConstraints,
    pub slicing_information: Vec<SlicingInfo>,
    pub must_support_elements: Vec<Arc<str>>,
    pub prohibited_elements: Vec<Arc<str>>,
}

/// Advanced type inference results
#[derive(Debug, Clone)]
pub struct TypeInferenceResult {
    pub inferred_type: FhirPathTypeInfo,
    pub confidence: f32,
    pub inference_steps: Vec<InferenceStep>,
    pub warnings: Vec<TypeWarning>,
    pub optimization_hints: Vec<OptimizationHint>,
}

/// IDE completion support
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub name: Arc<str>,
    pub type_info: FhirPathTypeInfo,
    pub documentation: Option<Arc<str>>,
    pub completion_kind: CompletionKind,
    pub relevance_score: f32,
}

#[derive(Debug, Clone)]
pub enum CompletionKind {
    Property,
    Function,
    Operator,
    Constant,
    Type,
}
```

### 3. Enhanced Type Function Implementation

Update the `type()` function to return specification-compliant reflection objects:

```rust
impl TypeFunction {
    async fn evaluate_with_reflection(
        &self,
        value: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let type_info = match value {
            FhirPathValue::String(_) => FhirPathTypeInfo::SimpleTypeInfo {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: Some("System.Any".to_string()),
            },
            FhirPathValue::Resource(resource) => {
                self.get_resource_type_info(resource, context).await?
            },
            FhirPathValue::Collection(items) => FhirPathTypeInfo::ListTypeInfo {
                element_type: Box::new(self.get_collection_element_type(items).await?),
            },
            // Handle other types...
        };
        
        Ok(FhirPathValue::TypeReflectionObject(type_info))
    }
}
```

### 4. FHIRSchema Integration Enhancements

Enhance the FhirSchemaModelProvider to leverage FHIRSchema data for complete type information:

```rust
impl FhirSchemaModelProvider {
    async fn build_class_info(&self, schema: &FhirSchema) -> Option<FhirPathTypeInfo> {
        let elements = self.extract_class_elements(schema).await?;
        
        Some(FhirPathTypeInfo::ClassInfo {
            namespace: "FHIR".to_string(),
            name: schema.name.clone(),
            base_type: self.get_base_type_from_schema(schema).await,
            elements,
        })
    }
    
    async fn extract_class_elements(&self, schema: &FhirSchema) -> Option<Vec<ClassInfoElement>> {
        let mut elements = Vec::new();
        
        for (path, element) in &schema.elements {
            if let Some(element_name) = self.extract_direct_element_name(path) {
                elements.push(ClassInfoElement {
                    name: element_name,
                    type_specifier: self.build_type_specifier(element).await?,
                    is_one_based: element.is_summary.unwrap_or(false),
                    min_cardinality: element.min,
                    max_cardinality: element.max.as_ref().and_then(|m| m.parse().ok()),
                });
            }
        }
        
        Some(elements)
    }
}
```

### 5. Performance Optimizations

Implement intelligent caching strategies:

```rust
pub struct TypeReflectionCache {
    type_info_cache: Arc<RwLock<LruCache<String, FhirPathTypeInfo>>>,
    hierarchy_cache: Arc<RwLock<LruCache<String, Vec<String>>>>,
    element_cache: Arc<RwLock<LruCache<String, Vec<ClassInfoElement>>>>,
}

impl TypeReflectionCache {
    async fn get_or_compute<F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> Option<FhirPathTypeInfo>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<FhirPathTypeInfo>>,
    {
        // Check cache first, compute and cache if missing
    }
}
```

## Implementation Phases

### Phase 1: Core Type Reflection Infrastructure
- Implement `FhirPathTypeInfo` and related structures
- Create type reflection cache system
- Add new ModelProvider trait methods
- Update existing TypeReflectionInfo compatibility layer

### Phase 2: Enhanced Type Function
- Implement specification-compliant `type()` function
- Add support for all TypeInfo concrete subtypes
- Integrate with ModelProvider type reflection
- Add comprehensive test coverage

### Phase 3: FHIRSchema Integration
- Enhance FhirSchemaModelProvider with full type reflection
- Implement choice type handling
- Add inheritance hierarchy traversal
- Optimize schema-to-reflection conversion

### Phase 4: Advanced Features
- Anonymous type support (TupleTypeInfo)
- Custom type system extensibility
- Type suggestion and auto-completion
- Performance profiling and optimization

### Phase 5: Validation and Compliance
- Comprehensive FHIRPath specification testing
- Performance benchmarking
- Documentation and examples
- Migration guide for existing code

## Benefits

1. **Full Specification Compliance**: Complete implementation of FHIRPath type reflection
2. **Enhanced Developer Experience**: Rich type information for tooling and debugging
3. **Improved Type Safety**: Better compile-time and runtime type checking
4. **Performance**: Intelligent caching and async-first design
5. **Extensibility**: Support for custom type systems and future enhancements
6. **Backward Compatibility**: Existing code continues to work without changes

## Risks and Mitigations

### Risk: Performance Impact
- **Mitigation**: Comprehensive caching strategy and lazy evaluation
- **Monitoring**: Benchmark against current implementation

### Risk: Implementation Complexity
- **Mitigation**: Phased implementation with clear milestones
- **Testing**: Extensive test coverage at each phase

### Risk: Breaking Changes
- **Mitigation**: Maintain backward compatibility through adapter pattern
- **Documentation**: Clear migration path for advanced users

## Success Criteria

1. **Functional**: All FHIRPath official tests pass with enhanced type reflection
2. **Performance**: No regression in existing benchmarks
3. **Compliance**: 100% coverage of §11 Types and Reflection specification
4. **Quality**: Zero compiler warnings and full documentation coverage
5. **Integration**: Seamless operation with existing FHIRSchema provider

## Alternatives Considered

### Alternative 1: Minimal Enhancement
- **Approach**: Add basic TypeInfo objects without full specification compliance
- **Rejected**: Insufficient for advanced FHIRPath use cases and tooling

### Alternative 2: Separate Reflection Service
- **Approach**: Create standalone type reflection service
- **Rejected**: Adds unnecessary complexity and doesn't leverage ModelProvider architecture

### Alternative 3: Synchronous Implementation
- **Approach**: Implement type reflection with synchronous APIs
- **Rejected**: Conflicts with async-first architecture principle

## Future Considerations

1. **Language Server Protocol (LSP)**: Enhanced type reflection enables rich IDE support
2. **Code Generation**: Type information can drive automatic code generation
3. **Schema Validation**: Advanced validation capabilities using type metadata
4. **Cross-Version Compatibility**: Support for multiple FHIR versions simultaneously

## References

- [FHIRPath Specification §11 Types and Reflection](../specs/§11-types-and-reflection.md)
- [FHIR R4 Structure Definitions](http://hl7.org/fhir/R4/structuredefinition.html)
- [Clinical Quality Language Tooling](https://github.com/cqframework/clinical_quality_language)
- [Rust Async Programming Best Practices](https://rust-lang.github.io/async-book/)