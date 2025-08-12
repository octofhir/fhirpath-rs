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

//! FHIR extension for FHIRPath

use crate::extension::{ExtensionMetadata, ExtensionRegistry, ExtensionResult, FhirPathExtension};
use crate::function::FunctionImpl;
use crate::functions::fhir_types::{ExtensionFunction, ResolveFunction};
use std::sync::Arc;

/// FHIR extension providing FHIR-specific functions
#[derive(Debug)]
pub struct FhirExtension {
    metadata: ExtensionMetadata,
}

impl FhirExtension {
    /// Create a new FHIR extension
    pub fn new() -> Self {
        let mut metadata = ExtensionMetadata::new(
            "fhir",
            "FHIR Extension",
            "1.0.0",
            "Provides FHIR-specific functions for FHIRPath expressions",
            "FHIRPath Registry Team",
        );

        // Set core version compatibility
        metadata.set_core_version_range(
            Some("0.2.0".to_string()),
            None, // No upper limit
        );

        // Set additional metadata
        metadata.license = Some("Apache-2.0".to_string());
        metadata.documentation = Some("https://docs.fhirpath.org/extensions/fhir".to_string());

        Self { metadata }
    }
}

impl Default for FhirExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl FhirPathExtension for FhirExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        &self.metadata
    }

    fn register_functions(&self, registry: &mut ExtensionRegistry) -> ExtensionResult<()> {
        // Register extension function
        let extension_fn = FunctionImpl::Async(Arc::new(ExtensionFunction));
        registry.register_function("fhir", "extension", extension_fn)?;

        // Register resolve function
        let resolve_fn = FunctionImpl::Async(Arc::new(ResolveFunction));
        registry.register_function("fhir", "resolve", resolve_fn)?;

        Ok(())
    }

    fn initialize(&self) -> ExtensionResult<()> {
        // FHIR extension requires no special initialization
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::manager::ExtensionManager;
    use crate::function::FunctionRegistry;

    #[test]
    fn test_fhir_extension_creation() {
        let extension = FhirExtension::new();
        let metadata = extension.metadata();

        assert_eq!(metadata.namespace, "fhir");
        assert_eq!(metadata.name, "FHIR Extension");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "FHIRPath Registry Team");
    }

    #[test]
    fn test_fhir_extension_loading() {
        let core_registry = FunctionRegistry::new();
        let mut manager = ExtensionManager::new(core_registry);

        let extension = Box::new(FhirExtension::new());
        assert!(manager.load_extension(extension).is_ok());

        assert!(manager.is_extension_loaded("fhir"));

        // Check that functions are available
        let resolution = manager.resolve_function("fhir:extension");
        assert!(resolution.is_found());

        let resolution = manager.resolve_function("fhir:resolve");
        assert!(resolution.is_found());
    }

    #[test]
    fn test_fhir_metadata_validation() {
        let extension = FhirExtension::new();
        let metadata = extension.metadata();

        assert!(metadata.validate_namespace().is_ok());
        assert!(metadata.functions.is_empty()); // Functions are added during registration
        assert!(metadata.variables.is_empty());
    }

    #[test]
    fn test_ambiguous_function_resolution() {
        let core_registry = FunctionRegistry::new();
        let mut manager = ExtensionManager::new(core_registry);

        // Load both CDA and FHIR extensions
        let cda_extension = Box::new(crate::extension::builtin::CdaExtension::new());
        let fhir_extension = Box::new(FhirExtension::new());

        manager.load_extension(cda_extension).unwrap();
        manager.load_extension(fhir_extension).unwrap();

        // Both extensions should be loaded
        assert!(manager.is_extension_loaded("cda"));
        assert!(manager.is_extension_loaded("fhir"));

        // Qualified names should resolve uniquely
        assert!(manager.resolve_function("fhir:extension").is_found());
        assert!(manager.resolve_function("fhir:resolve").is_found());
        assert!(manager.resolve_function("cda:hasTemplateIdOf").is_found());
    }
}
