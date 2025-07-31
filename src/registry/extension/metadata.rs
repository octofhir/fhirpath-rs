//! Extension metadata types

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Metadata for a FHIRPath extension
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    /// Extension namespace (e.g., "cda", "fhir")
    pub namespace: String,

    /// Human-readable extension name
    pub name: String,

    /// Extension version (semantic version)
    pub version: String,

    /// Extension description
    pub description: String,

    /// Extension author
    pub author: String,

    /// Minimum compatible core FHIRPath version
    pub min_core_version: Option<String>,

    /// Maximum compatible core FHIRPath version
    pub max_core_version: Option<String>,

    /// Extension dependencies (other extensions required)
    pub dependencies: Vec<ExtensionDependency>,

    /// Functions provided by this extension
    pub functions: HashSet<String>,

    /// Variables provided by this extension
    pub variables: HashSet<String>,

    /// Extension homepage URL
    pub homepage: Option<String>,

    /// Extension documentation URL
    pub documentation: Option<String>,

    /// Extension license
    pub license: Option<String>,
}

/// Extension dependency information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDependency {
    /// Namespace of the required extension
    pub namespace: String,

    /// Minimum version of the required extension
    pub min_version: Option<String>,

    /// Maximum version of the required extension
    pub max_version: Option<String>,

    /// Whether this dependency is optional
    pub optional: bool,
}

impl ExtensionMetadata {
    /// Create a new extension metadata
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            min_core_version: None,
            max_core_version: None,
            dependencies: Vec::new(),
            functions: HashSet::new(),
            variables: HashSet::new(),
            homepage: None,
            documentation: None,
            license: None,
        }
    }

    /// Add a function to the metadata
    pub fn add_function(&mut self, function_name: impl Into<String>) {
        self.functions.insert(function_name.into());
    }

    /// Add a variable to the metadata
    pub fn add_variable(&mut self, variable_name: impl Into<String>) {
        self.variables.insert(variable_name.into());
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dependency: ExtensionDependency) {
        self.dependencies.push(dependency);
    }

    /// Set core version compatibility
    pub fn set_core_version_range(
        &mut self,
        min_version: Option<String>,
        max_version: Option<String>,
    ) {
        self.min_core_version = min_version;
        self.max_core_version = max_version;
    }

    /// Check if the namespace is valid
    pub fn validate_namespace(&self) -> Result<(), String> {
        let namespace = &self.namespace;

        // Namespace must not be empty
        if namespace.is_empty() {
            return Err("Namespace cannot be empty".to_string());
        }

        // Namespace must start with a letter
        if !namespace
            .chars()
            .next()
            .unwrap_or(' ')
            .is_ascii_alphabetic()
        {
            return Err("Namespace must start with a letter".to_string());
        }

        // Namespace can only contain letters, digits, hyphens, and underscores
        if !namespace
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Namespace can only contain letters, digits, hyphens, and underscores".to_string(),
            );
        }

        // Reserved namespaces
        const RESERVED: &[&str] = &["core", "std", "builtin", "system"];
        if RESERVED.contains(&namespace.as_str()) {
            return Err(format!("Namespace '{namespace}' is reserved"));
        }

        Ok(())
    }

    /// Get the full qualified name for a function
    pub fn qualified_function_name(&self, function_name: &str) -> String {
        format!("{}:{}", self.namespace, function_name)
    }

    /// Get the full qualified name for a variable
    pub fn qualified_variable_name(&self, variable_name: &str) -> String {
        format!("{}:{}", self.namespace, variable_name)
    }
}

impl ExtensionDependency {
    /// Create a required extension dependency
    pub fn required(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            min_version: None,
            max_version: None,
            optional: false,
        }
    }

    /// Create an optional extension dependency
    pub fn optional(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            min_version: None,
            max_version: None,
            optional: true,
        }
    }

    /// Set version range for the dependency
    pub fn with_version_range(
        mut self,
        min_version: Option<String>,
        max_version: Option<String>,
    ) -> Self {
        self.min_version = min_version;
        self.max_version = max_version;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_metadata_creation() {
        let metadata = ExtensionMetadata::new(
            "test",
            "Test Extension",
            "1.0.0",
            "A test extension",
            "Test Author",
        );

        assert_eq!(metadata.namespace, "test");
        assert_eq!(metadata.name, "Test Extension");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "A test extension");
        assert_eq!(metadata.author, "Test Author");
        assert!(metadata.functions.is_empty());
        assert!(metadata.variables.is_empty());
    }

    #[test]
    fn test_namespace_validation() {
        let mut metadata = ExtensionMetadata::new("test", "Test", "1.0", "Test", "Author");
        assert!(metadata.validate_namespace().is_ok());

        metadata.namespace = "".to_string();
        assert!(metadata.validate_namespace().is_err());

        metadata.namespace = "123invalid".to_string();
        assert!(metadata.validate_namespace().is_err());

        metadata.namespace = "core".to_string();
        assert!(metadata.validate_namespace().is_err());

        metadata.namespace = "valid-namespace_123".to_string();
        assert!(metadata.validate_namespace().is_ok());
    }

    #[test]
    fn test_qualified_names() {
        let metadata = ExtensionMetadata::new("test", "Test", "1.0", "Test", "Author");

        assert_eq!(metadata.qualified_function_name("func"), "test:func");
        assert_eq!(metadata.qualified_variable_name("var"), "test:var");
    }

    #[test]
    fn test_dependency_creation() {
        let required_dep = ExtensionDependency::required("other");
        assert_eq!(required_dep.namespace, "other");
        assert!(!required_dep.optional);

        let optional_dep = ExtensionDependency::optional("optional")
            .with_version_range(Some("1.0.0".to_string()), Some("2.0.0".to_string()));
        assert_eq!(optional_dep.namespace, "optional");
        assert!(optional_dep.optional);
        assert_eq!(optional_dep.min_version, Some("1.0.0".to_string()));
        assert_eq!(optional_dep.max_version, Some("2.0.0".to_string()));
    }
}
