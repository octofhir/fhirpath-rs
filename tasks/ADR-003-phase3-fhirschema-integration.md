# ADR-003 Phase 3: FHIRSchema Integration Enhancements

**Related ADR:** [ADR-003: Enhanced FHIRPath Type Reflection System](../docs/adr/ADR-003-fhirpath-type-reflection-system.md)  
**Phase:** 3 of 5  
**Status:** Planned  
**Priority:** High  
**Estimated Effort:** 3-4 weeks  
**Prerequisites:** Phase 1 and Phase 2 completed

## Objective

Enhance the FhirSchemaModelProvider to fully leverage the existing FHIRSchema library's StructureDefinition translation capabilities, extending FHIRSchema as needed to provide complete, specification-compliant type reflection while supporting multi-version FHIR compatibility and custom implementation guides.

## Scope

### In Scope
1. **FHIRSchema Library Extensions**
   - Extend FHIRSchema API to expose type reflection metadata
   - Add FHIRPath-specific type conversion utilities
   - Enhance element type resolution capabilities
   - Add profile constraint extraction methods

2. **Multi-Version FHIR Support via FHIRSchema**
   - Leverage FHIRSchema's package management for R4, R4B, R5
   - Utilize FHIRSchema's version detection capabilities
   - Build on FHIRSchema's cross-version compatibility
   - Extend FHIRSchema with type compatibility checking

3. **Enhanced Type Information Extraction**
   - Utilize FHIRSchema's StructureDefinition parsing
   - Extract choice type (value[x]) information from FHIRSchema
   - Use FHIRSchema's backbone element resolution
   - Leverage FHIRSchema's inheritance hierarchy data

4. **Profile Integration via FHIRSchema**
   - Extend FHIRSchema to extract profile constraints
   - Use FHIRSchema's must-support element identification
   - Leverage FHIRSchema's slicing rule interpretation
   - Build on FHIRSchema's extension handling

### Out of Scope
- Runtime constraint validation (Phase 4)
- IDE integration features (Phase 4)
- Advanced type inference beyond schema (Phase 4)

## Technical Implementation

### 1. FHIRSchema Library Extensions

**File:** `../fhirschema-rs/src/fhirpath_integration.rs` (in parent directory)

```rust
use crate::{FhirSchema, Element, StructureDefinition};
use std::collections::HashMap;
use std::sync::Arc;

/// Extensions to FHIRSchema for FHIRPath type reflection
pub trait FhirSchemaFhirPathExt {
    /// Extract FHIRPath-compatible type information from schema
    fn get_fhirpath_type_info(&self, type_name: &str) -> Option<FhirPathTypeMetadata>;
    
    /// Get element type information with FHIRPath semantics
    fn get_element_fhirpath_type(&self, path: &str) -> Option<FhirPathElementInfo>;
    
    /// Extract choice type information (value[x] patterns)
    fn get_choice_type_info(&self, base_path: &str) -> Option<ChoiceTypeMetadata>;
    
    /// Get inheritance hierarchy for type
    fn get_type_hierarchy(&self, type_name: &str) -> Option<TypeHierarchy>;
    
    /// Extract profile constraints for FHIRPath validation
    fn get_profile_constraints(&self, profile_url: &str) -> Option<ProfileConstraints>;
}

impl FhirSchemaFhirPathExt for FhirSchema {
    fn get_fhirpath_type_info(&self, type_name: &str) -> Option<FhirPathTypeMetadata> {
        // Leverage existing schema parsing to extract type metadata
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{}", type_name);
        
        // Use existing schema element data
        let type_elements = self.get_type_elements(type_name)?;
        let base_type = self.get_base_type(type_name);
        
        Some(FhirPathTypeMetadata {
            name: type_name.to_string(),
            namespace: "FHIR".to_string(),
            base_type,
            elements: type_elements,
            is_resource: self.is_resource_type(type_name),
            is_primitive: self.is_primitive_type(type_name),
        })
    }
    
    fn get_element_fhirpath_type(&self, path: &str) -> Option<FhirPathElementInfo> {
        // Use existing element resolution from FHIRSchema
        let element = self.elements.get(path)?;
        
        Some(FhirPathElementInfo {
            name: self.extract_element_name(path)?,
            type_specifier: self.build_type_specifier_from_element(element)?,
            cardinality: self.extract_cardinality(element),
            is_choice_type: self.is_choice_element(element),
            documentation: element.definition.clone(),
        })
    }
    
    fn get_choice_type_info(&self, base_path: &str) -> Option<ChoiceTypeMetadata> {
        // Find all value[x] variants for the base path
        let choice_elements = self.find_choice_variants(base_path)?;
        
        Some(ChoiceTypeMetadata {
            base_name: base_path.to_string(),
            choices: choice_elements,
        })
    }
}

/// FHIRPath-specific type metadata extracted from FHIRSchema
#[derive(Debug, Clone)]
pub struct FhirPathTypeMetadata {
    pub name: String,
    pub namespace: String,
    pub base_type: Option<String>,
    pub elements: Vec<FhirPathElementInfo>,
    pub is_resource: bool,
    pub is_primitive: bool,
}

/// Element information optimized for FHIRPath operations
#[derive(Debug, Clone)]
pub struct FhirPathElementInfo {
    pub name: String,
    pub type_specifier: String,
    pub cardinality: FhirPathCardinality,
    pub is_choice_type: bool,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FhirPathCardinality {
    pub min: u32,
    pub max: Option<u32>, // None = unbounded
}
```

### 2. Enhanced FhirSchemaModelProvider Integration

**File:** `crates/fhirpath-model/src/fhirschema_provider.rs` (enhanced)

```rust
use octofhir_fhirschema::{FhirSchemaPackageManager, FhirSchemaFhirPathExt};
use crate::reflection::{FhirPathTypeInfo, ClassInfoElement};

impl FhirSchemaModelProvider {
    /// Enhanced type reflection using FHIRSchema extensions
    async fn get_fhirpath_type_info_enhanced(
        &self,
        type_name: &str,
    ) -> Option<FhirPathTypeInfo> {
        // Use FHIRSchema's existing capabilities with our extensions
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{type_name}");
        let schema = self.get_schema_cached(&canonical_url).await?;
        
        // Leverage FHIRSchema's FHIRPath integration
        let metadata = schema.get_fhirpath_type_info(type_name)?;
        
        // Convert to our FhirPathTypeInfo format
        self.convert_metadata_to_type_info(metadata).await
    }
    
    async fn convert_metadata_to_type_info(
        &self,
        metadata: FhirPathTypeMetadata,
    ) -> Option<FhirPathTypeInfo> {
        if metadata.is_primitive {
            Some(FhirPathTypeInfo::SimpleTypeInfo {
                namespace: self.interner.intern(&metadata.namespace),
                name: self.interner.intern(&metadata.name),
                base_type: metadata.base_type.map(|bt| self.interner.intern(&bt)),
                constraints: None,
            })
        } else {
            // Convert elements using FHIRSchema's processed data
            let elements = self.convert_elements_from_metadata(&metadata.elements).await?;
            let element_index = self.create_fast_lookup(&elements);
            
            Some(FhirPathTypeInfo::ClassInfo {
                namespace: self.interner.intern(&metadata.namespace),
                name: self.interner.intern(&metadata.name),
                base_type: metadata.base_type.map(|bt| self.interner.intern(&bt)),
                elements: elements.into(),
                element_index: Arc::new(element_index),
                profile_constraints: None,
            })
        }
    }
    
    async fn convert_elements_from_metadata(
        &self,
        elements: &[FhirPathElementInfo],
    ) -> Option<Vec<ClassInfoElement>> {
        let mut converted_elements = Vec::new();
        
        for element in elements {
            converted_elements.push(ClassInfoElement {
                name: self.interner.intern(&element.name),
                type_specifier: self.interner.intern(&element.type_specifier),
                is_one_based: false, // FHIR is zero-based
                cardinality: Cardinality {
                    min: element.cardinality.min,
                    max: element.cardinality.max,
                },
                constraints: None,
                name_hash: self.compute_hash(&element.name),
            });
        }
        
        Some(converted_elements)
    }
    
    /// Enhanced choice type resolution using FHIRSchema
    async fn resolve_choice_type(
        &self,
        base_path: &str,
        schema: &FhirSchema,
    ) -> Option<FhirPathTypeInfo> {
        let choice_metadata = schema.get_choice_type_info(base_path)?;
        
        let choice_types: Vec<FhirPathTypeInfo> = choice_metadata
            .choices
            .iter()
            .filter_map(|choice| self.convert_choice_element(choice))
            .collect();
        
        if choice_types.is_empty() {
            return None;
        }
        
        Some(FhirPathTypeInfo::ChoiceTypeInfo {
            base_name: self.interner.intern(&choice_metadata.base_name),
            choices: choice_types.into(),
            choice_index: Arc::new(self.create_choice_lookup(&choice_types)),
        })
    }
}
```

### 3. FHIRSchema Extensions Implementation Strategy

**Strategy for extending FHIRSchema in parent directory:**

```rust
// In ../fhirschema-rs/src/lib.rs - add new module
pub mod fhirpath_integration;
        
        for version in &config.supported_versions {
            let package_manager = Self::create_version_manager(*version, &config).await?;
            version_managers.insert(*version, Arc::new(package_manager));
        }
        
        Ok(Self {
            version_managers,
            default_version: config.default_version,
            cross_version_mappings: Arc::new(RwLock::new(CrossVersionMappings::new())),
            schema_cache: Arc::new(RwLock::new(MultiVersionSchemaCache::new())),
        })
    }
    
    pub async fn get_schema_for_version(
        &self,
        canonical_url: &str,
        version: FhirVersion,
    ) -> Option<Arc<FhirSchema>> {
        // Check cache first
        {
            let cache = self.schema_cache.read().await;
            if let Some(schema) = cache.get(canonical_url, version) {
                return Some(schema);
            }
        }
        
        // Fetch from appropriate version manager
        let manager = self.version_managers.get(&version)?;
        let schema = manager.get_schema(canonical_url).await?;
        
        // Cache the result
        {
            let mut cache = self.schema_cache.write().await;
            cache.insert(canonical_url, version, schema.clone());
        }
        
        Some(schema)
    }
    
    pub async fn get_compatible_type(
        &self,
        type_name: &str,
        source_version: FhirVersion,
        target_version: FhirVersion,
    ) -> Option<String> {
        if source_version == target_version {
            return Some(type_name.to_string());
        }
        
        let mappings = self.cross_version_mappings.read().await;
        mappings.get_mapping(type_name, source_version, target_version)
    }
}

#[derive(Debug, Clone)]
pub struct MultiVersionConfig {
    pub supported_versions: Vec<FhirVersion>,
    pub default_version: FhirVersion,
    pub auto_install_packages: bool,
    pub cross_version_compatibility: bool,
}
```

### 2. Enhanced Schema-to-TypeInfo Conversion

**File:** `crates/fhirpath-model/src/schema/type_converter.rs`

```rust
use octofhir_fhirschema::{FhirSchema, Element as FhirSchemaElement};
use crate::reflection::{FhirPathTypeInfo, ClassInfoElement, TupleTypeInfoElement, Cardinality};

/// Converts FHIRSchema elements to FhirPathTypeInfo
pub struct SchemaTypeConverter {
    interner: Arc<StringInterner>,
    primitive_type_mappings: HashMap<String, String>,
}

impl SchemaTypeConverter {
    pub fn new() -> Self {
        let mut primitive_mappings = HashMap::new();
        primitive_mappings.insert("boolean".to_string(), "Boolean".to_string());
        primitive_mappings.insert("integer".to_string(), "Integer".to_string());
        primitive_mappings.insert("string".to_string(), "String".to_string());
        primitive_mappings.insert("decimal".to_string(), "Decimal".to_string());
        primitive_mappings.insert("date".to_string(), "Date".to_string());
        primitive_mappings.insert("dateTime".to_string(), "DateTime".to_string());
        primitive_mappings.insert("instant".to_string(), "DateTime".to_string());
        primitive_mappings.insert("time".to_string(), "Time".to_string());
        
        Self {
            interner: Arc::new(StringInterner::global()),
            primitive_type_mappings: primitive_mappings,
        }
    }
    
    pub async fn convert_schema_to_class_info(
        &self,
        schema: &FhirSchema,
    ) -> Result<FhirPathTypeInfo, ModelError> {
        let type_name = self.extract_type_name_from_url(&schema.url)?;
        let elements = self.extract_class_elements(schema, &type_name).await?;
        let base_type = self.extract_base_type(schema);
        
        // Create element lookup table for O(1) access
        let element_index = self.create_element_index(&elements);
        
        Ok(FhirPathTypeInfo::ClassInfo {
            namespace: self.interner.intern("FHIR"),
            name: self.interner.intern(&type_name),
            base_type: base_type.map(|bt| self.interner.intern(&bt)),
            elements: elements.into(),
            element_index: Arc::new(element_index),
            profile_constraints: None, // Will be populated by profile resolver
        })
    }
    
    async fn extract_class_elements(
        &self,
        schema: &FhirSchema,
        type_name: &str,
    ) -> Result<Vec<ClassInfoElement>, ModelError> {
        let mut elements = Vec::new();
        
        for (path, element) in &schema.elements {
            if let Some(element_name) = self.extract_direct_element_name(path, type_name) {
                let class_element = self.convert_element_to_class_info_element(
                    &element_name,
                    element,
                ).await?;
                elements.push(class_element);
            }
        }
        
        // Sort by name for consistent ordering
        elements.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(elements)
    }
    
    async fn convert_element_to_class_info_element(
        &self,
        element_name: &str,
        element: &FhirSchemaElement,
    ) -> Result<ClassInfoElement, ModelError> {
        let type_specifier = self.build_type_specifier(element).await?;
        let cardinality = self.extract_cardinality(element);
        
        Ok(ClassInfoElement {
            name: self.interner.intern(element_name),
            type_specifier: self.interner.intern(&type_specifier),
            is_one_based: false, // FHIR is zero-based
            cardinality,
            constraints: None, // Will be populated by constraint resolver
            name_hash: self.compute_name_hash(element_name),
        })
    }
    
    async fn build_type_specifier(
        &self,
        element: &FhirSchemaElement,
    ) -> Result<String, ModelError> {
        if let Some(types) = &element.element_type {
            if types.is_empty() {
                return Ok("System.Any".to_string());
            }
            
            if types.len() == 1 {
                // Single type
                let type_code = &types[0].code;
                let mapped_type = self.map_fhir_type_to_fhirpath(type_code);
                Ok(mapped_type)
            } else {
                // Choice type - return union representation
                let type_names: Vec<String> = types
                    .iter()
                    .map(|t| self.map_fhir_type_to_fhirpath(&t.code))
                    .collect();
                Ok(format!("Union<{}>", type_names.join(", ")))
            }
        } else {
            Ok("System.Any".to_string())
        }
    }
    
    fn map_fhir_type_to_fhirpath(&self, fhir_type: &str) -> String {
        if let Some(mapped) = self.primitive_type_mappings.get(fhir_type) {
            format!("System.{}", mapped)
        } else {
            format!("FHIR.{}", fhir_type)
        }
    }
    
    fn extract_cardinality(&self, element: &FhirSchemaElement) -> Cardinality {
        let min = element.min.unwrap_or(0);
        let max = element.max.as_ref()
            .and_then(|m| if m == "*" { None } else { m.parse().ok() });
        
        Cardinality { min, max }
    }
    
    fn extract_direct_element_name(&self, path: &str, type_name: &str) -> Option<String> {
        let prefix = format!("{}.", type_name);
        if let Some(element_path) = path.strip_prefix(&prefix) {
            // Only include direct children (no nested paths)
            if !element_path.contains('.') {
                Some(element_path.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn extract_base_type(&self, schema: &FhirSchema) -> Option<String> {
        schema.base_definition.as_ref().and_then(|base_url| {
            base_url.path_segments()?.last().map(|s| s.to_string())
        })
    }
}
```

### 3. Profile and Implementation Guide Integration

**File:** `crates/fhirpath-model/src/schema/profile_resolver.rs`

```rust
use octofhir_fhirschema::{FhirSchema, StructureDefinition};
use crate::reflection::{ProfileConstraints, ProfiledTypeInfo};

/// Resolves profile constraints and implementation guide requirements
pub struct ProfileResolver {
    schema_manager: Arc<MultiVersionSchemaManager>,
    profile_cache: Arc<RwLock<HashMap<String, ProfileConstraints>>>,
}

impl ProfileResolver {
    pub async fn resolve_profile_constraints(
        &self,
        profile_url: &str,
        base_type: &str,
    ) -> Result<ProfileConstraints, ModelError> {
        // Check cache first
        {
            let cache = self.profile_cache.read().await;
            if let Some(constraints) = cache.get(profile_url) {
                return Ok(constraints.clone());
            }
        }
        
        // Fetch profile schema
        let profile_schema = self.schema_manager
            .get_schema_for_version(profile_url, FhirVersion::R4)
            .await
            .ok_or_else(|| ModelError::profile_not_found(profile_url))?;
        
        // Extract constraints from profile
        let constraints = self.extract_profile_constraints(&profile_schema).await?;
        
        // Cache the result
        {
            let mut cache = self.profile_cache.write().await;
            cache.insert(profile_url.to_string(), constraints.clone());
        }
        
        Ok(constraints)
    }
    
    async fn extract_profile_constraints(
        &self,
        profile_schema: &FhirSchema,
    ) -> Result<ProfileConstraints, ModelError> {
        let mut must_support = Vec::new();
        let mut slicing_rules = Vec::new();
        let mut extension_constraints = Vec::new();
        
        // Extract must-support elements
        for (path, element) in &profile_schema.elements {
            if element.must_support.unwrap_or(false) {
                must_support.push(Arc::from(path.as_str()));
            }
            
            // Extract slicing information
            if let Some(slicing) = &element.slicing {
                let slicing_rule = SlicingRule {
                    path: Arc::from(path.as_str()),
                    discriminator: slicing.discriminator.clone(),
                    rules: slicing.rules.clone(),
                    ordered: slicing.ordered.unwrap_or(false),
                };
                slicing_rules.push(slicing_rule);
            }
        }
        
        Ok(ProfileConstraints {
            profile_url: Arc::from(profile_schema.url.as_str()),
            must_support,
            slicing_rules,
            extension_constraints,
        })
    }
}
```

## Implementation Tasks

### Week 1: FHIRSchema Library Extensions
- [ ] Add `fhirpath_integration` module to FHIRSchema library
- [ ] Implement `FhirSchemaFhirPathExt` trait
- [ ] Add FHIRPath-specific metadata extraction methods
- [ ] Create type conversion utilities leveraging existing schema parsing

### Week 2: Enhanced Type Information Extraction
- [ ] Implement `get_fhirpath_type_info` using FHIRSchema's StructureDefinition data
- [ ] Add choice type (value[x]) resolution via `get_choice_type_info`
- [ ] Support backbone elements using FHIRSchema's element hierarchy
- [ ] Extract inheritance information from FHIRSchema's base type data

### Week 3: FhirSchemaModelProvider Integration
- [ ] Update FhirSchemaModelProvider to use new FHIRSchema extensions
- [ ] Implement type conversion from FHIRSchema metadata to FhirPathTypeInfo
- [ ] Add caching layer for converted type information
- [ ] Test with existing FHIRSchema package management

### Week 4: Profile and Multi-Version Support
- [ ] Extend FHIRSchema to extract profile constraints
- [ ] Leverage FHIRSchema's multi-package support for versions
- [ ] Add profile-specific type information extraction
- [ ] Comprehensive testing with real implementation guides

## Success Criteria

### Functional Requirements
- [ ] FHIRSchema extensions provide complete type metadata
- [ ] Seamless integration with existing FHIRSchema capabilities
- [ ] Profile constraint extraction from FHIRSchema data
- [ ] Choice type resolution using FHIRSchema's StructureDefinition parsing

### Performance Requirements
- [ ] Schema lookup <10μs average across versions
- [ ] Profile constraint resolution <100μs
- [ ] Memory usage optimized with intelligent caching
- [ ] Cross-version compatibility checks <1μs

### Quality Requirements
- [ ] 100% test coverage for schema integration
- [ ] Support for all major implementation guides
- [ ] Zero data loss in schema conversion
- [ ] Comprehensive error handling

## Dependencies

### Phase Dependencies
- Phase 1: Core type reflection infrastructure
- Phase 2: Enhanced type function implementation

### External Dependencies
- FHIRSchema package manager enhancements
- Implementation guide packages (US Core, IPS, etc.)
- Profile validation utilities

## Testing Strategy

### Multi-Version Testing
- Simultaneous R4, R4B, R5 operation
- Cross-version type compatibility
- Version-specific schema handling
- Migration scenario testing

### Profile Integration Testing
- US Core profile constraints
- International Patient Summary (IPS)
- Custom implementation guides
- Slicing rule interpretation

### Performance Testing
- Schema caching effectiveness
- Profile resolution performance
- Memory usage optimization
- Concurrent access patterns

## Deliverables

1. **Multi-Version Schema Manager** - Support for multiple FHIR versions
2. **Enhanced Schema Converter** - Complete schema-to-TypeInfo conversion
3. **Profile Resolver** - Implementation guide constraint integration
4. **Cross-Version Mapping** - Type compatibility across FHIR versions
5. **Test Suite** - Comprehensive multi-version testing
6. **Documentation** - Integration guide and examples

## Integration Points

This phase provides:
- Complete schema-based type information for Phase 4
- Profile constraint foundation for validation
- Multi-version support for enterprise deployment
- Enhanced type resolution for IDE integration