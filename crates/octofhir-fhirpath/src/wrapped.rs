//! Wrapped values with rich FHIR metadata for enhanced type safety and debugging

use std::fmt;

use crate::core::{Collection, FhirPathValue, Result};
use crate::path::CanonicalPath;
use crate::typing::{TypeResolver, type_utils};

/// Metadata associated with a FHIRPath value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueMetadata {
    /// FHIR type name (resolved from ModelProvider)
    /// Examples: "string", "HumanName", "Patient", "boolean"
    pub fhir_type: String,

    /// Resource type if this value represents a FHIR resource root
    /// Examples: Some("Patient"), Some("Observation"), None for primitives
    pub resource_type: Option<String>,

    /// Canonical path representation
    pub path: CanonicalPath,

    /// Array index if this value is from an array access
    /// Used for generating indexed paths like "name[0]", "given[1]"
    pub index: Option<usize>,
}

impl ValueMetadata {
    /// Create metadata for a resource root (Patient, Observation, etc.)
    pub fn resource(resource_type: String) -> Self {
        let path = CanonicalPath::root(resource_type.clone());
        Self {
            fhir_type: resource_type.clone(),
            resource_type: Some(resource_type),
            path,
            index: None,
        }
    }

    /// Create metadata for a primitive value
    pub fn primitive(fhir_type: String, path: CanonicalPath) -> Self {
        Self {
            fhir_type,
            resource_type: None,
            path,
            index: None,
        }
    }

    /// Create metadata for a complex type (HumanName, Coding, etc.)
    pub fn complex(fhir_type: String, path: CanonicalPath) -> Self {
        Self {
            fhir_type,
            resource_type: None,
            path,
            index: None,
        }
    }

    /// Create metadata for an unknown/unresolved type
    pub fn unknown(path: CanonicalPath) -> Self {
        Self {
            fhir_type: "unknown".to_string(),
            resource_type: None,
            path,
            index: None,
        }
    }

    /// Derive child metadata for property access
    pub fn derive_property(&self, property: &str, child_type: String) -> Self {
        let new_path = self.path.append_property(property);
        Self {
            fhir_type: child_type,
            resource_type: None, // Child properties are not resource roots
            path: new_path,
            index: None,
        }
    }

    /// Derive child metadata for array index access
    pub fn derive_index(&self, index: usize, element_type: Option<String>) -> Self {
        let new_path = self.path.append_index(index);
        let fhir_type = element_type.unwrap_or_else(|| {
            // If we don't know the element type, try to strip 'Array<>' wrapper
            if self.fhir_type.starts_with("Array<") && self.fhir_type.ends_with(">") {
                self.fhir_type[6..self.fhir_type.len() - 1].to_string()
            } else {
                self.fhir_type.clone()
            }
        });

        Self {
            fhir_type,
            resource_type: None,
            path: new_path,
            index: Some(index),
        }
    }

    /// Update the FHIR type (for type resolution)
    pub fn with_type(mut self, fhir_type: String) -> Self {
        self.fhir_type = fhir_type;
        self
    }

    /// Update the path
    pub fn with_path(mut self, path: CanonicalPath) -> Self {
        self.path = path;
        self
    }

    /// Get path as string
    pub fn path_string(&self) -> String {
        self.path.to_string()
    }

    /// Create metadata with type resolution from ModelProvider
    pub async fn resolve_from_path(path: CanonicalPath, resolver: &TypeResolver) -> Result<Self> {
        let fhir_type = resolver.resolve_type_by_path(&path).await?;
        let resource_type = if let Some(root) = path.root_name() {
            if resolver.is_resource_type(root).await {
                Some(root.to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            fhir_type,
            resource_type,
            path,
            index: None,
        })
    }

    /// Create metadata for a FhirPathValue with type inference
    pub fn infer_from_value(value: &crate::core::FhirPathValue, path: CanonicalPath) -> Self {
        let fhir_type = type_utils::fhirpath_value_to_fhir_type(value);

        Self {
            fhir_type,
            resource_type: None, // Will be resolved later if needed
            path,
            index: None,
        }
    }

    /// Update type using resolver (for accurate type resolution)
    pub async fn resolve_type(mut self, resolver: &TypeResolver) -> Result<Self> {
        self.fhir_type = resolver.resolve_type_by_path(&self.path).await?;
        Ok(self)
    }
}

impl fmt::Display for ValueMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.fhir_type, self.path)
    }
}

/// A FHIRPath value wrapped with rich metadata
#[derive(Debug, Clone)]
pub struct WrappedValue {
    /// The actual FHIRPath value
    pub value: FhirPathValue,
    /// Associated metadata with type and path information
    pub metadata: ValueMetadata,
}

impl WrappedValue {
    /// Create a new wrapped value
    pub fn new(value: FhirPathValue, metadata: ValueMetadata) -> Self {
        Self { value, metadata }
    }

    /// Wrap a plain value with metadata
    pub fn wrap(value: FhirPathValue, metadata: ValueMetadata) -> Self {
        Self::new(value, metadata)
    }

    /// Get a reference to the plain value (no naming conflict with Result::unwrap)
    pub fn as_plain(&self) -> &FhirPathValue {
        &self.value
    }

    /// Extract the plain value by consuming the wrapper
    pub fn into_plain(self) -> FhirPathValue {
        self.value
    }

    /// Get both value and metadata as references
    pub fn parts(&self) -> (&FhirPathValue, &ValueMetadata) {
        (&self.value, &self.metadata)
    }

    /// Decompose into value and metadata by consuming the wrapper
    pub fn decompose(self) -> (FhirPathValue, ValueMetadata) {
        (self.value, self.metadata)
    }

    /// Get a reference to the metadata
    pub fn metadata(&self) -> &ValueMetadata {
        &self.metadata
    }

    /// Get the FHIR type name
    pub fn fhir_type(&self) -> &str {
        &self.metadata.fhir_type
    }

    /// Get the canonical path
    pub fn path(&self) -> &CanonicalPath {
        &self.metadata.path
    }

    /// Get the path as string
    pub fn path_string(&self) -> String {
        self.metadata.path.to_string()
    }

    /// Get the resource type if applicable
    pub fn resource_type(&self) -> Option<&str> {
        self.metadata.resource_type.as_deref()
    }

    /// Update metadata while keeping the same value
    pub fn with_metadata(mut self, metadata: ValueMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Update only the FHIR type
    pub fn with_type(mut self, fhir_type: String) -> Self {
        self.metadata.fhir_type = fhir_type;
        self
    }

    /// Update only the path
    pub fn with_path(mut self, path: CanonicalPath) -> Self {
        self.metadata.path = path;
        self
    }

    /// Check if this represents an empty value
    pub fn is_empty(&self) -> bool {
        matches!(self.value, FhirPathValue::Empty)
    }
}

impl fmt::Display for WrappedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.value, self.metadata)
    }
}

/// A collection of wrapped values with preserved metadata
pub type WrappedCollection = Vec<WrappedValue>;

/// Utility functions for working with wrapped collections
pub mod collection_utils {
    use super::*;

    /// Create an empty wrapped collection
    pub fn empty() -> WrappedCollection {
        Vec::new()
    }

    /// Create a single-item wrapped collection
    pub fn single(wrapped: WrappedValue) -> WrappedCollection {
        vec![wrapped]
    }

    /// Create a wrapped collection from plain values with shared base metadata
    pub fn from_plain_with_base_metadata(
        values: Vec<FhirPathValue>,
        base_metadata: ValueMetadata,
    ) -> WrappedCollection {
        let values_len = values.len();
        values
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                let metadata = if values_len > 1 {
                    // Multiple values - derive index metadata
                    base_metadata.derive_index(i, None)
                } else {
                    // Single value - use base metadata as-is
                    base_metadata.clone()
                };
                WrappedValue::new(value, metadata)
            })
            .collect()
    }

    /// Convert wrapped collection back to plain values
    pub fn to_plain_collection(wrapped: WrappedCollection) -> Collection {
        let values: Vec<FhirPathValue> = wrapped.into_iter().map(|w| w.value).collect();
        Collection::from_values(values)
    }

    /// Convert plain collection to wrapped with inferred metadata
    pub fn from_plain_collection(
        collection: Collection,
        base_path: CanonicalPath,
        base_type: String,
    ) -> WrappedCollection {
        let values = collection.into_vec();
        let base_metadata = ValueMetadata {
            fhir_type: base_type,
            resource_type: None,
            path: base_path,
            index: None,
        };

        from_plain_with_base_metadata(values, base_metadata)
    }

    /// Check if collection is empty
    pub fn is_empty(collection: &WrappedCollection) -> bool {
        collection.is_empty()
    }

    /// Get collection length
    pub fn len(collection: &WrappedCollection) -> usize {
        collection.len()
    }

    /// Get first wrapped value if any
    pub fn first(collection: &WrappedCollection) -> Option<&WrappedValue> {
        collection.first()
    }
}

/// Conversion traits for ergonomic API
pub trait IntoWrapped {
    fn into_wrapped(self, metadata: ValueMetadata) -> WrappedValue;
}

pub trait IntoPlain {
    fn into_plain(self) -> FhirPathValue;
}

impl IntoWrapped for FhirPathValue {
    fn into_wrapped(self, metadata: ValueMetadata) -> WrappedValue {
        WrappedValue::new(self, metadata)
    }
}

impl IntoPlain for WrappedValue {
    fn into_plain(self) -> FhirPathValue {
        self.value
    }
}

/// Helper functions for integrating with existing evaluator code
pub mod integration {
    use super::*;

    /// Convert from current engine result format to wrapped format
    pub fn wrap_evaluation_result(
        value: FhirPathValue,
        base_type: String,
        base_path: CanonicalPath,
    ) -> WrappedCollection {
        match value {
            FhirPathValue::Empty => collection_utils::empty(),
            FhirPathValue::Collection(collection) => {
                let base_metadata = ValueMetadata {
                    fhir_type: base_type,
                    resource_type: None,
                    path: base_path,
                    index: None,
                };
                collection_utils::from_plain_with_base_metadata(
                    collection.into_vec(),
                    base_metadata,
                )
            }
            single_value => {
                let metadata = ValueMetadata {
                    fhir_type: base_type,
                    resource_type: None,
                    path: base_path,
                    index: None,
                };
                collection_utils::single(WrappedValue::new(single_value, metadata))
            }
        }
    }

    /// Convert evaluation result to wrapped format with type resolution
    pub async fn wrap_evaluation_result_with_types(
        value: FhirPathValue,
        base_path: CanonicalPath,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match value {
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            FhirPathValue::Collection(collection) => {
                wrap_collection_with_types(collection.into_vec(), base_path, resolver).await
            }
            single_value => {
                let metadata = ValueMetadata::resolve_from_path(base_path, resolver).await?;
                Ok(collection_utils::single(WrappedValue::new(
                    single_value,
                    metadata,
                )))
            }
        }
    }

    /// Wrap a collection of values with proper type resolution
    async fn wrap_collection_with_types(
        values: Vec<FhirPathValue>,
        base_path: CanonicalPath,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        if values.is_empty() {
            return Ok(collection_utils::empty());
        }

        // Resolve base type
        let base_type = resolver.resolve_type_by_path(&base_path).await?;
        let element_type = resolver.resolve_element_type(&base_type).await?;

        // Create wrapped values with indexed paths
        let values_len = values.len();
        let mut wrapped_values = Vec::new();
        for (i, value) in values.into_iter().enumerate() {
            let metadata = if values_len > 1 {
                // Multiple values - create indexed metadata
                let indexed_path = base_path.append_index(i);
                ValueMetadata {
                    fhir_type: element_type.clone(),
                    resource_type: None,
                    path: indexed_path,
                    index: Some(i),
                }
            } else {
                // Single value - use base metadata
                ValueMetadata {
                    fhir_type: base_type.clone(),
                    resource_type: None,
                    path: base_path.clone(),
                    index: None,
                }
            };

            wrapped_values.push(WrappedValue::new(value, metadata));
        }

        Ok(wrapped_values)
    }

    /// Infer metadata from a resource JSON value
    pub async fn infer_resource_metadata(
        resource_json: &serde_json::Value,
        resolver: &TypeResolver,
    ) -> Result<ValueMetadata> {
        let resource_type = resolver.resolve_resource_type(resource_json).await?;
        let path = CanonicalPath::root(resource_type.clone());

        Ok(ValueMetadata {
            fhir_type: resource_type.clone(),
            resource_type: Some(resource_type),
            path,
            index: None,
        })
    }

    /// Convert wrapped result back to current engine result format (for compatibility)
    pub fn unwrap_to_evaluation_result(wrapped: WrappedCollection) -> FhirPathValue {
        if wrapped.is_empty() {
            FhirPathValue::Empty
        } else if wrapped.len() == 1 {
            wrapped.into_iter().next().unwrap().value
        } else {
            let values: Vec<FhirPathValue> = wrapped.into_iter().map(|w| w.value).collect();
            FhirPathValue::Collection(Collection::from_values(values))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::FhirPathValue;

    #[test]
    fn test_metadata_creation() {
        let resource_meta = ValueMetadata::resource("Patient".to_string());
        assert_eq!(resource_meta.fhir_type, "Patient");
        assert_eq!(resource_meta.resource_type, Some("Patient".to_string()));
        assert_eq!(resource_meta.path.to_string(), "Patient");

        let primitive_meta = ValueMetadata::primitive(
            "string".to_string(),
            CanonicalPath::parse("Patient.name").unwrap(),
        );
        assert_eq!(primitive_meta.fhir_type, "string");
        assert_eq!(primitive_meta.resource_type, None);
        assert_eq!(primitive_meta.path.to_string(), "Patient.name");
    }

    #[test]
    fn test_metadata_derivation() {
        let base = ValueMetadata::resource("Patient".to_string());
        let name_meta = base.derive_property("name", "HumanName".to_string());

        assert_eq!(name_meta.fhir_type, "HumanName");
        assert_eq!(name_meta.path.to_string(), "Patient.name");
        assert_eq!(name_meta.resource_type, None);

        let indexed_meta = name_meta.derive_index(0, Some("HumanName".to_string()));
        assert_eq!(indexed_meta.path.to_string(), "Patient.name[0]");
        assert_eq!(indexed_meta.index, Some(0));
    }

    #[test]
    fn test_wrapped_value_operations() {
        let value = FhirPathValue::String("John".to_string());
        let metadata = ValueMetadata::primitive(
            "string".to_string(),
            CanonicalPath::parse("Patient.name.given").unwrap(),
        );
        let wrapped = WrappedValue::new(value.clone(), metadata);

        assert_eq!(wrapped.as_plain(), &value);
        assert_eq!(wrapped.fhir_type(), "string");
        assert_eq!(wrapped.path_string(), "Patient.name.given");

        let (extracted_value, extracted_metadata) = wrapped.parts();
        assert_eq!(extracted_value, &value);
        assert_eq!(extracted_metadata.fhir_type, "string");
    }

    #[test]
    fn test_collection_utilities() {
        let values = vec![
            FhirPathValue::String("John".to_string()),
            FhirPathValue::String("Jane".to_string()),
        ];
        let base_metadata = ValueMetadata::primitive(
            "string".to_string(),
            CanonicalPath::parse("Patient.name.given").unwrap(),
        );

        let wrapped_collection =
            collection_utils::from_plain_with_base_metadata(values, base_metadata);
        assert_eq!(collection_utils::len(&wrapped_collection), 2);

        let first = collection_utils::first(&wrapped_collection).unwrap();
        assert_eq!(first.path_string(), "Patient.name.given[0]");
        assert_eq!(first.fhir_type(), "string");
    }

    #[test]
    fn test_conversion_traits() {
        let value = FhirPathValue::String("test".to_string());
        let metadata = ValueMetadata::primitive(
            "string".to_string(),
            CanonicalPath::parse("test.path").unwrap(),
        );

        let wrapped: WrappedValue = value.clone().into_wrapped(metadata);
        let plain: FhirPathValue = wrapped.into_plain();

        assert_eq!(plain, value);
    }
}
