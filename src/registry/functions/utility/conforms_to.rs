//! conformsTo() function - checks if resource conforms to profile

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
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

/// HTTP client trait for fetching profiles
#[async_trait]
pub trait ProfileFetcher: Send + Sync {
    async fn fetch_profile(&self, url: &str) -> Result<StructureDefinition, FunctionError>;
}

/// Default HTTP-based profile fetcher
pub struct HttpProfileFetcher {
    #[cfg(feature = "reqwest")]
    client: reqwest::Client,
}

impl HttpProfileFetcher {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "reqwest")]
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ProfileFetcher for HttpProfileFetcher {
    async fn fetch_profile(&self, _url: &str) -> Result<StructureDefinition, FunctionError> {
        #[cfg(feature = "reqwest")]
        {
            let response = self
                .client
                .get(url)
                .header("Accept", "application/fhir+json")
                .send()
                .await
                .map_err(|e| FunctionError::EvaluationError {
                    name: "conformsTo".to_string(),
                    message: format!("Failed to fetch profile from {}: {}", url, e),
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
                        message: format!("Failed to parse profile JSON from {}: {}", url, e),
                    })?;

            self.parse_structure_definition(profile_json)
        }

        #[cfg(not(feature = "reqwest"))]
        {
            Err(FunctionError::EvaluationError {
                name: "conformsTo".to_string(),
                message: "HTTP client not available. Enable 'reqwest' feature to fetch external profiles.".to_string(),
            })
        }
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

        // Get the profile definition (from cache or fetch)
        let profile = match self.get_profile(profile_url).await {
            Ok(profile) => profile,
            Err(_e) => {
                // If we can't get the profile, fall back to basic validation
                return self.basic_conformance_check(profile_url, context);
            }
        };

        // Perform comprehensive validation against the profile
        let conforms = self.validate_against_profile(&context.input, &profile)?;
        Ok(FhirPathValue::Boolean(conforms))
    }
}

impl ConformsToFunction {
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

    /// Basic conformance check (fallback when profile can't be fetched)
    fn basic_conformance_check(
        &self,
        profile_url: &str,
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Extract resource type from context input
        let resource_type = match &context.input {
            FhirPathValue::Resource(resource) => {
                if let Some(JsonValue::String(rt)) = resource.get_property("resourceType") {
                    rt.clone()
                } else {
                    return Ok(FhirPathValue::Boolean(false));
                }
            }
            _ => return Ok(FhirPathValue::Boolean(false)),
        };

        // Basic profile conformance check for standard FHIR profiles
        if profile_url.starts_with("http://hl7.org/fhir/StructureDefinition/") {
            let profile_resource_type = profile_url
                .strip_prefix("http://hl7.org/fhir/StructureDefinition/")
                .unwrap_or("");

            let conforms = resource_type == profile_resource_type;
            return Ok(FhirPathValue::Boolean(conforms));
        }

        // For other URLs, return false as we can't validate without the profile
        Ok(FhirPathValue::Boolean(false))
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
        resource: &crate::model::FhirResource,
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
