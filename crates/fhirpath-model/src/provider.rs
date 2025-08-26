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

//! Model provider trait for FHIR type information
//!
//! This module re-exports the enhanced ModelProvider from octofhir-fhir-model
//! and provides compatibility shims for the old interface.

// Re-export the enhanced ModelProvider and related types
pub use octofhir_fhir_model::provider::{
    BoxedValueWithMetadata, ConstraintViolation, DetailedConformanceResult, ElementDefinition,
    ElementType, EmptyModelProvider, ExpressionAnalysis, FhirPathAnalysisResult, FhirVersion,
    NavigationContext, NavigationValidation, PolymorphicTypeInfo, PrimitiveExtensionData,
    ProviderMetrics, ResolutionContext, SearchParameter, StructureDefinition, ValueReflection,
    ViolationSeverity,
};
pub use octofhir_fhirschema::PackageSpec;

// Import choice type mapper
use crate::choice_type_mapper::ChoiceVariant;

// Define our own async-first ModelProvider trait
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use sonic_rs::JsonValueTrait;

// Re-export type reflection system
pub use octofhir_fhir_model::reflection::{
    ElementInfo, TupleElementInfo, TypeHierarchy, TypeReflectionInfo, TypeSuggestion,
};

// Re-export conformance validation
pub use octofhir_fhir_model::conformance::{
    CacheStatistics, ConformanceMetadata, ConformanceResult, ConformanceValidator,
    ConformanceViolation as ConfViolation, ConformanceWarning, ProfileRule, RuleCategory,
    SourceLocation, ValidationContext, ValidationMetrics, ValidationMode, ValidationProfile,
    ValidationRule, ValidationRuleResult, ValidationScope,
};

// Re-export constraints
pub use octofhir_fhir_model::constraints::{
    ConstraintEvaluationStats, ConstraintInfo, ConstraintResult, ConstraintSeverity,
    ConstraintValue,
};

// Re-export enhanced boxing system
pub use octofhir_fhir_model::boxing::{
    BoxableValue, BoxedFhirPathValue, ComplexValue, Extension, PrimitiveExtension,
};

// Re-export error types
pub use octofhir_fhir_model::error::{ModelError, Result as ModelResult};

/// Components of a parsed FHIR reference URL
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceComponents {
    /// Resource type (e.g., "Patient")
    pub resource_type: String,
    /// Resource ID
    pub resource_id: String,
    /// Version ID (if specified with /_history/)
    pub version_id: Option<String>,
    /// Fragment identifier (for contained resources, starts with #)
    pub fragment: Option<String>,
    /// Full URL (if the reference was a complete URL)
    pub full_url: Option<String>,
    /// Base URL (extracted from full URL)
    pub base_url: Option<String>,
}

/// Async-first ModelProvider trait for FHIR type introspection and validation
/// This replaces the synchronous ModelProvider with an async-first design
#[async_trait]
pub trait ModelProvider: Send + Sync + std::fmt::Debug {
    /// Get type reflection information for a given FHIR type
    async fn get_type_reflection(&self, type_name: &str) -> Option<TypeReflectionInfo>;

    /// Get element reflection for a specific property path
    async fn get_element_reflection(
        &self,
        type_name: &str,
        element_path: &str,
    ) -> Option<TypeReflectionInfo>;

    /// Get property type information (alias for get_element_reflection)
    async fn get_property_type(
        &self,
        parent_type: &str,
        property: &str,
    ) -> Option<TypeReflectionInfo> {
        self.get_element_reflection(parent_type, property).await
    }

    /// Get structure definition for a type (optional)
    async fn get_structure_definition(&self, type_name: &str) -> Option<StructureDefinition> {
        let _ = type_name;
        None
    }

    /// Validate conformance to a profile
    async fn validate_conformance(
        &self,
        value: &dyn ValueReflection,
        profile_url: &str,
    ) -> Result<octofhir_fhir_model::conformance::ConformanceResult, ModelError>;

    /// Get constraints for a type
    async fn get_constraints(
        &self,
        type_name: &str,
    ) -> Vec<octofhir_fhir_model::constraints::ConstraintInfo>;

    /// Resolve a reference URL to a value
    async fn resolve_reference(
        &self,
        reference_url: &str,
        context: &dyn ResolutionContext,
    ) -> Option<Box<dyn ValueReflection>>;

    /// Resolve a reference to a FhirPathValue in the context of a specific Bundle or resource
    /// This is the enhanced method that the resolve() function should use
    async fn resolve_reference_in_context(
        &self,
        reference_url: &str,
        root_resource: &crate::FhirPathValue,
        current_resource: Option<&crate::FhirPathValue>,
    ) -> Option<crate::FhirPathValue>;

    /// Resolve a reference within a Bundle's entries by fullUrl or resource type/id
    async fn resolve_in_bundle(
        &self,
        reference_url: &str,
        bundle: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue>;

    /// Resolve a reference within contained resources (fragment references starting with #)
    async fn resolve_in_contained(
        &self,
        reference_url: &str,
        containing_resource: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue>;

    /// Resolve an external reference (ResourceType/id format) using FHIR server or other external provider
    async fn resolve_external_reference(&self, reference_url: &str)
    -> Option<crate::FhirPathValue>;

    /// Parse a reference URL into its components (resource type, id, version, fragment)
    fn parse_reference_url(&self, reference_url: &str) -> Option<ReferenceComponents>;

    /// Get the base URL for this provider's FHIR server (if any)
    fn get_base_fhir_url(&self) -> Option<String> {
        None
    }

    /// Analyze a FHIRPath expression
    async fn analyze_expression(&self, expression: &str) -> Result<ExpressionAnalysis, ModelError>;

    /// Box a value with metadata
    async fn box_value_with_metadata(
        &self,
        value: &dyn ValueReflection,
        type_name: &str,
    ) -> Result<BoxedValueWithMetadata, ModelError>;

    /// Extract primitive extensions from a value
    async fn extract_primitive_extensions(
        &self,
        value: &dyn ValueReflection,
        element_path: &str,
    ) -> Option<PrimitiveExtensionData>;

    /// Find extensions by URL on a value (handles both direct extensions and primitive extensions)
    async fn find_extensions_by_url(
        &self,
        value: &crate::FhirPathValue,
        parent_resource: &crate::FhirPathValue,
        element_path: Option<&str>,
        url: &str,
    ) -> Vec<crate::FhirPathValue>;

    /// Get search parameters for a resource type
    async fn get_search_params(&self, resource_type: &str) -> Vec<SearchParameter>;

    /// Check if a type is a FHIR resource type
    async fn is_resource_type(&self, type_name: &str) -> bool;

    /// Get the FHIR version supported by this provider
    fn fhir_version(&self) -> FhirVersion;

    // is_subtype_of moved to advanced operations section below

    /// Check if a resource type matches a target type (includes inheritance)
    /// This method performs a comprehensive type compatibility check:
    /// - Direct match: returns true if resource_type == target_type
    /// - Inheritance check: returns true if resource_type is a subtype of target_type
    async fn is_type_compatible(&self, resource_type: &str, target_type: &str) -> bool {
        // Direct match
        if resource_type == target_type {
            return true;
        }

        // Check inheritance hierarchy
        self.is_subtype_of(resource_type, target_type).await
    }

    /// Validate if a resource conforms to a specific StructureDefinition profile
    /// Returns Ok(true) if the resource conforms, Ok(false) if it doesn't,
    /// or Err if the profile cannot be resolved or validation fails
    async fn validates_resource_against_profile(
        &self,
        resource: &crate::FhirPathValue,
        profile_url: &str,
    ) -> Result<bool, ModelError> {
        // Extract the resource type from the resource
        let resource_type = match resource {
            crate::FhirPathValue::Resource(res) => {
                res.resource_type().unwrap_or("Resource").to_string()
            }
            crate::FhirPathValue::JsonValue(json) => json
                .as_inner()
                .get("resourceType")
                .and_then(|rt| rt.as_str())
                .unwrap_or("Resource")
                .to_string(),
            _ => return Ok(false), // Non-resources cannot conform to profiles
        };

        // For base FHIR resources, check if the profile URL matches the resource type
        if profile_url.contains("/StructureDefinition/") {
            let profile_name = profile_url
                .split("/StructureDefinition/")
                .last()
                .unwrap_or("");

            // Check if this is a base FHIR resource profile
            if profile_name == resource_type {
                return Ok(true);
            }
        }

        // For more complex validation, use the validate_conformance method
        // This is a simplified implementation - full validation would require
        // fetching and analyzing the StructureDefinition
        Ok(false)
    }

    /// Get all properties of a type
    async fn get_properties(&self, type_name: &str) -> Vec<(String, TypeReflectionInfo)>;

    /// Get base type of a given type
    async fn get_base_type(&self, type_name: &str) -> Option<String>;

    /// Validate navigation path
    async fn validate_navigation_path(
        &self,
        type_name: &str,
        path: &str,
    ) -> Result<NavigationValidation, ModelError>;

    /// Extract type name from FhirPathValue (handles TypeInfoObject, String, etc.)
    /// This is a shared utility for type operations like is() and ofType()
    fn extract_type_name(&self, type_arg: &crate::FhirPathValue) -> Result<String, ModelError> {
        use crate::FhirPathValue;

        match type_arg {
            FhirPathValue::Empty => Err(ModelError::ConstraintError {
                constraint_key: "type-conversion".to_string(),
                message: "Empty value cannot be used as type argument".to_string(),
            }),
            FhirPathValue::String(type_name) => Ok(type_name.to_string()),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Handle type identifiers like Patient, FHIR.Patient, System.Integer
                if namespace.as_ref() == "System" {
                    Ok(name.as_ref().to_string())
                } else if namespace.as_ref() == "FHIR" {
                    // For FHIR types, use just the name (e.g., "Patient" from "FHIR.Patient")
                    Ok(name.as_ref().to_string())
                } else {
                    // For unqualified types, use the name directly
                    Ok(name.as_ref().to_string())
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    // Recursively extract from single-item collection
                    self.extract_type_name(items.iter().next().unwrap())
                } else if items.is_empty() {
                    Err(ModelError::ConstraintError {
                        constraint_key: "type-conversion".to_string(),
                        message: "Empty collection cannot be used as type argument".to_string(),
                    })
                } else {
                    // Multiple items - try to extract a common type name if they're all strings
                    let first_item = items.iter().next().unwrap();
                    if let FhirPathValue::String(type_name) = first_item {
                        Ok(type_name.to_string())
                    } else {
                        Err(ModelError::ConstraintError {
                            constraint_key: "type-conversion".to_string(),
                            message: format!(
                                "Multi-item collection cannot be used as type argument, got {} items",
                                items.len()
                            ),
                        })
                    }
                }
            }
            FhirPathValue::Resource(resource) => {
                // If a resource is passed as type argument, use its resource type
                if let Some(resource_type) = resource.resource_type() {
                    Ok(resource_type.to_string())
                } else {
                    Ok("Resource".to_string())
                }
            }
            value => {
                // Try to convert to string as fallback
                match value.to_string_value() {
                    Some(s) => Ok(s),
                    None => Err(ModelError::ConstraintError {
                        constraint_key: "type-conversion".to_string(),
                        message: format!(
                            "Type argument must be convertible to string, got {}",
                            value.type_name()
                        ),
                    }),
                }
            }
        }
    }

    /// Check if a property is a choice type (value[x] pattern)
    async fn is_choice_property(&self, type_name: &str, property: &str) -> bool {
        let _ = (type_name, property);
        false
    }

    /// Get all available choice variants for a choice property
    async fn get_choice_variants(&self, type_name: &str, property: &str) -> Vec<ChoiceVariant> {
        let _ = (type_name, property);
        vec![]
    }

    /// Resolve the actual choice property from data
    async fn resolve_choice_property(
        &self,
        type_name: &str,
        property: &str,
        data: &crate::FhirPathValue,
    ) -> Option<String> {
        let _ = (type_name, property, data);
        None
    }

    /// Get the base property name from a choice variant (e.g., "valueQuantity" -> "value")
    async fn get_choice_base_property(
        &self,
        type_name: &str,
        variant_property: &str,
    ) -> Option<String> {
        let _ = (type_name, variant_property);
        None
    }

    // === Advanced Type Operations ===

    /// Check if a type is a subtype of another (inheritance checking)
    async fn is_subtype_of(&self, derived_type: &str, base_type: &str) -> bool {
        // Default implementation - should be overridden by providers
        derived_type == base_type
    }

    /// Get all supertypes of a given type (inheritance chain)
    async fn get_supertypes(&self, type_name: &str) -> Vec<String> {
        // Default implementation returns empty
        let _ = type_name;
        vec![]
    }

    /// Check if a type is abstract (cannot be instantiated)
    async fn is_abstract_type(&self, type_name: &str) -> bool {
        // Default implementation - Resource and Element are abstract
        matches!(
            type_name,
            "Resource" | "DomainResource" | "Element" | "Base"
        )
    }

    /// Attempt to cast a value to a target type
    async fn try_cast_value(
        &self,
        value: &crate::FhirPathValue,
        target_type: &str,
    ) -> Result<Option<crate::FhirPathValue>, ModelError> {
        let source_type = self.get_value_type_name(value);

        // Same type - direct return
        if source_type == target_type {
            return Ok(Some(value.clone()));
        }

        // Check if source type is subtype of target (upcast)
        if self.is_subtype_of(&source_type, target_type).await {
            return Ok(Some(value.clone()));
        }

        // Check if target type is subtype of source (downcast)
        if self.is_subtype_of(target_type, &source_type).await {
            return self.try_downcast_value(value, target_type).await;
        }

        // Try primitive type conversions
        self.try_primitive_conversion(value, target_type).await
    }

    /// Check if a value is of a specific type (for 'is' operator)
    async fn is_value_of_type(&self, value: &crate::FhirPathValue, target_type: &str) -> bool {
        let source_type = self.get_value_type_name(value);

        // Exact match
        if source_type == target_type {
            return true;
        }

        // Check inheritance relationship
        self.is_subtype_of(&source_type, target_type).await
    }

    /// Get the runtime type name of a value
    fn get_value_type_name(&self, value: &crate::FhirPathValue) -> String {
        use crate::FhirPathValue;
        match value {
            FhirPathValue::String(_) => "string".to_string(),
            FhirPathValue::Boolean(_) => "boolean".to_string(),
            FhirPathValue::Integer(_) => "integer".to_string(),
            FhirPathValue::Decimal(_) => "decimal".to_string(),
            FhirPathValue::DateTime(_) => "dateTime".to_string(),
            FhirPathValue::Date(_) => "date".to_string(),
            FhirPathValue::Time(_) => "time".to_string(),
            FhirPathValue::JsonValue(json) => {
                // First check if it's a FHIR resource with resourceType
                if let Some(rt) = json.get_property("resourceType") {
                    if rt.is_string() {
                        return rt
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Element".to_string());
                    }
                }

                // For primitive values wrapped in JsonValue (from FHIR data),
                // return their primitive type name to maintain FHIR context
                let inner = json.as_inner();
                if inner.is_boolean() {
                    "boolean".to_string()
                } else if inner.is_number() {
                    if inner.as_i64().is_some() {
                        "integer".to_string()
                    } else {
                        "decimal".to_string()
                    }
                } else if inner.is_str() {
                    "string".to_string()
                } else if inner.is_array() {
                    "Collection".to_string()
                } else if inner.is_object() {
                    // Object without resourceType - could be a complex FHIR element
                    "Element".to_string()
                } else {
                    "Element".to_string()
                }
            }
            FhirPathValue::Resource(resource) => {
                resource.resource_type().unwrap_or("Resource").to_string()
            }
            FhirPathValue::Collection(_) => "Collection".to_string(),
            FhirPathValue::TypeInfoObject { name, .. } => name.to_string(),
            FhirPathValue::Quantity(_) => "Quantity".to_string(),
            FhirPathValue::Empty => "Empty".to_string(),
        }
    }

    /// Attempt downcast validation (helper method)
    async fn try_downcast_value(
        &self,
        value: &crate::FhirPathValue,
        target_type: &str,
    ) -> Result<Option<crate::FhirPathValue>, ModelError> {
        // For downcast, check if the runtime type is compatible
        let runtime_type = self.get_value_type_name(value);

        if self.is_subtype_of(&runtime_type, target_type).await {
            Ok(Some(value.clone()))
        } else {
            Ok(None) // Downcast failed
        }
    }

    /// Attempt primitive type conversion (helper method)
    async fn try_primitive_conversion(
        &self,
        value: &crate::FhirPathValue,
        target_type: &str,
    ) -> Result<Option<crate::FhirPathValue>, ModelError> {
        use crate::FhirPathValue;

        match (value, target_type) {
            // String conversions
            (FhirPathValue::String(s), "integer") => {
                if let Ok(i) = s.parse::<i64>() {
                    Ok(Some(FhirPathValue::Integer(i)))
                } else {
                    Ok(None)
                }
            }
            (FhirPathValue::String(s), "decimal") => {
                if let Ok(d) = s.parse::<f64>() {
                    use rust_decimal::Decimal;
                    if let Ok(decimal) = Decimal::try_from(d) {
                        Ok(Some(FhirPathValue::Decimal(decimal)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            (FhirPathValue::String(s), "boolean") => match s.to_lowercase().as_str() {
                "true" => Ok(Some(FhirPathValue::Boolean(true))),
                "false" => Ok(Some(FhirPathValue::Boolean(false))),
                _ => Ok(None),
            },

            // Numeric conversions
            (FhirPathValue::Integer(i), "string") => {
                Ok(Some(FhirPathValue::String(i.to_string().into())))
            }
            (FhirPathValue::Integer(i), "decimal") => {
                use rust_decimal::Decimal;
                let decimal = Decimal::from(*i);
                Ok(Some(FhirPathValue::Decimal(decimal)))
            }
            (FhirPathValue::Decimal(d), "string") => {
                Ok(Some(FhirPathValue::String(d.to_string().into())))
            }
            (FhirPathValue::Decimal(d), "integer") => {
                if d.fract() == rust_decimal::Decimal::ZERO {
                    if let Some(int_val) = d.to_i64() {
                        Ok(Some(FhirPathValue::Integer(int_val)))
                    } else {
                        Ok(None) // Out of range
                    }
                } else {
                    Ok(None) // Cannot convert non-integer decimal to integer
                }
            }

            // Boolean conversions
            (FhirPathValue::Boolean(b), "string") => {
                Ok(Some(FhirPathValue::String(b.to_string().into())))
            }

            _ => Ok(None), // No conversion available
        }
    }
}

// Legacy compatibility - map old TypeInfo to new TypeReflectionInfo
use super::types::TypeInfo;

/// Compatibility adapter for old ModelProvider interface
pub trait LegacyModelProvider {
    /// Convert old TypeInfo to new TypeReflectionInfo
    fn get_type_info_legacy(&self, type_name: &str) -> Option<TypeInfo>;

    /// Convert old property type lookup to new interface
    fn get_property_type_legacy(&self, parent_type: &str, property: &str) -> Option<TypeInfo>;
}

/// Adapter to convert new ModelProvider to legacy interface
pub struct ModelProviderAdapter<T: ModelProvider> {
    provider: T,
}

impl<T: ModelProvider> ModelProviderAdapter<T> {
    /// Create a new adapter
    pub fn new(provider: T) -> Self {
        Self { provider }
    }

    /// Get the inner provider
    pub fn inner(&self) -> &T {
        &self.provider
    }

    /// Get the inner provider mutably
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.provider
    }
}

impl<T: ModelProvider> LegacyModelProvider for ModelProviderAdapter<T> {
    fn get_type_info_legacy(&self, _type_name: &str) -> Option<TypeInfo> {
        // Legacy adapter is deprecated - async methods should be used instead
        // This is a placeholder to maintain compatibility
        None
    }

    fn get_property_type_legacy(&self, _parent_type: &str, _property: &str) -> Option<TypeInfo> {
        // Legacy adapter is deprecated - async methods should be used instead
        // This is a placeholder to maintain compatibility
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_adapter() {
        use crate::mock_provider::MockModelProvider;

        let mock_provider = MockModelProvider::empty();
        let adapter = ModelProviderAdapter::new(mock_provider);

        // Test legacy interface
        let type_info = adapter.get_type_info_legacy("Patient");
        assert!(type_info.is_none()); // EmptyProvider returns None

        let property_type = adapter.get_property_type_legacy("Patient", "name");
        assert!(property_type.is_none()); // EmptyProvider returns None
    }

    #[tokio::test]
    async fn test_enhanced_provider_methods() {
        use crate::mock_provider::MockModelProvider;

        let provider = MockModelProvider::empty();

        // Test that enhanced methods are available
        let analysis = provider.analyze_expression("Patient.name").await.unwrap();
        assert!(analysis.referenced_types.is_empty());

        let validation = provider
            .validate_navigation_path("Patient", "name")
            .await
            .unwrap();
        assert!(!validation.is_valid);
    }
}

// Note: FhirVersion is imported from octofhir_fhir_model

/// Configuration for FHIRSchema-based model provider
#[derive(Debug, Clone)]
pub struct FhirSchemaConfig {
    /// FHIR version to use
    pub fhir_version: FhirVersion,
    /// Whether to automatically install core FHIR package
    pub auto_install_core: bool,
    /// Core package specification (if different from default)  
    pub core_package_spec: Option<PackageSpec>,
    /// Additional packages to install
    pub additional_packages: Vec<PackageSpec>,
    /// Installation options
    pub install_options: Option<octofhir_fhirschema::InstallOptions>,
    /// Multi-tier cache configuration (now default)
    pub cache_config: super::cache::CacheConfig,
    /// Legacy cache configuration (optional, for backward compatibility)
    pub legacy_cache_config: Option<super::legacy_cache::CacheConfig>,
}

impl Default for FhirSchemaConfig {
    fn default() -> Self {
        Self {
            fhir_version: FhirVersion::R4,
            auto_install_core: true,
            core_package_spec: None,
            additional_packages: vec![],
            install_options: None,
            cache_config: super::cache::CacheConfig::default(),
            legacy_cache_config: None, // Legacy cache is now opt-in
        }
    }
}
