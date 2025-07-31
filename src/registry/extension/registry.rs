//! Extension registry implementation

use super::{ExtensionError, ExtensionMetadata, ExtensionResult, VariableResolver};
use crate::registry::function::FunctionImpl;
use rustc_hash::FxHashMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Registry for extension functions and variables
#[derive(Clone)]
pub struct ExtensionRegistry {
    /// Extension functions by qualified name (namespace:function)
    functions: FxHashMap<String, Arc<FunctionImpl>>,

    /// Extension variables by qualified name (namespace:variable)
    variables: FxHashMap<String, VariableResolver>,

    /// Extension metadata by namespace
    metadata: HashMap<String, ExtensionMetadata>,

    /// Active namespaces
    namespaces: HashSet<String>,

    /// Function names by namespace for fast lookup
    functions_by_namespace: HashMap<String, HashSet<String>>,

    /// Variable names by namespace for fast lookup
    variables_by_namespace: HashMap<String, HashSet<String>>,
}

impl ExtensionRegistry {
    /// Create a new extension registry
    pub fn new() -> Self {
        Self {
            functions: FxHashMap::default(),
            variables: FxHashMap::default(),
            metadata: HashMap::new(),
            namespaces: HashSet::new(),
            functions_by_namespace: HashMap::new(),
            variables_by_namespace: HashMap::new(),
        }
    }

    /// Register extension metadata
    pub fn register_metadata(&mut self, metadata: ExtensionMetadata) -> ExtensionResult<()> {
        // Validate namespace
        metadata
            .validate_namespace()
            .map_err(|reason| ExtensionError::invalid_namespace(&metadata.namespace, reason))?;

        // Check for namespace conflicts
        if self.namespaces.contains(&metadata.namespace) {
            return Err(ExtensionError::already_registered(&metadata.namespace));
        }

        // Register the namespace and metadata
        self.namespaces.insert(metadata.namespace.clone());
        self.functions_by_namespace
            .insert(metadata.namespace.clone(), HashSet::new());
        self.variables_by_namespace
            .insert(metadata.namespace.clone(), HashSet::new());
        self.metadata.insert(metadata.namespace.clone(), metadata);

        Ok(())
    }

    /// Register a function for an extension
    pub fn register_function(
        &mut self,
        namespace: &str,
        function_name: &str,
        function: FunctionImpl,
    ) -> ExtensionResult<()> {
        // Check if namespace is registered
        if !self.namespaces.contains(namespace) {
            return Err(ExtensionError::not_found(namespace));
        }

        let qualified_name = format!("{namespace}:{function_name}");

        // Check for function conflicts
        if self.functions.contains_key(&qualified_name) {
            return Err(ExtensionError::function_conflict(namespace, function_name));
        }

        // Register the function
        self.functions.insert(qualified_name, Arc::new(function));
        self.functions_by_namespace
            .get_mut(namespace)
            .unwrap()
            .insert(function_name.to_string());

        // Update metadata
        if let Some(metadata) = self.metadata.get_mut(namespace) {
            metadata.add_function(function_name);
        }

        Ok(())
    }

    /// Register a variable resolver for an extension
    pub fn register_variable(
        &mut self,
        namespace: &str,
        variable_name: &str,
        resolver: VariableResolver,
    ) -> ExtensionResult<()> {
        // Check if namespace is registered
        if !self.namespaces.contains(namespace) {
            return Err(ExtensionError::not_found(namespace));
        }

        let qualified_name = format!("{namespace}:{variable_name}");

        // Check for variable conflicts
        if self.variables.contains_key(&qualified_name) {
            return Err(ExtensionError::variable_conflict(namespace, variable_name));
        }

        // Register the variable
        self.variables.insert(qualified_name, resolver);
        self.variables_by_namespace
            .get_mut(namespace)
            .unwrap()
            .insert(variable_name.to_string());

        // Update metadata
        if let Some(metadata) = self.metadata.get_mut(namespace) {
            metadata.add_variable(variable_name);
        }

        Ok(())
    }

    /// Get a function by qualified name (namespace:function)
    pub fn get_function(&self, qualified_name: &str) -> Option<&Arc<FunctionImpl>> {
        self.functions.get(qualified_name)
    }

    /// Get a variable resolver by qualified name (namespace:variable)
    pub fn get_variable(&self, qualified_name: &str) -> Option<&VariableResolver> {
        self.variables.get(qualified_name)
    }

    /// Get extension metadata by namespace
    pub fn get_metadata(&self, namespace: &str) -> Option<&ExtensionMetadata> {
        self.metadata.get(namespace)
    }

    /// Check if a namespace is registered
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespaces.contains(namespace)
    }

    /// Get all registered namespaces
    pub fn namespaces(&self) -> impl Iterator<Item = &String> {
        self.namespaces.iter()
    }

    /// Get all functions in a namespace
    pub fn functions_in_namespace(&self, namespace: &str) -> Option<&HashSet<String>> {
        self.functions_by_namespace.get(namespace)
    }

    /// Get all variables in a namespace
    pub fn variables_in_namespace(&self, namespace: &str) -> Option<&HashSet<String>> {
        self.variables_by_namespace.get(namespace)
    }

    /// Find all namespaces that have a function with the given name
    pub fn find_function_namespaces(&self, function_name: &str) -> Vec<String> {
        let mut namespaces = Vec::new();

        for (namespace, functions) in &self.functions_by_namespace {
            if functions.contains(function_name) {
                namespaces.push(namespace.clone());
            }
        }

        namespaces
    }

    /// Find all namespaces that have a variable with the given name
    pub fn find_variable_namespaces(&self, variable_name: &str) -> Vec<String> {
        let mut namespaces = Vec::new();

        for (namespace, variables) in &self.variables_by_namespace {
            if variables.contains(variable_name) {
                namespaces.push(namespace.clone());
            }
        }

        namespaces
    }

    /// Get all extension metadata
    pub fn all_metadata(&self) -> impl Iterator<Item = &ExtensionMetadata> {
        self.metadata.values()
    }

    /// Remove an extension and all its functions/variables
    pub fn unregister_extension(&mut self, namespace: &str) -> ExtensionResult<()> {
        if !self.namespaces.contains(namespace) {
            return Err(ExtensionError::not_found(namespace));
        }

        // Remove all functions for this namespace
        if let Some(function_names) = self.functions_by_namespace.get(namespace) {
            for function_name in function_names {
                let qualified_name = format!("{namespace}:{function_name}");
                self.functions.remove(&qualified_name);
            }
        }

        // Remove all variables for this namespace
        if let Some(variable_names) = self.variables_by_namespace.get(namespace) {
            for variable_name in variable_names {
                let qualified_name = format!("{namespace}:{variable_name}");
                self.variables.remove(&qualified_name);
            }
        }

        // Remove namespace data
        self.namespaces.remove(namespace);
        self.functions_by_namespace.remove(namespace);
        self.variables_by_namespace.remove(namespace);
        self.metadata.remove(namespace);

        Ok(())
    }

    /// Get statistics about the registry
    pub fn stats(&self) -> ExtensionStats {
        ExtensionStats {
            namespace_count: self.namespaces.len(),
            function_count: self.functions.len(),
            variable_count: self.variables.len(),
        }
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the extension registry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionStats {
    /// Number of registered namespaces
    pub namespace_count: usize,

    /// Total number of extension functions
    pub function_count: usize,

    /// Total number of extension variables
    pub variable_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FhirPathValue, TypeInfo};
    use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
    use crate::registry::signature::FunctionSignature;

    // Test function for testing
    #[derive(Debug)]
    struct TestFunction;

    impl FhirPathFunction for TestFunction {
        fn name(&self) -> &str {
            "test"
        }

        fn human_friendly_name(&self) -> &str {
            "Test Function"
        }

        fn signature(&self) -> &FunctionSignature {
            static SIG: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
            SIG.get_or_init(|| FunctionSignature::new("test", vec![], TypeInfo::Any))
        }

        fn evaluate(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            Ok(FhirPathValue::String("test".to_string()))
        }
    }

    #[test]
    fn test_extension_registry_creation() {
        let registry = ExtensionRegistry::new();
        assert_eq!(registry.stats().namespace_count, 0);
        assert_eq!(registry.stats().function_count, 0);
        assert_eq!(registry.stats().variable_count, 0);
    }

    #[test]
    fn test_metadata_registration() {
        let mut registry = ExtensionRegistry::new();
        let metadata = ExtensionMetadata::new(
            "test",
            "Test Extension",
            "1.0.0",
            "Test description",
            "Test Author",
        );

        assert!(registry.register_metadata(metadata).is_ok());
        assert!(registry.has_namespace("test"));
        assert!(registry.get_metadata("test").is_some());
    }

    #[test]
    fn test_function_registration() {
        let mut registry = ExtensionRegistry::new();
        let metadata = ExtensionMetadata::new(
            "test",
            "Test Extension",
            "1.0.0",
            "Test description",
            "Test Author",
        );

        registry.register_metadata(metadata).unwrap();

        let function = FunctionImpl::Trait(Arc::new(TestFunction));
        assert!(
            registry
                .register_function("test", "myFunc", function)
                .is_ok()
        );

        assert!(registry.get_function("test:myFunc").is_some());
        assert_eq!(registry.stats().function_count, 1);
    }

    #[test]
    fn test_namespace_conflicts() {
        let mut registry = ExtensionRegistry::new();
        let metadata1 = ExtensionMetadata::new(
            "test",
            "Test Extension 1",
            "1.0.0",
            "Test description",
            "Test Author",
        );
        let metadata2 = ExtensionMetadata::new(
            "test",
            "Test Extension 2",
            "1.0.0",
            "Test description",
            "Test Author",
        );

        assert!(registry.register_metadata(metadata1).is_ok());
        assert!(registry.register_metadata(metadata2).is_err());
    }

    #[test]
    fn test_function_finding() {
        let mut registry = ExtensionRegistry::new();

        // Register two extensions with the same function name
        let metadata1 = ExtensionMetadata::new("ext1", "Extension 1", "1.0", "", "");
        let metadata2 = ExtensionMetadata::new("ext2", "Extension 2", "1.0", "", "");

        registry.register_metadata(metadata1).unwrap();
        registry.register_metadata(metadata2).unwrap();

        let function1 = FunctionImpl::Trait(Arc::new(TestFunction));
        let function2 = FunctionImpl::Trait(Arc::new(TestFunction));

        registry
            .register_function("ext1", "common", function1)
            .unwrap();
        registry
            .register_function("ext2", "common", function2)
            .unwrap();

        let namespaces = registry.find_function_namespaces("common");
        assert_eq!(namespaces.len(), 2);
        assert!(namespaces.contains(&"ext1".to_string()));
        assert!(namespaces.contains(&"ext2".to_string()));
    }
}
