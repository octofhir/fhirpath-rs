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

// Define our own async-first ModelProvider trait
use async_trait::async_trait;

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

    /// Get search parameters for a resource type
    async fn get_search_params(&self, resource_type: &str) -> Vec<SearchParameter>;

    /// Check if a type is a FHIR resource type
    async fn is_resource_type(&self, type_name: &str) -> bool;

    /// Get the FHIR version supported by this provider
    fn fhir_version(&self) -> FhirVersion;

    /// Check if child_type is a subtype of parent_type
    async fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool;

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
    /// Cache configuration
    pub cache_config: super::cache::CacheConfig,
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
        }
    }
}
