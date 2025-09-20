use std::ops::Range;
use std::sync::Arc;

use crate::ast::analysis::AnalysisMetadata;
use crate::core::{FP0151, FhirPathError, ModelProvider, SourceLocation};
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use octofhir_fhir_model::TypeInfo;

/// Result type for hierarchy analysis operations
pub type AnalysisResult = Result<AnalysisMetadata, FhirPathError>;

/// Type hierarchy analyzer for validating type inheritance using ModelProvider
pub struct HierarchyAnalyzer {
    model_provider: Arc<dyn ModelProvider>,
}

impl HierarchyAnalyzer {
    /// Create a new HierarchyAnalyzer with the given ModelProvider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    /// Validate type inheritance using ModelProvider::is_type_derived_from()
    /// This leverages the ModelProvider's knowledge of the FHIR type hierarchy
    pub fn validate_type_inheritance(
        &self,
        derived_type: &str,
        base_type: &str,
        operation: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Use ModelProvider::is_type_derived_from()
        if self
            .model_provider
            .is_type_derived_from(derived_type, base_type)
        {
            // Valid inheritance - set appropriate result type
            metadata.type_info = Some(TypeInfo {
                type_name: derived_type.to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some(derived_type.to_string()),
            });

            Ok(metadata)
        } else {
            self.create_inheritance_error(derived_type, base_type, operation, location)
        }
    }

    /// Validate reverse inheritance (checking if base type can be assigned from derived type)
    pub fn validate_type_assignment(
        &self,
        target_type: &str,
        source_type: &str,
        operation: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Check both directions for type compatibility
        let is_compatible = target_type == source_type
            || self
                .model_provider
                .is_type_derived_from(source_type, target_type)
            || self
                .model_provider
                .is_type_derived_from(target_type, source_type);

        if is_compatible {
            // Valid assignment - use target type
            metadata.type_info = Some(TypeInfo {
                type_name: target_type.to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some(target_type.to_string()),
            });

            Ok(metadata)
        } else {
            self.create_assignment_error(target_type, source_type, operation, location)
        }
    }

    /// Validate polymorphic type access (e.g., Resource.ofType(Patient))
    pub fn validate_polymorphic_access(
        &self,
        base_type: &str,
        target_type: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Check if target type is derived from base type
        if self
            .model_provider
            .is_type_derived_from(target_type, base_type)
        {
            // Valid polymorphic access
            metadata.type_info = Some(TypeInfo {
                type_name: target_type.to_string(),
                singleton: Some(false), // ofType returns collection
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some(target_type.to_string()),
            });

            Ok(metadata)
        } else {
            self.create_polymorphic_error(base_type, target_type, location)
        }
    }

    /// Check if a type is a known FHIR resource type
    pub async fn validate_resource_type(
        &self,
        type_name: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Use ModelProvider to check if type exists
        match self.model_provider.get_type(type_name).await {
            Ok(Some(type_info)) => {
                // Type exists - check if it's a resource
                if self.is_resource_type(&type_info) {
                    metadata.type_info = Some(type_info);
                    Ok(metadata)
                } else {
                    self.create_not_resource_error(type_name, location)
                }
            }
            Ok(None) => {
                // Type doesn't exist
                self.create_unknown_type_error(type_name, location)
            }
            Err(e) => Err(FhirPathError::model_error(
                FP0151,
                format!("ModelProvider error: {e}"),
            )),
        }
    }

    /// Check if a TypeInfo represents a FHIR resource
    fn is_resource_type(&self, type_info: &TypeInfo) -> bool {
        // Check if this type derives from Resource
        self.model_provider
            .is_type_derived_from(&type_info.type_name, "Resource")
            || type_info.type_name == "Resource"
    }

    /// Create inheritance error diagnostic
    fn create_inheritance_error(
        &self,
        derived_type: &str,
        base_type: &str,
        operation: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "FP0504".to_string(),
                namespace: None,
            },
            message: format!(
                "Type inheritance violation in '{operation}' operation\n  = help: Type '{derived_type}' is not derived from '{base_type}'\n  = note: Check the FHIR specification for valid type hierarchies"
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        };

        metadata.add_diagnostic(diagnostic);
        Ok(metadata)
    }

    /// Create type assignment error diagnostic
    fn create_assignment_error(
        &self,
        target_type: &str,
        source_type: &str,
        operation: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "FP0505".to_string(),
                namespace: None,
            },
            message: format!(
                "Type assignment incompatibility in '{operation}' operation\n  = help: Cannot assign '{source_type}' to '{target_type}'\n  = note: Types must be the same or have an inheritance relationship"
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        };

        metadata.add_diagnostic(diagnostic);
        Ok(metadata)
    }

    /// Create polymorphic access error diagnostic
    fn create_polymorphic_error(
        &self,
        base_type: &str,
        target_type: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "FP0506".to_string(),
                namespace: None,
            },
            message: format!(
                "Invalid polymorphic type access\n  = help: Type '{target_type}' is not derived from '{base_type}'\n  = note: Use ofType() only with types that inherit from the base type"
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        };

        metadata.add_diagnostic(diagnostic);
        Ok(metadata)
    }

    /// Create unknown type error diagnostic
    fn create_unknown_type_error(&self, type_name: &str, location: Range<usize>) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "FP0507".to_string(),
                namespace: None,
            },
            message: format!(
                "Unknown type '{type_name}'\n  = help: Check the spelling and ensure the type exists in the FHIR specification\n  = note: Valid FHIR types include Patient, Observation, Practitioner, etc."
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        };

        metadata.add_diagnostic(diagnostic);
        Ok(metadata)
    }

    /// Create not resource error diagnostic
    fn create_not_resource_error(&self, type_name: &str, location: Range<usize>) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: DiagnosticCode {
                code: "FP0508".to_string(),
                namespace: None,
            },
            message: format!(
                "Type '{type_name}' is not a FHIR resource\n  = help: This type exists but is not a resource (it may be a data type or element)\n  = note: Resource types include Patient, Observation, Practitioner, etc."
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        };

        metadata.add_diagnostic(diagnostic);
        Ok(metadata)
    }

    /// Get common ancestors of two types (useful for type resolution in complex expressions)
    pub fn get_common_ancestor_types(&self, type1: &str, type2: &str) -> Vec<String> {
        let mut common_ancestors = Vec::new();

        // Common FHIR type hierarchies
        let common_base_types = ["Resource", "DomainResource", "Element", "Base"];

        for base_type in &common_base_types {
            if self.model_provider.is_type_derived_from(type1, base_type)
                && self.model_provider.is_type_derived_from(type2, base_type)
            {
                common_ancestors.push(base_type.to_string());
            }
        }

        common_ancestors
    }

    /// Check if a type conversion is safe (no data loss)
    pub fn is_safe_type_conversion(&self, from_type: &str, to_type: &str) -> bool {
        // Safe conversions:
        // 1. Same type
        if from_type == to_type {
            return true;
        }

        // 2. Derived to base type (upcast)
        if self.model_provider.is_type_derived_from(from_type, to_type) {
            return true;
        }

        // 3. Common data type conversions
        self.is_compatible_data_type_conversion(from_type, to_type)
    }

    /// Check if data type conversion is compatible
    fn is_compatible_data_type_conversion(&self, from_type: &str, to_type: &str) -> bool {
        match (
            from_type.to_lower_case().as_str(),
            to_type.to_lower_case().as_str(),
        ) {
            // String conversions
            ("string", "string") => true,
            ("id", "string") => true,
            ("code", "string") => true,
            ("uri", "string") => true,
            ("url", "string") => true,
            ("canonical", "string") => true,
            ("oid", "string") => true,
            ("uuid", "string") => true,

            // Numeric conversions
            ("integer", "decimal") => true,
            ("positiveint", "integer") => true,
            ("unsignedint", "integer") => true,

            // Date/time conversions
            ("date", "datetime") => true,
            ("datetime", "instant") => true,

            _ => false,
        }
    }
}

trait StringExt {
    fn to_lower_case(&self) -> String;
}

impl StringExt for str {
    fn to_lower_case(&self) -> String {
        self.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    fn create_test_analyzer() -> HierarchyAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        HierarchyAnalyzer::new(provider)
    }

    fn create_test_location(offset: usize, length: usize) -> Range<usize> {
        offset..offset + length
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert_eq!(
            std::mem::size_of_val(&analyzer),
            std::mem::size_of::<HierarchyAnalyzer>()
        );
    }

    #[test]
    fn test_safe_type_conversion_same_type() {
        let analyzer = create_test_analyzer();
        assert!(analyzer.is_safe_type_conversion("Patient", "Patient"));
        assert!(analyzer.is_safe_type_conversion("string", "string"));
    }

    #[test]
    fn test_compatible_data_type_conversions() {
        let analyzer = create_test_analyzer();

        // String family conversions
        assert!(analyzer.is_compatible_data_type_conversion("id", "string"));
        assert!(analyzer.is_compatible_data_type_conversion("code", "string"));
        assert!(analyzer.is_compatible_data_type_conversion("uri", "string"));

        // Numeric conversions
        assert!(analyzer.is_compatible_data_type_conversion("integer", "decimal"));
        assert!(analyzer.is_compatible_data_type_conversion("positiveint", "integer"));

        // Date conversions
        assert!(analyzer.is_compatible_data_type_conversion("date", "datetime"));
        assert!(analyzer.is_compatible_data_type_conversion("datetime", "instant"));

        // Invalid conversions
        assert!(!analyzer.is_compatible_data_type_conversion("string", "integer"));
        assert!(!analyzer.is_compatible_data_type_conversion("boolean", "decimal"));
    }

    #[test]
    fn test_get_common_ancestor_types() {
        let analyzer = create_test_analyzer();

        // For EmptyModelProvider, this will return empty since is_type_derived_from returns false
        let ancestors = analyzer.get_common_ancestor_types("Patient", "Observation");
        // With EmptyModelProvider, we expect no common ancestors found
        assert!(ancestors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_type_inheritance() {
        let analyzer = create_test_analyzer();
        let location = create_test_location(0, 7); // "Patient"

        // For EmptyModelProvider, is_type_derived_from always returns false except for same type
        let result = analyzer.validate_type_inheritance("Patient", "Resource", "ofType", location);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // Should have error since EmptyModelProvider doesn't support inheritance
        assert!(!metadata.diagnostics.is_empty());
        assert!(metadata.diagnostics.iter().any(|d| d.code.code == "FP0504"));
    }

    #[tokio::test]
    async fn test_validate_type_assignment_same_type() {
        let analyzer = create_test_analyzer();
        let location = create_test_location(0, 7); // "Patient"

        let result = analyzer.validate_type_assignment("Patient", "Patient", "as", location);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // Should not have errors for same type
        let errors: Vec<_> = metadata
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(errors.is_empty());

        // Should have correct type info
        assert!(metadata.type_info.is_some());
        let type_info = metadata.type_info.unwrap();
        assert_eq!(type_info.type_name, "Patient");
    }

    #[tokio::test]
    async fn test_validate_polymorphic_access() {
        let analyzer = create_test_analyzer();
        let location = create_test_location(15, 7); // "Patient" in "Resource.ofType(Patient)"

        let result = analyzer.validate_polymorphic_access("Resource", "Patient", location);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // With EmptyModelProvider, should have error since inheritance isn't supported
        assert!(!metadata.diagnostics.is_empty());
        assert!(metadata.diagnostics.iter().any(|d| d.code.code == "FP0506"));
    }

    #[tokio::test]
    async fn test_validate_resource_type_unknown() {
        let analyzer = create_test_analyzer();
        let location = create_test_location(0, 11); // "UnknownType"

        let result = analyzer
            .validate_resource_type("UnknownType", location)
            .await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // Should have error for unknown type
        assert!(!metadata.diagnostics.is_empty());
        assert!(metadata.diagnostics.iter().any(|d| d.code.code == "FP0507"));
    }
}
