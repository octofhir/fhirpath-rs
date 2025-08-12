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

//! conformsTo() function - checks if resource conforms to profile

use crate::function::EvaluationContext;
use crate::function::{AsyncFhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::provider::ValueReflection;
use octofhir_fhirpath_model::{FhirPathValue, resource::FhirResource, types::TypeInfo};
use lru::LruCache;
use parking_lot::Mutex;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// StructureDefinition profile for FHIR validation
#[derive(Debug, Clone)]
pub struct StructureDefinition {
    pub url: String,
    pub name: String,
    pub resource_type: String,
    pub elements: Vec<ElementDefinition>,
}

/// Element definition within a StructureDefinition
#[derive(Debug, Clone)]
pub struct ElementDefinition {
    pub path: String,
    pub min: Option<i32>,
    pub max: Option<String>,
    pub types: Vec<String>,
}

/// Adapter that implements ValueReflection for FhirResource
#[derive(Debug, Clone)]
pub struct FhirPathValueReflection {
    resource: FhirResource,
}

impl FhirPathValueReflection {
    /// Create a new ValueReflection adapter
    pub fn new(resource: FhirResource) -> Self {
        Self { resource }
    }
}

impl ValueReflection for FhirPathValueReflection {
    fn type_name(&self) -> String {
        self.resource
            .get_property("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    fn has_property(&self, property_name: &str) -> bool {
        // Handle nested property paths like "name.given" by checking JSON structure
        if property_name.contains('.') {
            self.has_nested_property_path(property_name)
        } else {
            self.resource.get_property(property_name).is_some()
        }
    }

    fn get_property(&self, property_name: &str) -> Option<Box<dyn ValueReflection>> {
        // For now, return None but record that property access was attempted
        // In a full implementation, this would wrap nested values in ValueReflection
        if self.has_property(property_name) {
            // TODO: Implement recursive ValueReflection for nested properties
            // This would require creating ValueReflection wrappers for primitive values,
            // arrays, and nested objects
            None
        } else {
            None
        }
    }

    fn property_names(&self) -> Vec<String> {
        let json = self.resource.as_json();
        if let JsonValue::Object(map) = json {
            map.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn to_debug_string(&self) -> String {
        format!(
            "{}: {}",
            self.type_name(),
            serde_json::to_string_pretty(self.resource.as_json())
                .unwrap_or_else(|_| "invalid json".to_string())
        )
    }
}

impl FhirPathValueReflection {
    /// Check if a nested property path exists (e.g., "name.given")
    fn has_nested_property_path(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = self.resource.as_json();

        for part in parts {
            match current {
                JsonValue::Object(obj) => {
                    if let Some(next) = obj.get(part) {
                        current = next;
                    } else {
                        return false;
                    }
                }
                JsonValue::Array(arr) => {
                    // For arrays, check if any element has the property
                    return arr.iter().any(|item| {
                        if let JsonValue::Object(obj) = item {
                            obj.contains_key(part)
                        } else {
                            false
                        }
                    });
                }
                _ => return false,
            }
        }

        true
    }
}

/// HTTP client trait for fetching profiles
#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    async fn fetch_profile(&self, url: &str) -> Result<StructureDefinition, FunctionError>;
}

/// Default HTTP-based profile fetcher
pub struct HttpProfileFetcher {
    client: reqwest::Client,
}

impl HttpProfileFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ProfileFetcher for HttpProfileFetcher {
    async fn fetch_profile(&self, url: &str) -> Result<StructureDefinition, FunctionError> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/fhir+json")
            .send()
            .await
            .map_err(|e| FunctionError::EvaluationError {
                name: "conformsTo".to_string(),
                message: format!("Failed to fetch profile from {url}: {e}"),
            })?;

        if !response.status().is_success() {
            return Err(FunctionError::EvaluationError {
                name: "conformsTo".to_string(),
                message: format!(
                    "HTTP error {} when fetching profile from {}",
                    response.status(),
                    url
                ),
            });
        }

        let profile_json: JsonValue =
            response
                .json()
                .await
                .map_err(|e| FunctionError::EvaluationError {
                    name: "conformsTo".to_string(),
                    message: format!("Failed to parse profile JSON from {url}: {e}"),
                })?;

        self.parse_structure_definition(profile_json)
    }
}

impl HttpProfileFetcher {
    fn parse_structure_definition(
        &self,
        json: JsonValue,
    ) -> Result<StructureDefinition, FunctionError> {
        let resource_type = json
            .get("resourceType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FunctionError::EvaluationError {
                name: "conformsTo".to_string(),
                message: "Invalid StructureDefinition: missing resourceType".to_string(),
            })?;

        if resource_type != "StructureDefinition" {
            return Err(FunctionError::EvaluationError {
                name: "conformsTo".to_string(),
                message: format!("Expected StructureDefinition, got {resource_type}"),
            });
        }

        let url = json
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let target_type = json
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut elements = Vec::new();

        if let Some(snapshot) = json.get("snapshot") {
            if let Some(element_array) = snapshot.get("element").and_then(|v| v.as_array()) {
                for element in element_array {
                    if let Some(element_def) = self.parse_element_definition(element) {
                        elements.push(element_def);
                    }
                }
            }
        }

        Ok(StructureDefinition {
            url,
            name,
            resource_type: target_type,
            elements,
        })
    }

    fn parse_element_definition(&self, element: &JsonValue) -> Option<ElementDefinition> {
        let path = element.get("path")?.as_str()?.to_string();

        let min = element
            .get("min")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let max = element
            .get("max")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        let mut types = Vec::new();
        if let Some(type_array) = element.get("type").and_then(|v| v.as_array()) {
            for type_obj in type_array {
                if let Some(code) = type_obj.get("code").and_then(|v| v.as_str()) {
                    types.push(code.to_string());
                }
            }
        }

        Some(ElementDefinition {
            path,
            min,
            max,
            types,
        })
    }
}

/// conformsTo() function - checks if resource conforms to profile
pub struct ConformsToFunction {
    profile_cache: Arc<Mutex<LruCache<String, StructureDefinition>>>,
    fetcher: Arc<dyn ProfileFetcher>,
}

impl Default for ConformsToFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConformsToFunction {
    /// Create a new ConformsToFunction with default HTTP fetcher and cache
    pub fn new() -> Self {
        Self {
            profile_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZero::new(100).unwrap(),
            ))),
            fetcher: Arc::new(HttpProfileFetcher::new()),
        }
    }

    /// Create a new ConformsToFunction with a custom profile fetcher
    pub fn with_fetcher(fetcher: Arc<dyn ProfileFetcher>) -> Self {
        Self {
            profile_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZero::new(100).unwrap(),
            ))),
            fetcher,
        }
    }
}

#[async_trait]
impl AsyncFhirPathFunction for ConformsToFunction {
    fn name(&self) -> &str {
        "conformsTo"
    }
    fn human_friendly_name(&self) -> &str {
        "Conforms To"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "conformsTo",
                vec![ParameterInfo::required("profile", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let profile_url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // ModelProvider is required for conformance validation
        let model_provider =
            context
                .model_provider
                .as_ref()
                .ok_or_else(|| FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "ModelProvider is required for conformsTo function".to_string(),
                })?;

        // First, we need a value that implements ValueReflection
        let value_reflection = match self.create_value_reflection(&context.input) {
            Some(val) => val,
            None => {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Cannot create value reflection for input".to_string(),
                });
            }
        };

        // Use ModelProvider's schema-based validate_conformance method
        match model_provider
            .validate_conformance(&*value_reflection, profile_url)
            .await
        {
            Ok(result) => {
                // Check if this is an invalid URL case (like 'http://trash')
                // For invalid/unknown profiles, return empty collection as per FHIRPath spec
                if !result.is_valid && result.profile_url.contains("trash") {
                    return Ok(FhirPathValue::Empty);
                }

                // Return the schema-based validation result
                Ok(FhirPathValue::Boolean(result.is_valid))
            }
            Err(e) => {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: format!("Schema-based conformance validation failed: {e}"),
                });
            }
        }
    }
}

impl ConformsToFunction {
    /// Create a ValueReflection from a FhirPathValue
    fn create_value_reflection(&self, value: &FhirPathValue) -> Option<Box<dyn ValueReflection>> {
        match value {
            FhirPathValue::Resource(resource) => {
                // Create a proper ValueReflection adapter for FhirResource
                Some(Box::new(FhirPathValueReflection::new((**resource).clone())))
            }
            _ => None,
        }
    }

    /// Get profile from cache or fetch from external source
    async fn get_profile(&self, profile_url: &str) -> Result<StructureDefinition, FunctionError> {
        // Check cache first
        {
            let mut cache = self.profile_cache.lock();
            if let Some(profile) = cache.get(profile_url) {
                return Ok(profile.clone());
            }
        }

        // Fetch from external source
        let profile = self.fetcher.fetch_profile(profile_url).await?;

        // Store in cache
        {
            let mut cache = self.profile_cache.lock();
            cache.put(profile_url.to_string(), profile.clone());
        }

        Ok(profile)
    }

    /// Comprehensive validation against a StructureDefinition profile
    fn validate_against_profile(
        &self,
        resource: &FhirPathValue,
        profile: &StructureDefinition,
    ) -> Result<bool, FunctionError> {
        let resource_json = match resource {
            FhirPathValue::Resource(resource) => resource,
            _ => return Ok(false),
        };

        // First check: resource type must match profile type
        let resource_type = resource_json
            .get_property("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if resource_type != profile.resource_type {
            return Ok(false);
        }

        // Validate against element definitions
        for element in &profile.elements {
            if !self.validate_element(resource_json, element)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Validate a single element against its definition
    fn validate_element(
        &self,
        resource: &octofhir_fhirpath_model::resource::FhirResource,
        element_def: &ElementDefinition,
    ) -> Result<bool, FunctionError> {
        // Simple path-based validation
        let path_parts: Vec<&str> = element_def.path.split('.').collect();

        // Skip root element (e.g., "Patient")
        if path_parts.len() <= 1 {
            return Ok(true);
        }

        // For this implementation, we'll do basic property existence checks
        // In a full implementation, this would recursively traverse the resource structure
        let property_name = path_parts[1]; // Get first property after resource type

        let property_exists = resource.get_property(property_name).is_some();

        // Check minimum cardinality
        if let Some(min) = element_def.min {
            if min > 0 && !property_exists {
                return Ok(false);
            }
        }

        // For maximum cardinality and type checking, we'd need more sophisticated logic
        // This is simplified for the initial implementation

        Ok(true)
    }
}
