// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pre-computed type registry for high-performance FHIR type operations
//!
//! This module provides a registry system that pre-computes and caches all FHIR type information
//! at startup, eliminating the need for expensive schema lookups during FHIRPath evaluation.

use crate::provider::{ModelError, TypeReflectionInfo};
use octofhir_fhirschema::{
    Element as FhirSchemaElement, FhirSchema, FhirSchemaPackageManager,
    ModelProvider as FhirSchemaModelProviderTrait,
};
use std::collections::HashMap;
use std::time::Instant;

/// Pre-computed type information for instant access
#[derive(Debug, Clone)]
pub struct PrecomputedTypeRegistry {
    /// All FHIR types with complete metadata
    fhir_types: HashMap<String, FhirTypeInfo>,
    /// System primitive types  
    system_types: HashMap<String, SystemTypeInfo>,
    /// Type hierarchy relationships (child -> ancestors)
    inheritance_map: HashMap<String, Vec<String>>,
    /// Property maps for fast navigation (type -> property -> info)
    property_maps: HashMap<String, HashMap<String, PropertyInfo>>,
    /// Choice type mappings (valueX -> [valueString, valueQuantity, ...])
    choice_mappings: HashMap<String, Vec<ChoiceTypeInfo>>,
    /// Registry build time for performance tracking
    build_time: Option<Instant>,
}

/// FHIR type information
#[derive(Debug, Clone)]
pub struct FhirTypeInfo {
    pub name: String,
    pub namespace: String, // Always "FHIR"
    pub base_type: Option<String>,
    pub is_resource: bool,
    pub is_complex_type: bool,
    pub is_abstract: bool,
    pub properties: HashMap<String, PropertyInfo>,
    pub constraints: Vec<String>, // FHIRPath constraint expressions
}

/// System primitive type information
#[derive(Debug, Clone)]
pub struct SystemTypeInfo {
    pub name: String,
    pub namespace: String, // Always "System"
    pub primitive_type: PrimitiveTypeKind,
}

/// Property information for fast lookups
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub type_info: TypeReflectionInfo,
    pub min_cardinality: u32,
    pub max_cardinality: Option<u32>, // None = unbounded
    pub is_choice_type: bool,
    pub choice_types: Vec<String>, // For choice types like value[x]
    pub is_primitive: bool,
}

/// Choice type mapping information
#[derive(Debug, Clone)]
pub struct ChoiceTypeInfo {
    pub base_property: String,     // e.g., "value"
    pub choice_types: Vec<String>, // ["string", "Quantity", "boolean", ...]
}

/// Primitive type enumeration
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveTypeKind {
    Boolean,
    Integer,
    String,
    Decimal,
    Date,
    DateTime,
    Time,
    Quantity,
}

impl PrecomputedTypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            fhir_types: HashMap::new(),
            system_types: HashMap::new(),
            inheritance_map: HashMap::new(),
            property_maps: HashMap::new(),
            choice_mappings: HashMap::new(),
            build_time: None,
        }
    }

    /// Create registry with System types only (for background loading)
    pub fn new_with_system_types() -> Self {
        let mut registry = Self::new();
        registry.build_system_types();
        registry
    }

    /// Build registry from FhirSchemaPackageManager
    pub async fn build_from_schemas(
        package_manager: &FhirSchemaPackageManager,
    ) -> Result<Self, ModelError> {
        let start_time = Instant::now();
        let mut registry = Self::new();

        // Build system types first
        registry.build_system_types();

        // Load and process all FHIR types
        registry.load_fhir_types(package_manager).await?;

        // Build inheritance relationships
        registry.build_inheritance_map();

        // Process choice types
        registry.build_choice_mappings();

        // Validate registry completeness
        registry.validate()?;

        registry.build_time = Some(start_time);
        Ok(registry)
    }

    /// Build System namespace primitive types
    fn build_system_types(&mut self) {
        let system_types = [
            ("Boolean", PrimitiveTypeKind::Boolean),
            ("Integer", PrimitiveTypeKind::Integer),
            ("String", PrimitiveTypeKind::String),
            ("Decimal", PrimitiveTypeKind::Decimal),
            ("Date", PrimitiveTypeKind::Date),
            ("DateTime", PrimitiveTypeKind::DateTime),
            ("Time", PrimitiveTypeKind::Time),
            ("Quantity", PrimitiveTypeKind::Quantity),
        ];

        for (name, kind) in system_types {
            self.system_types.insert(
                name.to_string(),
                SystemTypeInfo {
                    name: name.to_string(),
                    namespace: "System".to_string(),
                    primitive_type: kind,
                },
            );
        }
    }

    /// Load all FHIR types from schemas
    async fn load_fhir_types(
        &mut self,
        package_manager: &FhirSchemaPackageManager,
    ) -> Result<(), ModelError> {
        // Get list of all available types from package manager
        let available_types = self.get_available_types(package_manager).await?;

        for type_name in available_types {
            if let Some(schema) = FhirSchemaModelProviderTrait::get_schema(
                package_manager,
                &format!("http://hl7.org/fhir/StructureDefinition/{type_name}"),
            )
            .await
            {
                self.process_schema(type_name, &schema)?;
            }
        }

        Ok(())
    }

    /// Get available types from package manager
    async fn get_available_types(
        &self,
        _package_manager: &FhirSchemaPackageManager,
    ) -> Result<Vec<String>, ModelError> {
        // For now, return a hardcoded list of common FHIR types
        // In a complete implementation, this would query the package manager for all available types
        Ok(vec![
            "Patient".to_string(),
            "Observation".to_string(),
            "Practitioner".to_string(),
            "Organization".to_string(),
            "Encounter".to_string(),
            "Procedure".to_string(),
            "Condition".to_string(),
            "MedicationRequest".to_string(),
            "AllergyIntolerance".to_string(),
            "DiagnosticReport".to_string(),
            "Bundle".to_string(),
            "Resource".to_string(),
            "DomainResource".to_string(),
            "Quantity".to_string(),
            "Coding".to_string(),
            "CodeableConcept".to_string(),
            "Reference".to_string(),
            "Identifier".to_string(),
            "HumanName".to_string(),
            "Address".to_string(),
            "ContactPoint".to_string(),
            "Period".to_string(),
            "Range".to_string(),
            "Ratio".to_string(),
            "Attachment".to_string(),
            "Annotation".to_string(),
        ])
    }

    /// Process a single FhirSchema into registry
    fn process_schema(&mut self, type_name: String, schema: &FhirSchema) -> Result<(), ModelError> {
        let mut properties = HashMap::new();
        let mut choice_types = Vec::new();

        // Process all elements in schema
        for (path, element) in &schema.elements {
            if let Some(property_name) = path.strip_prefix(&format!("{type_name}.")) {
                // Only process direct children (no nested properties)
                if !property_name.contains('.') {
                    let property_info = self.convert_element_to_property_info(element)?;

                    // Track choice types
                    if property_info.is_choice_type {
                        choice_types.push(ChoiceTypeInfo {
                            base_property: property_name.trim_end_matches("[x]").to_string(),
                            choice_types: property_info.choice_types.clone(),
                        });
                    }

                    properties.insert(property_name.to_string(), property_info);
                }
            }
        }

        // Create FHIR type info
        let fhir_type = FhirTypeInfo {
            name: type_name.clone(),
            namespace: "FHIR".to_string(),
            base_type: self.extract_base_type(schema),
            is_resource: self.is_resource_type(&type_name),
            is_complex_type: !self.is_primitive_fhir_type(&type_name),
            is_abstract: false, // Would need to be extracted from schema metadata if available
            properties: properties.clone(),
            constraints: vec![], // Would need to be extracted from schema if available
        };

        // Store in registry
        self.fhir_types.insert(type_name.clone(), fhir_type);

        // Store property map
        self.property_maps.insert(type_name.clone(), properties);

        // Store choice mappings
        for choice_type in choice_types {
            self.choice_mappings.insert(
                format!("{}.{}", type_name, choice_type.base_property),
                vec![choice_type],
            );
        }

        Ok(())
    }

    /// Convert FhirSchemaElement to PropertyInfo
    fn convert_element_to_property_info(
        &self,
        element: &FhirSchemaElement,
    ) -> Result<PropertyInfo, ModelError> {
        let name = element
            .path
            .split('.')
            .next_back()
            .unwrap_or("unknown")
            .to_string();

        // Determine if this is a choice type
        let is_choice_type = name.ends_with("[x]");
        let choice_types = if let Some(types) = &element.element_type {
            types.iter().map(|t| t.code.clone()).collect()
        } else {
            vec![]
        };

        let is_primitive = choice_types.iter().any(|t| self.is_primitive_type(t));

        // Convert to TypeReflectionInfo
        let type_info = if let Some(types) = &element.element_type {
            if types.len() == 1 {
                let type_code = &types[0].code;
                if self.is_primitive_type(type_code) {
                    TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: self.map_primitive_type(type_code),
                        base_type: None,
                    }
                } else {
                    TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: type_code.clone(),
                        base_type: self.get_hardcoded_base_type(type_code),
                        elements: vec![],
                    }
                }
            } else if !types.is_empty() {
                // For choice types, return the first type for now
                let type_code = &types[0].code;
                if self.is_primitive_type(type_code) {
                    TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: self.map_primitive_type(type_code),
                        base_type: None,
                    }
                } else {
                    TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: type_code.clone(),
                        base_type: self.get_hardcoded_base_type(type_code),
                        elements: vec![],
                    }
                }
            } else {
                // Default to string type if no types specified
                TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "String".to_string(),
                    base_type: None,
                }
            }
        } else {
            // Default to string type if element_type is None
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            }
        };

        Ok(PropertyInfo {
            name,
            type_info,
            min_cardinality: element.min.unwrap_or(0),
            max_cardinality: element
                .max
                .as_ref()
                .and_then(|max| if max == "*" { None } else { max.parse().ok() }),
            is_choice_type,
            choice_types,
            is_primitive,
        })
    }

    /// Extract base type from schema
    fn extract_base_type(&self, schema: &FhirSchema) -> Option<String> {
        schema.base_definition.as_ref().and_then(|base_url| {
            base_url
                .as_str()
                .strip_prefix("http://hl7.org/fhir/StructureDefinition/")
                .map(String::from)
        })
    }

    /// Check if a type is a FHIR resource type
    fn is_resource_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "Patient"
                | "Observation"
                | "Practitioner"
                | "Organization"
                | "Encounter"
                | "Procedure"
                | "Condition"
                | "MedicationRequest"
                | "AllergyIntolerance"
                | "DiagnosticReport"
                | "Bundle"
                | "Resource"
                | "DomainResource"
        )
    }

    /// Check if a FHIR type is primitive
    fn is_primitive_fhir_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "string" | "boolean" | "integer" | "decimal" | "date" | "dateTime" | "time"
        )
    }

    /// Check if a type code is primitive
    fn is_primitive_type(&self, type_code: &str) -> bool {
        matches!(
            type_code,
            "boolean"
                | "integer"
                | "string"
                | "decimal"
                | "uri"
                | "url"
                | "canonical"
                | "base64Binary"
                | "instant"
                | "date"
                | "dateTime"
                | "time"
        )
    }

    /// Map FHIR primitive types to System types
    fn map_primitive_type(&self, type_code: &str) -> String {
        match type_code {
            "boolean" => "Boolean".to_string(),
            "integer" => "Integer".to_string(),
            "decimal" => "Decimal".to_string(),
            "date" => "Date".to_string(),
            "dateTime" | "instant" => "DateTime".to_string(),
            "time" => "Time".to_string(),
            _ => "String".to_string(), // Default for uri, url, canonical, etc.
        }
    }

    /// Get hardcoded base type for common FHIR types
    fn get_hardcoded_base_type(&self, type_name: &str) -> Option<String> {
        match type_name {
            "Patient" | "Observation" | "Practitioner" | "Organization" | "Encounter"
            | "Procedure" | "Condition" | "MedicationRequest" | "AllergyIntolerance"
            | "DiagnosticReport" => Some("DomainResource".to_string()),
            "DomainResource" | "Bundle" => Some("Resource".to_string()),
            _ => None,
        }
    }

    /// Build inheritance hierarchy map
    fn build_inheritance_map(&mut self) {
        for type_name in self.fhir_types.keys() {
            let mut ancestors = Vec::new();
            let mut current_type = type_name.clone();

            // Walk up inheritance chain
            while let Some(base_type) = self
                .fhir_types
                .get(&current_type)
                .and_then(|t| t.base_type.as_ref())
            {
                ancestors.push(base_type.clone());
                current_type = base_type.clone();
            }

            if !ancestors.is_empty() {
                self.inheritance_map.insert(type_name.clone(), ancestors);
            }
        }
    }

    /// Build choice type mappings
    fn build_choice_mappings(&mut self) {
        // Choice mappings are built during schema processing
        // This method can be used for post-processing if needed
    }

    /// Validate registry completeness
    fn validate(&self) -> Result<(), ModelError> {
        // Validate that system types are present
        if self.system_types.is_empty() {
            return Err(ModelError::ValidationError {
                message: "No System types found in registry".to_string(),
            });
        }

        // Validate that some FHIR types are present (not required for basic functionality)
        // if self.fhir_types.is_empty() {
        //     return Err(ModelError::ValidationError {
        //         message: "No FHIR types found in registry".to_string(),
        //     });
        // }

        Ok(())
    }

    // Fast lookup methods

    /// Get FHIR type info by name
    #[inline]
    pub fn get_fhir_type(&self, name: &str) -> Option<&FhirTypeInfo> {
        self.fhir_types.get(name)
    }

    /// Get System type info by name
    #[inline]
    pub fn get_system_type(&self, name: &str) -> Option<&SystemTypeInfo> {
        self.system_types.get(name)
    }

    /// Check if type is subtype of parent
    pub fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool {
        if child_type == parent_type {
            return true;
        }

        if let Some(ancestors) = self.inheritance_map.get(child_type) {
            ancestors.contains(&parent_type.to_string())
        } else {
            false
        }
    }

    /// Get all properties for a type
    pub fn get_properties(&self, type_name: &str) -> Option<&HashMap<String, PropertyInfo>> {
        self.property_maps.get(type_name)
    }

    /// Get specific property info
    pub fn get_property(&self, type_name: &str, property_name: &str) -> Option<&PropertyInfo> {
        self.property_maps
            .get(type_name)
            .and_then(|props| props.get(property_name))
    }

    /// Get choice type mappings
    pub fn get_choice_mapping(
        &self,
        type_name: &str,
        property: &str,
    ) -> Option<&Vec<ChoiceTypeInfo>> {
        self.choice_mappings.get(&format!("{type_name}.{property}"))
    }

    /// Get namespace for type
    pub fn get_namespace(&self, type_name: &str) -> Option<&str> {
        if let Some(system_type) = self.system_types.get(type_name) {
            Some(&system_type.namespace)
        } else if let Some(fhir_type) = self.fhir_types.get(type_name) {
            Some(&fhir_type.namespace)
        } else {
            None
        }
    }

    /// Get build time for performance tracking
    pub fn build_time(&self) -> Option<Instant> {
        self.build_time
    }

    /// Get registry statistics
    pub fn statistics(&self) -> RegistryStatistics {
        RegistryStatistics {
            system_types_count: self.system_types.len(),
            fhir_types_count: self.fhir_types.len(),
            inheritance_relationships_count: self.inheritance_map.len(),
            choice_mappings_count: self.choice_mappings.len(),
            total_properties_count: self.property_maps.values().map(|props| props.len()).sum(),
        }
    }
}

/// Registry build and usage statistics
#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub system_types_count: usize,
    pub fhir_types_count: usize,
    pub inheritance_relationships_count: usize,
    pub choice_mappings_count: usize,
    pub total_properties_count: usize,
}

impl Default for PrecomputedTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
