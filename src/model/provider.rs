//! Model provider trait for FHIR type information
//!
//! This module re-exports the enhanced ModelProvider from octofhir-fhir-model
//! and provides compatibility shims for the old interface.

// Re-export the enhanced ModelProvider and related types
pub use octofhir_fhir_model::provider::{
    BoxedValueWithMetadata, ConstraintViolation, DetailedConformanceResult, ElementDefinition,
    ElementType, EmptyModelProvider, ExpressionAnalysis, FhirPathAnalysisResult, FhirVersion,
    ModelProvider, NavigationContext, NavigationValidation, PolymorphicTypeInfo,
    PrimitiveExtensionData, ProviderMetrics, ResolutionContext, SearchParameter,
    StructureDefinition, ValueReflection, ViolationSeverity,
};

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
    fn get_type_info_legacy(&self, type_name: &str) -> Option<TypeInfo> {
        // Convert TypeReflectionInfo to legacy TypeInfo
        self.provider
            .get_type_reflection(type_name)
            .map(|type_reflection| {
                match type_reflection {
                    TypeReflectionInfo::SimpleType { name, .. } => {
                        // Map common FHIR and system types to the correct legacy enum variant
                        match name.as_str() {
                            "Boolean" => TypeInfo::Boolean,
                            "Integer" => TypeInfo::Integer,
                            "Decimal" => TypeInfo::Decimal,
                            "String" => TypeInfo::String,
                            "Date" => TypeInfo::Date,
                            "DateTime" => TypeInfo::DateTime,
                            "Time" => TypeInfo::Time,
                            "Quantity" => TypeInfo::Quantity,
                            resource_name if name.chars().next().unwrap_or('a').is_uppercase() => {
                                TypeInfo::Resource(resource_name.to_string())
                            }
                            _ => TypeInfo::String, // Fallback to string for unknown types
                        }
                    }
                    TypeReflectionInfo::ClassInfo { name, .. } => {
                        // Class types are typically FHIR resources
                        TypeInfo::Resource(name)
                    }
                    TypeReflectionInfo::ListType { element_type } => {
                        // For legacy compatibility, we simplify list types
                        let inner_type = match *element_type {
                            TypeReflectionInfo::SimpleType { name, .. } => match name.as_str() {
                                "Boolean" => TypeInfo::Boolean,
                                "Integer" => TypeInfo::Integer,
                                "Decimal" => TypeInfo::Decimal,
                                "String" => TypeInfo::String,
                                _ => TypeInfo::String,
                            },
                            _ => TypeInfo::String,
                        };
                        TypeInfo::Collection(Box::new(inner_type))
                    }
                    TypeReflectionInfo::TupleType { .. } => {
                        TypeInfo::Any // Use Any for tuple types
                    }
                }
            })
    }

    fn get_property_type_legacy(&self, parent_type: &str, property: &str) -> Option<TypeInfo> {
        self.provider
            .get_property_type(parent_type, property)
            .map(|type_reflection| {
                // Same conversion logic as above
                match type_reflection {
                    TypeReflectionInfo::SimpleType { name, .. } => {
                        // Map common FHIR and system types to the correct legacy enum variant
                        match name.as_str() {
                            "Boolean" => TypeInfo::Boolean,
                            "Integer" => TypeInfo::Integer,
                            "Decimal" => TypeInfo::Decimal,
                            "String" => TypeInfo::String,
                            "Date" => TypeInfo::Date,
                            "DateTime" => TypeInfo::DateTime,
                            "Time" => TypeInfo::Time,
                            "Quantity" => TypeInfo::Quantity,
                            resource_name if name.chars().next().unwrap_or('a').is_uppercase() => {
                                TypeInfo::Resource(resource_name.to_string())
                            }
                            _ => TypeInfo::String, // Fallback to string for unknown types
                        }
                    }
                    TypeReflectionInfo::ClassInfo { name, .. } => {
                        // Class types are typically FHIR resources
                        TypeInfo::Resource(name)
                    }
                    TypeReflectionInfo::ListType { element_type } => {
                        // For legacy compatibility, we simplify list types
                        let inner_type = match *element_type {
                            TypeReflectionInfo::SimpleType { name, .. } => match name.as_str() {
                                "Boolean" => TypeInfo::Boolean,
                                "Integer" => TypeInfo::Integer,
                                "Decimal" => TypeInfo::Decimal,
                                "String" => TypeInfo::String,
                                _ => TypeInfo::String,
                            },
                            _ => TypeInfo::String,
                        };
                        TypeInfo::Collection(Box::new(inner_type))
                    }
                    TypeReflectionInfo::TupleType { .. } => {
                        TypeInfo::Any // Use Any for tuple types
                    }
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_adapter() {
        let empty_provider = EmptyModelProvider::new();
        let adapter = ModelProviderAdapter::new(empty_provider);

        // Test legacy interface
        let type_info = adapter.get_type_info_legacy("Patient");
        assert!(type_info.is_none()); // EmptyProvider returns None

        let property_type = adapter.get_property_type_legacy("Patient", "name");
        assert!(property_type.is_none()); // EmptyProvider returns None
    }

    #[test]
    fn test_enhanced_provider_methods() {
        let provider = EmptyModelProvider::new();

        // Test that enhanced methods are available
        let analysis = provider.analyze_expression("Patient.name").unwrap();
        assert!(analysis.referenced_types.is_empty());

        let validation = provider
            .validate_navigation_path("Patient", "name")
            .unwrap();
        assert!(!validation.is_valid);
    }
}
