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

use octofhir_fhirpath_core::FhirPathError;
use octofhir_fhirpath_model::{FhirPathValue, resource::FhirResource};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A memory-efficient FHIR Bundle entry using Arc for shared data
///
/// This struct represents a single entry in a FHIR Bundle, using `Arc<T>` for
/// sharing common data and reducing memory footprint when Bundle entries are cloned.
#[derive(Debug, Clone)]
pub struct ArcBundleEntry {
    /// The logical ID of the entry, if present
    pub id: Option<Arc<str>>,
    /// The absolute URL for the resource, if present
    pub fullurl: Option<Arc<str>>,
    /// The actual FHIR resource content as JSON
    pub resource: Arc<Value>,
    /// Information about the request that created this entry, if present
    pub request: Option<Arc<Value>>,
    /// Information about the response for this entry, if present
    pub response: Option<Arc<Value>>,
    /// Search-related information for this entry, if present
    pub search: Option<Arc<Value>>,
}

impl ArcBundleEntry {
    /// Creates an ArcBundleEntry from a JSON Value
    ///
    /// # Arguments
    /// * `entry` - The JSON object representing a Bundle entry
    ///
    /// # Returns
    /// * `Result<Self, FhirPathError>` - The parsed entry or an error
    pub fn from_json(entry: &Value) -> Result<Self, FhirPathError> {
        let obj = entry
            .as_object()
            .ok_or_else(|| FhirPathError::evaluation_error("Bundle entry must be an object"))?;

        Ok(ArcBundleEntry {
            id: obj.get("id").and_then(|v| v.as_str()).map(Arc::from),
            fullurl: obj.get("fullUrl").and_then(|v| v.as_str()).map(Arc::from),
            resource: obj
                .get("resource")
                .map(|v| Arc::new(v.clone()))
                .unwrap_or_else(|| Arc::new(Value::Null)),
            request: obj.get("request").map(|v| Arc::new(v.clone())),
            response: obj.get("response").map(|v| Arc::new(v.clone())),
            search: obj.get("search").map(|v| Arc::new(v.clone())),
        })
    }

    /// Gets the resourceType of the contained resource
    ///
    /// # Returns
    /// * `Option<&str>` - The resource type name, if present
    pub fn get_resource_type(&self) -> Option<&str> {
        self.resource
            .as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|v| v.as_str())
    }

    /// Converts the JSON resource to a FhirPathValue for evaluation
    ///
    /// # Returns
    /// * `Option<FhirPathValue>` - The materialized resource, or None if null
    pub fn materialize_resource(&self) -> Option<FhirPathValue> {
        if self.resource.is_null() {
            None
        } else {
            Some(FhirPathValue::Resource(Arc::new(FhirResource::from_json(
                self.resource.as_ref().clone(),
            ))))
        }
    }
}

/// A memory-efficient FHIR Bundle with indexed access
///
/// This struct provides fast lookup capabilities for Bundle entries while minimizing
/// memory usage through Arc sharing. It maintains indices for quick access by
/// resource type, ID, and URL.
#[derive(Debug, Clone)]
pub struct ArcBundle {
    /// All entries in the bundle, stored as a shared array slice
    pub entries: Arc<[ArcBundleEntry]>,
    /// Index mapping resource types to entry indices for fast lookup
    pub resource_index: Arc<HashMap<String, Vec<usize>>>,
    /// Index mapping entry IDs to entry indices for fast lookup
    pub id_index: Arc<HashMap<String, usize>>,
    /// Index mapping fullUrls to entry indices for fast lookup
    pub url_index: Arc<HashMap<String, usize>>,
    /// Bundle-level metadata and properties
    pub metadata: Arc<BundleMetadata>,
}

/// Metadata extracted from a FHIR Bundle resource
///
/// Contains bundle-level properties like type, total count, timestamp, etc.
#[derive(Debug, Clone)]
pub struct BundleMetadata {
    /// The type of Bundle (e.g., "searchset", "collection", "transaction")
    pub bundle_type: Option<Arc<str>>,
    /// Total number of matches for search bundles
    pub total: Option<u64>,
    /// When the bundle was assembled (ISO 8601 timestamp)
    pub timestamp: Option<Arc<str>>,
    /// Persistent identifier for this bundle instance
    pub identifier: Option<Arc<Value>>,
    /// Digital signature for the bundle content
    pub signature: Option<Arc<Value>>,
}

impl BundleMetadata {
    /// Creates BundleMetadata from a Bundle JSON object
    ///
    /// # Arguments
    /// * `bundle` - The JSON object representing a FHIR Bundle
    ///
    /// # Returns
    /// * `Result<Self, FhirPathError>` - The parsed metadata or an error
    pub fn from_json(bundle: &Value) -> Result<Self, FhirPathError> {
        let obj = bundle
            .as_object()
            .ok_or_else(|| FhirPathError::evaluation_error("Bundle must be an object"))?;

        Ok(BundleMetadata {
            bundle_type: obj.get("type").and_then(|v| v.as_str()).map(Arc::from),
            total: obj.get("total").and_then(|v| v.as_u64()),
            timestamp: obj.get("timestamp").and_then(|v| v.as_str()).map(Arc::from),
            identifier: obj.get("identifier").map(|v| Arc::new(v.clone())),
            signature: obj.get("signature").map(|v| Arc::new(v.clone())),
        })
    }
}

impl ArcBundle {
    /// Creates an ArcBundle from a FHIR Bundle JSON object
    ///
    /// This method parses the Bundle, creates indices for fast lookup, and wraps
    /// everything in Arc for efficient sharing.
    ///
    /// # Arguments
    /// * `bundle` - The JSON object representing a FHIR Bundle
    ///
    /// # Returns
    /// * `Result<Self, FhirPathError>` - The parsed bundle or an error
    pub fn from_json(bundle: &Value) -> Result<Self, FhirPathError> {
        let obj = bundle
            .as_object()
            .ok_or_else(|| FhirPathError::evaluation_error("Bundle must be an object"))?;

        let resource_type = obj
            .get("resourceType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FhirPathError::evaluation_error("Bundle must have resourceType"))?;

        if resource_type != "Bundle" {
            return Err(FhirPathError::evaluation_error(format!(
                "Expected Bundle resourceType, got {resource_type}"
            )));
        }

        let empty_vec = Vec::new();
        let entries_json = obj
            .get("entry")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let entries: Vec<ArcBundleEntry> = entries_json
            .iter()
            .map(ArcBundleEntry::from_json)
            .collect::<Result<Vec<_>, _>>()?;

        let mut resource_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut id_index: HashMap<String, usize> = HashMap::new();
        let mut url_index: HashMap<String, usize> = HashMap::new();

        for (idx, entry) in entries.iter().enumerate() {
            if let Some(resource_type) = entry.get_resource_type() {
                resource_index
                    .entry(resource_type.to_string())
                    .or_default()
                    .push(idx);
            }

            if let Some(id) = &entry.id {
                id_index.insert(id.to_string(), idx);
            }

            if let Some(fullurl) = &entry.fullurl {
                url_index.insert(fullurl.to_string(), idx);
            }
        }

        let metadata = BundleMetadata::from_json(bundle)?;

        Ok(ArcBundle {
            entries: Arc::from(entries),
            resource_index: Arc::new(resource_index),
            id_index: Arc::new(id_index),
            url_index: Arc::new(url_index),
            metadata: Arc::new(metadata),
        })
    }

    /// Gets all bundle entries of a specific resource type
    ///
    /// # Arguments
    /// * `resource_type` - The FHIR resource type to filter by
    ///
    /// # Returns
    /// * `Vec<&ArcBundleEntry>` - Vector of entries matching the resource type
    pub fn get_entries_by_type(&self, resource_type: &str) -> Vec<&ArcBundleEntry> {
        self.resource_index
            .get(resource_type)
            .map(|indices| indices.iter().map(|&idx| &self.entries[idx]).collect())
            .unwrap_or_default()
    }

    /// Gets a bundle entry by its logical ID
    ///
    /// # Arguments
    /// * `id` - The logical ID to search for
    ///
    /// # Returns
    /// * `Option<&ArcBundleEntry>` - The entry if found, None otherwise
    pub fn get_entry_by_id(&self, id: &str) -> Option<&ArcBundleEntry> {
        self.id_index.get(id).map(|&idx| &self.entries[idx])
    }

    /// Gets a bundle entry by its fullUrl
    ///
    /// # Arguments
    /// * `url` - The fullUrl to search for
    ///
    /// # Returns
    /// * `Option<&ArcBundleEntry>` - The entry if found, None otherwise
    pub fn get_entry_by_url(&self, url: &str) -> Option<&ArcBundleEntry> {
        self.url_index.get(url).map(|&idx| &self.entries[idx])
    }

    /// Converts all bundle entries to FhirPathValues for evaluation
    ///
    /// # Returns
    /// * `Vec<FhirPathValue>` - All resources as FhirPath values
    pub fn materialize_all_resources(&self) -> Vec<FhirPathValue> {
        self.entries
            .iter()
            .filter_map(|entry| entry.materialize_resource())
            .collect()
    }

    /// Converts bundle entries of a specific type to FhirPathValues
    ///
    /// # Arguments
    /// * `resource_type` - The FHIR resource type to materialize
    ///
    /// # Returns
    /// * `Vec<FhirPathValue>` - Matching resources as FhirPath values
    pub fn materialize_resources_by_type(&self, resource_type: &str) -> Vec<FhirPathValue> {
        self.get_entries_by_type(resource_type)
            .into_iter()
            .filter_map(|entry| entry.materialize_resource())
            .collect()
    }

    /// Gets the total number of entries in the bundle
    ///
    /// # Returns
    /// * `usize` - The number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Checks if the bundle contains any entries of the specified resource type
    ///
    /// # Arguments
    /// * `resource_type` - The FHIR resource type to check for
    ///
    /// # Returns
    /// * `bool` - True if the bundle contains entries of this type
    pub fn has_resource_type(&self, resource_type: &str) -> bool {
        self.resource_index.contains_key(resource_type)
    }

    /// Gets a list of all resource types present in the bundle
    ///
    /// # Returns
    /// * `Vec<&String>` - Vector of unique resource type names
    pub fn resource_types(&self) -> Vec<&String> {
        self.resource_index.keys().collect()
    }
}

/// A filtered view over an ArcBundle
///
/// BundleView provides an efficient way to work with a subset of bundle entries
/// without copying the underlying data. It holds references to selected indices
/// in the original bundle.
#[derive(Debug, Clone)]
pub struct BundleView {
    /// Reference to the underlying bundle
    bundle: Arc<ArcBundle>,
    /// Indices of entries to include in this view
    indices: Arc<[usize]>,
}

impl BundleView {
    /// Creates a new BundleView with specified entry indices
    ///
    /// # Arguments
    /// * `bundle` - The underlying bundle to view
    /// * `indices` - Vector of entry indices to include in the view
    ///
    /// # Returns
    /// * `Self` - A new BundleView instance
    pub fn new(bundle: Arc<ArcBundle>, indices: Vec<usize>) -> Self {
        BundleView {
            bundle,
            indices: Arc::from(indices),
        }
    }

    /// Creates a BundleView filtered by resource type
    ///
    /// # Arguments
    /// * `bundle` - The underlying bundle to filter
    /// * `resource_type` - The FHIR resource type to include
    ///
    /// # Returns
    /// * `Self` - A new BundleView containing only entries of the specified type
    pub fn from_type_filter(bundle: Arc<ArcBundle>, resource_type: &str) -> Self {
        let indices = bundle
            .resource_index
            .get(resource_type)
            .cloned()
            .unwrap_or_default();
        Self::new(bundle, indices)
    }

    /// Converts the entries in this view to FhirPathValues for evaluation
    ///
    /// # Returns
    /// * `Vec<FhirPathValue>` - The entries in this view as FhirPath values
    pub fn materialize(&self) -> Vec<FhirPathValue> {
        self.indices
            .iter()
            .filter_map(|&idx| self.bundle.entries[idx].materialize_resource())
            .collect()
    }

    /// Gets the number of entries in this view
    ///
    /// # Returns
    /// * `usize` - The number of entries in the view
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Checks if this view contains any entries
    ///
    /// # Returns
    /// * `bool` - True if the view is empty
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Creates an iterator over the entries in this view
    ///
    /// # Returns
    /// * `impl Iterator<Item = &ArcBundleEntry>` - Iterator over view entries
    pub fn iter(&self) -> impl Iterator<Item = &ArcBundleEntry> + '_ {
        self.indices
            .iter()
            .map(move |&idx| &self.bundle.entries[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_arc_bundle_creation() {
        let bundle_json = json!({
            "resourceType": "Bundle",
            "type": "searchset",
            "total": 2,
            "entry": [
                {
                    "fullUrl": "http://example.org/Patient/1",
                    "resource": {
                        "resourceType": "Patient",
                        "id": "1",
                        "name": [{"family": "Smith"}]
                    }
                },
                {
                    "fullUrl": "http://example.org/Patient/2",
                    "resource": {
                        "resourceType": "Patient",
                        "id": "2",
                        "name": [{"family": "Jones"}]
                    }
                }
            ]
        });

        let bundle = ArcBundle::from_json(&bundle_json).unwrap();

        assert_eq!(bundle.entry_count(), 2);
        assert!(bundle.has_resource_type("Patient"));
        assert_eq!(bundle.get_entries_by_type("Patient").len(), 2);

        let patient1 = bundle.get_entry_by_url("http://example.org/Patient/1");
        assert!(patient1.is_some());
        assert_eq!(patient1.unwrap().get_resource_type(), Some("Patient"));
    }

    #[test]
    fn test_bundle_view() {
        let bundle_json = json!({
            "resourceType": "Bundle",
            "type": "collection",
            "entry": [
                {
                    "resource": {
                        "resourceType": "Patient",
                        "id": "1"
                    }
                },
                {
                    "resource": {
                        "resourceType": "Observation",
                        "id": "1"
                    }
                },
                {
                    "resource": {
                        "resourceType": "Patient",
                        "id": "2"
                    }
                }
            ]
        });

        let bundle = Arc::new(ArcBundle::from_json(&bundle_json).unwrap());
        let patient_view = BundleView::from_type_filter(bundle.clone(), "Patient");

        assert_eq!(patient_view.len(), 2);
        assert!(!patient_view.is_empty());

        let materialized = patient_view.materialize();
        assert_eq!(materialized.len(), 2);
    }

    #[test]
    fn test_arc_sharing() {
        let bundle_json = json!({
            "resourceType": "Bundle",
            "type": "searchset",
            "entry": [
                {
                    "resource": {
                        "resourceType": "Patient",
                        "id": "1"
                    }
                }
            ]
        });

        let bundle1 = ArcBundle::from_json(&bundle_json).unwrap();
        let bundle2 = bundle1.clone();

        assert!(Arc::ptr_eq(&bundle1.entries, &bundle2.entries));
        assert!(Arc::ptr_eq(
            &bundle1.resource_index,
            &bundle2.resource_index
        ));
        assert!(Arc::ptr_eq(&bundle1.metadata, &bundle2.metadata));
    }
}
