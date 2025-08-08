use crate::error::FhirPathError;
use crate::model::{FhirPathValue, FhirResource};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ArcBundleEntry {
    pub id: Option<Arc<str>>,
    pub fullurl: Option<Arc<str>>,
    pub resource: Arc<Value>,
    pub request: Option<Arc<Value>>,
    pub response: Option<Arc<Value>>,
    pub search: Option<Arc<Value>>,
}

impl ArcBundleEntry {
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

    pub fn get_resource_type(&self) -> Option<&str> {
        self.resource
            .as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|v| v.as_str())
    }

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

#[derive(Debug, Clone)]
pub struct ArcBundle {
    pub entries: Arc<[ArcBundleEntry]>,
    pub resource_index: Arc<HashMap<String, Vec<usize>>>,
    pub id_index: Arc<HashMap<String, usize>>,
    pub url_index: Arc<HashMap<String, usize>>,
    pub metadata: Arc<BundleMetadata>,
}

#[derive(Debug, Clone)]
pub struct BundleMetadata {
    pub bundle_type: Option<Arc<str>>,
    pub total: Option<u64>,
    pub timestamp: Option<Arc<str>>,
    pub identifier: Option<Arc<Value>>,
    pub signature: Option<Arc<Value>>,
}

impl BundleMetadata {
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

    pub fn get_entries_by_type(&self, resource_type: &str) -> Vec<&ArcBundleEntry> {
        self.resource_index
            .get(resource_type)
            .map(|indices| indices.iter().map(|&idx| &self.entries[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_entry_by_id(&self, id: &str) -> Option<&ArcBundleEntry> {
        self.id_index.get(id).map(|&idx| &self.entries[idx])
    }

    pub fn get_entry_by_url(&self, url: &str) -> Option<&ArcBundleEntry> {
        self.url_index.get(url).map(|&idx| &self.entries[idx])
    }

    pub fn materialize_all_resources(&self) -> Vec<FhirPathValue> {
        self.entries
            .iter()
            .filter_map(|entry| entry.materialize_resource())
            .collect()
    }

    pub fn materialize_resources_by_type(&self, resource_type: &str) -> Vec<FhirPathValue> {
        self.get_entries_by_type(resource_type)
            .into_iter()
            .filter_map(|entry| entry.materialize_resource())
            .collect()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn has_resource_type(&self, resource_type: &str) -> bool {
        self.resource_index.contains_key(resource_type)
    }

    pub fn resource_types(&self) -> Vec<&String> {
        self.resource_index.keys().collect()
    }
}

#[derive(Debug, Clone)]
pub struct BundleView {
    bundle: Arc<ArcBundle>,
    indices: Arc<[usize]>,
}

impl BundleView {
    pub fn new(bundle: Arc<ArcBundle>, indices: Vec<usize>) -> Self {
        BundleView {
            bundle,
            indices: Arc::from(indices),
        }
    }

    pub fn from_type_filter(bundle: Arc<ArcBundle>, resource_type: &str) -> Self {
        let indices = bundle
            .resource_index
            .get(resource_type)
            .cloned()
            .unwrap_or_default();
        Self::new(bundle, indices)
    }

    pub fn materialize(&self) -> Vec<FhirPathValue> {
        self.indices
            .iter()
            .filter_map(|&idx| self.bundle.entries[idx].materialize_resource())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

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
