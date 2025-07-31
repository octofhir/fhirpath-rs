//! CDA (Clinical Document Architecture) extension for FHIRPath

use crate::registry::extension::{
    ExtensionMetadata, ExtensionRegistry, ExtensionResult, FhirPathExtension,
};
use crate::registry::function::FunctionImpl;
use crate::registry::functions::cda::HasTemplateIdOfFunction;
use std::sync::Arc;

/// CDA extension providing CDA-specific functions
#[derive(Debug)]
pub struct CdaExtension {
    metadata: ExtensionMetadata,
}

impl CdaExtension {
    /// Create a new CDA extension
    pub fn new() -> Self {
        let mut metadata = ExtensionMetadata::new(
            "cda",
            "Clinical Document Architecture Extension",
            "1.0.0",
            "Provides CDA-specific functions for FHIRPath expressions",
            "FHIRPath Registry Team",
        );

        // Set core version compatibility
        metadata.set_core_version_range(
            Some("0.2.0".to_string()),
            None, // No upper limit
        );

        // Set additional metadata
        metadata.license = Some("Apache-2.0".to_string());
        metadata.documentation = Some("https://docs.fhirpath.org/extensions/cda".to_string());

        Self { metadata }
    }
}

impl Default for CdaExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl FhirPathExtension for CdaExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        &self.metadata
    }

    fn register_functions(&self, registry: &mut ExtensionRegistry) -> ExtensionResult<()> {
        // Register hasTemplateIdOf function
        let has_template_id_of = FunctionImpl::Trait(Arc::new(HasTemplateIdOfFunction));
        registry.register_function("cda", "hasTemplateIdOf", has_template_id_of)?;

        Ok(())
    }

    fn initialize(&self) -> ExtensionResult<()> {
        // CDA extension requires no special initialization
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::extension::manager::ExtensionManager;
    use crate::registry::function::FunctionRegistry;

    #[test]
    fn test_cda_extension_creation() {
        let extension = CdaExtension::new();
        let metadata = extension.metadata();

        assert_eq!(metadata.namespace, "cda");
        assert_eq!(metadata.name, "Clinical Document Architecture Extension");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "FHIRPath Registry Team");
    }

    #[test]
    fn test_cda_extension_loading() {
        let core_registry = FunctionRegistry::new();
        let mut manager = ExtensionManager::new(core_registry);

        let extension = Box::new(CdaExtension::new());
        assert!(manager.load_extension(extension).is_ok());

        assert!(manager.is_extension_loaded("cda"));

        // Check that the function is available
        let resolution = manager.resolve_function("cda:hasTemplateIdOf");
        assert!(resolution.is_found());

        // Check unqualified resolution
        let resolution = manager.resolve_function("hasTemplateIdOf");
        assert!(resolution.is_found());
    }

    #[test]
    fn test_cda_metadata_validation() {
        let extension = CdaExtension::new();
        let metadata = extension.metadata();

        assert!(metadata.validate_namespace().is_ok());
        assert!(metadata.functions.is_empty()); // Functions are added during registration
        assert!(metadata.variables.is_empty());
    }
}
