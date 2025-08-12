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

//! Extension manager for loading and managing extensions

use super::{
    ExtensionError, ExtensionRegistry, ExtensionResult, FhirPathExtension, FunctionResolution,
};
use crate::registry::function::FunctionRegistry;
use std::collections::HashMap;
use std::sync::Arc;

/// Manager for FHIRPath extensions
pub struct ExtensionManager {
    /// Core function registry
    core_registry: FunctionRegistry,

    /// Extension registry
    extension_registry: ExtensionRegistry,

    /// Loaded extensions by namespace
    extensions: HashMap<String, Box<dyn FhirPathExtension>>,

    /// Extension loading order for dependency resolution
    load_order: Vec<String>,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new(core_registry: FunctionRegistry) -> Self {
        Self {
            core_registry,
            extension_registry: ExtensionRegistry::new(),
            extensions: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Load an extension
    pub fn load_extension(&mut self, extension: Box<dyn FhirPathExtension>) -> ExtensionResult<()> {
        let metadata = extension.metadata();
        let namespace = metadata.namespace.clone();

        // Validate metadata
        metadata
            .validate_namespace()
            .map_err(|reason| ExtensionError::invalid_namespace(&namespace, reason))?;

        // Check if already loaded
        if self.extensions.contains_key(&namespace) {
            return Err(ExtensionError::already_registered(&namespace));
        }

        // Check dependencies
        self.check_dependencies(&namespace, &metadata.dependencies)?;

        // Register metadata
        self.extension_registry
            .register_metadata(metadata.clone())?;

        // Register functions and variables
        extension
            .register_functions(&mut self.extension_registry)
            .map_err(|e| ExtensionError::initialization_failed(&namespace, e.to_string()))?;

        extension
            .register_variables(&mut self.extension_registry)
            .map_err(|e| ExtensionError::initialization_failed(&namespace, e.to_string()))?;

        // Initialize extension
        extension
            .initialize()
            .map_err(|e| ExtensionError::initialization_failed(&namespace, e.to_string()))?;

        // Store extension and update load order
        self.extensions.insert(namespace.clone(), extension);
        self.load_order.push(namespace);

        Ok(())
    }

    /// Unload an extension
    pub fn unload_extension(&mut self, namespace: &str) -> ExtensionResult<()> {
        // Check if extension is loaded
        if !self.extensions.contains_key(namespace) {
            return Err(ExtensionError::not_found(namespace));
        }

        // Check if other extensions depend on this one
        for (other_namespace, other_extension) in &self.extensions {
            if other_namespace != namespace {
                let other_metadata = other_extension.metadata();
                for dep in &other_metadata.dependencies {
                    if dep.namespace == namespace && !dep.optional {
                        return Err(ExtensionError::DependencyNotFound {
                            namespace: other_namespace.clone(),
                            dependency: namespace.to_string(),
                        });
                    }
                }
            }
        }

        // Cleanup extension
        if let Some(extension) = self.extensions.get(namespace) {
            extension
                .cleanup()
                .map_err(|e| ExtensionError::cleanup_failed(namespace, e.to_string()))?;
        }

        // Remove from registries
        self.extension_registry.unregister_extension(namespace)?;
        self.extensions.remove(namespace);
        self.load_order.retain(|ns| ns != namespace);

        Ok(())
    }

    /// Resolve a function by name with namespace support
    pub fn resolve_function(&self, name: &str) -> FunctionResolution {
        // Check if it's a qualified name (contains ':')
        if let Some(colon_pos) = name.find(':') {
            let namespace = &name[..colon_pos];
            let _function_name = &name[colon_pos + 1..];

            // Look up in extension registry
            if let Some(function) = self.extension_registry.get_function(name) {
                return FunctionResolution::Extension {
                    namespace: namespace.to_string(),
                    function: function.clone(),
                };
            }

            return FunctionResolution::NotFound;
        }

        // Check core registry first
        if let Some(function) = self.core_registry.get(name) {
            return FunctionResolution::Core(Arc::new(function.clone()));
        }

        // Check extension registries for unqualified names
        let matching_namespaces = self.extension_registry.find_function_namespaces(name);

        match matching_namespaces.len() {
            0 => FunctionResolution::NotFound,
            1 => {
                let namespace = &matching_namespaces[0];
                let qualified_name = format!("{namespace}:{name}");
                if let Some(function) = self.extension_registry.get_function(&qualified_name) {
                    FunctionResolution::Extension {
                        namespace: namespace.clone(),
                        function: function.clone(),
                    }
                } else {
                    FunctionResolution::NotFound
                }
            }
            _ => FunctionResolution::Ambiguous(matching_namespaces),
        }
    }

    /// Get extension metadata by namespace
    pub fn get_extension_metadata(&self, namespace: &str) -> Option<&super::ExtensionMetadata> {
        self.extension_registry.get_metadata(namespace)
    }

    /// List all loaded extensions
    pub fn list_extensions(&self) -> Vec<&super::ExtensionMetadata> {
        self.extension_registry.all_metadata().collect()
    }

    /// Check if an extension is loaded
    pub fn is_extension_loaded(&self, namespace: &str) -> bool {
        self.extensions.contains_key(namespace)
    }

    /// Get the core function registry
    pub fn core_registry(&self) -> &FunctionRegistry {
        &self.core_registry
    }

    /// Get a mutable reference to the core function registry
    pub fn core_registry_mut(&mut self) -> &mut FunctionRegistry {
        &mut self.core_registry
    }

    /// Get the extension registry
    pub fn extension_registry(&self) -> &ExtensionRegistry {
        &self.extension_registry
    }

    /// Get extension load order
    pub fn load_order(&self) -> &[String] {
        &self.load_order
    }

    /// Check extension dependencies
    fn check_dependencies(
        &self,
        namespace: &str,
        dependencies: &[super::metadata::ExtensionDependency],
    ) -> ExtensionResult<()> {
        for dep in dependencies {
            if !dep.optional && !self.is_extension_loaded(&dep.namespace) {
                return Err(ExtensionError::DependencyNotFound {
                    namespace: namespace.to_string(),
                    dependency: dep.namespace.clone(),
                });
            }

            // TODO: Check version compatibility when we have version parsing
        }

        Ok(())
    }
}

impl Clone for ExtensionManager {
    fn clone(&self) -> Self {
        // Note: We can't clone the extensions themselves since they're trait objects
        // This clone only preserves the core registry and extension registry state
        Self {
            core_registry: self.core_registry.clone(),
            extension_registry: self.extension_registry.clone(),
            extensions: HashMap::new(), // Extensions are not cloneable
            load_order: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FhirPathValue, TypeInfo};
    use crate::registry::extension::metadata::ExtensionMetadata;
    use crate::registry::function::{
        EvaluationContext, FhirPathFunction, FunctionImpl, FunctionResult,
    };
    use crate::registry::signature::FunctionSignature;

    // Test extension
    struct TestExtension {
        metadata: ExtensionMetadata,
    }

    impl TestExtension {
        fn new() -> Self {
            Self {
                metadata: ExtensionMetadata::new(
                    "test",
                    "Test Extension",
                    "1.0.0",
                    "A test extension",
                    "Test Author",
                ),
            }
        }
    }

    impl FhirPathExtension for TestExtension {
        fn metadata(&self) -> &ExtensionMetadata {
            &self.metadata
        }

        fn register_functions(&self, registry: &mut ExtensionRegistry) -> ExtensionResult<()> {
            // Register a simple test function
            registry.register_function("test", "hello", TestFunction::new())?;
            Ok(())
        }
    }

    // Test function
    #[derive(Debug, Clone)]
    struct TestFunction;

    impl TestFunction {
        fn new() -> FunctionImpl {
            FunctionImpl::Trait(Arc::new(Self))
        }
    }

    impl FhirPathFunction for TestFunction {
        fn name(&self) -> &str {
            "hello"
        }

        fn human_friendly_name(&self) -> &str {
            "Hello Function"
        }

        fn signature(&self) -> &FunctionSignature {
            static SIG: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
            SIG.get_or_init(|| FunctionSignature::new("hello", vec![], TypeInfo::String))
        }

        fn evaluate(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            Ok(FhirPathValue::String("Hello, World!".into()))
        }
    }

    #[test]
    fn test_extension_manager_creation() {
        let core_registry = FunctionRegistry::new();
        let manager = ExtensionManager::new(core_registry);

        assert_eq!(manager.list_extensions().len(), 0);
        assert!(!manager.is_extension_loaded("test"));
    }

    #[test]
    fn test_extension_loading() {
        let core_registry = FunctionRegistry::new();
        let mut manager = ExtensionManager::new(core_registry);

        let extension = Box::new(TestExtension::new());
        assert!(manager.load_extension(extension).is_ok());

        assert!(manager.is_extension_loaded("test"));
        assert_eq!(manager.list_extensions().len(), 1);
    }

    #[test]
    fn test_function_resolution() {
        let core_registry = FunctionRegistry::new();
        let mut manager = ExtensionManager::new(core_registry);

        let extension = Box::new(TestExtension::new());
        manager.load_extension(extension).unwrap();

        // Test qualified name resolution
        let resolution = manager.resolve_function("test:hello");
        assert!(resolution.is_found());

        // Test unqualified name resolution
        let resolution = manager.resolve_function("hello");
        assert!(resolution.is_found());

        // Test non-existent function
        let resolution = manager.resolve_function("nonexistent");
        assert!(!resolution.is_found());
    }

    #[test]
    fn test_extension_unloading() {
        let core_registry = FunctionRegistry::new();
        let mut manager = ExtensionManager::new(core_registry);

        let extension = Box::new(TestExtension::new());
        manager.load_extension(extension).unwrap();

        assert!(manager.is_extension_loaded("test"));
        assert!(manager.unload_extension("test").is_ok());
        assert!(!manager.is_extension_loaded("test"));
    }
}
