//! FHIR Schema support for model provider

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::error::{ModelError, Result};
use crate::provider::{FhirVersion, ModelProvider, SearchParameter};
use crate::types::TypeInfo;

/// FHIR Schema representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirSchema {
    /// Schema URL
    pub url: String,
    /// Schema version
    pub version: String,
    /// Schema date
    pub date: String,
    /// Type definitions
    pub definitions: HashMap<String, TypeDefinition>,
}

/// Type definition in FHIR Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// Type URL
    pub url: String,
    /// Base type
    pub base: Option<String>,
    /// Type kind (resource, complex-type, primitive-type)
    pub kind: String,
    /// Derivation (specialization, constraint)
    pub derivation: Option<String>,
    /// Element definitions
    pub elements: HashMap<String, ElementDefinition>,
}

/// Element definition in FHIR Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementDefinition {
    /// Element types
    #[serde(rename = "type")]
    pub types: Option<Vec<TypeReference>>,
    /// Minimum cardinality
    pub min: u32,
    /// Maximum cardinality ("*" for unbounded)
    pub max: String,
    /// Fixed value
    pub fixed: Option<serde_json::Value>,
    /// Pattern value
    pub pattern: Option<serde_json::Value>,
    /// Binding information
    pub binding: Option<Binding>,
    /// Is modifier element
    pub is_modifier: Option<bool>,
    /// Is summary element
    pub is_summary: Option<bool>,
}

/// Type reference in element definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeReference {
    /// Type code
    pub code: String,
    /// Target profiles
    #[serde(rename = "targetProfile")]
    pub target_profiles: Option<Vec<String>>,
}

/// Binding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// Binding strength
    pub strength: String,
    /// Value set URL
    #[serde(rename = "valueSet")]
    pub value_set: Option<String>,
}

/// FHIR Schema-based model provider
#[derive(Debug, Clone)]
pub struct FhirSchemaProvider {
    schema: Arc<FhirSchema>,
    version: FhirVersion,
    type_cache: Arc<RwLock<HashMap<String, TypeInfo>>>,
}

impl FhirSchemaProvider {
    /// Create a new provider from a schema
    pub fn new(schema: FhirSchema, version: FhirVersion) -> Self {
        Self {
            schema: Arc::new(schema),
            version,
            type_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load schema from URL
    #[cfg(feature = "async-schema")]
    pub async fn from_url(url: &str) -> Result<Self> {
        use reqwest;

        let response = reqwest::get(url)
            .await
            .map_err(|e| ModelError::schema_load_error(format!("Failed to fetch schema: {}", e)))?;

        let schema_text = response
            .text()
            .await
            .map_err(|e| ModelError::schema_load_error(format!("Failed to read response: {}", e)))?;

        let schema: FhirSchema = serde_json::from_str(&schema_text)
            .map_err(|e| ModelError::schema_load_error(format!("Failed to parse schema: {}", e)))?;

        let version = detect_fhir_version(&schema);

        Ok(Self::new(schema, version))
    }

    /// Load schema from file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        use std::fs;

        let schema_text = fs::read_to_string(path)
            .map_err(|e| ModelError::schema_load_error(format!("Failed to read file: {}", e)))?;

        let schema: FhirSchema = serde_json::from_str(&schema_text)
            .map_err(|e| ModelError::schema_load_error(format!("Failed to parse schema: {}", e)))?;

        let version = detect_fhir_version(&schema);

        Ok(Self::new(schema, version))
    }

    /// Get a type definition
    pub fn get_type_definition(&self, type_name: &str) -> Option<&TypeDefinition> {
        self.schema.definitions.get(type_name)
    }

    /// Convert element definition to TypeInfo
    fn element_to_type_info(&self, element: &ElementDefinition) -> TypeInfo {
        if let Some(types) = &element.types {
            if types.len() == 1 {
                self.type_ref_to_type_info(&types[0])
            } else {
                // Multiple types - create a union
                let type_infos: Vec<TypeInfo> = types
                    .iter()
                    .map(|t| self.type_ref_to_type_info(t))
                    .collect();
                TypeInfo::union(type_infos)
            }
        } else {
            TypeInfo::Any
        }
    }

    /// Convert type reference to TypeInfo
    fn type_ref_to_type_info(&self, type_ref: &TypeReference) -> TypeInfo {
        match type_ref.code.as_str() {
            "boolean" => TypeInfo::Boolean,
            "integer" => TypeInfo::Integer,
            "string" => TypeInfo::String,
            "decimal" => TypeInfo::Decimal,
            "uri" | "url" | "canonical" => TypeInfo::String,
            "base64Binary" => TypeInfo::String,
            "instant" => TypeInfo::DateTime,
            "date" => TypeInfo::Date,
            "dateTime" => TypeInfo::DateTime,
            "time" => TypeInfo::Time,
            "code" | "oid" | "id" | "markdown" => TypeInfo::String,
            "unsignedInt" | "positiveInt" => TypeInfo::Integer,
            "uuid" => TypeInfo::String,
            "Quantity" | "Age" | "Distance" | "Duration" | "Count" | "Money" => TypeInfo::Quantity,
            "BackboneElement" | "Element" => TypeInfo::Any,
            other => TypeInfo::Resource(other.to_string()),
        }
    }
}

impl ModelProvider for FhirSchemaProvider {
    fn get_type_info(&self, type_name: &str) -> Option<TypeInfo> {
        // Check cache first
        if let Some(cached) = self.type_cache.read().get(type_name).cloned() {
            return Some(cached);
        }

        // Look up in schema
        let type_def = self.get_type_definition(type_name)?;
        let type_info = match type_def.kind.as_str() {
            "primitive-type" => match type_name {
                "boolean" => TypeInfo::Boolean,
                "integer" => TypeInfo::Integer,
                "string" => TypeInfo::String,
                "decimal" => TypeInfo::Decimal,
                "date" => TypeInfo::Date,
                "dateTime" | "instant" => TypeInfo::DateTime,
                "time" => TypeInfo::Time,
                _ => TypeInfo::String,
            },
            "resource" | "complex-type" => TypeInfo::Resource(type_name.to_string()),
            _ => TypeInfo::Any,
        };

        // Cache the result
        self.type_cache.write().insert(type_name.to_string(), type_info.clone());

        Some(type_info)
    }

    fn get_property_type(&self, parent_type: &str, property: &str) -> Option<TypeInfo> {
        let type_def = self.get_type_definition(parent_type)?;
        let element = type_def.elements.get(property)?;
        Some(self.element_to_type_info(element))
    }

    fn get_search_params(&self, _resource_type: &str) -> Vec<SearchParameter> {
        // TODO: Implement search parameter extraction from schema
        Vec::new()
    }

    fn is_resource_type(&self, type_name: &str) -> bool {
        self.get_type_definition(type_name)
            .map(|def| def.kind == "resource")
            .unwrap_or(false)
    }

    fn fhir_version(&self) -> FhirVersion {
        self.version
    }

    fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool {
        if child_type == parent_type {
            return true;
        }

        if let Some(type_def) = self.get_type_definition(child_type) {
            if let Some(base) = &type_def.base {
                return self.is_subtype_of(base, parent_type);
            }
        }

        false
    }

    fn get_properties(&self, type_name: &str) -> Vec<(String, TypeInfo)> {
        if let Some(type_def) = self.get_type_definition(type_name) {
            type_def
                .elements
                .iter()
                .map(|(name, element)| {
                    (name.clone(), self.element_to_type_info(element))
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_base_type(&self, type_name: &str) -> Option<String> {
        self.get_type_definition(type_name)
            .and_then(|def| def.base.clone())
    }
}

/// Detect FHIR version from schema
fn detect_fhir_version(schema: &FhirSchema) -> FhirVersion {
    if schema.url.contains("/r5/") || schema.version.contains("5.0") {
        FhirVersion::R5
    } else if schema.url.contains("/r4b/") || schema.version.contains("4.3") {
        FhirVersion::R4B
    } else if schema.url.contains("/r4/") || schema.version.contains("4.0") {
        FhirVersion::R4
    } else {
        // Default to R5
        FhirVersion::R5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_ref_conversion() {
        let schema = FhirSchema {
            url: "test".to_string(),
            version: "5.0.0".to_string(),
            date: "2023-01-01".to_string(),
            definitions: HashMap::new(),
        };

        let provider = FhirSchemaProvider::new(schema, FhirVersion::R5);

        let bool_ref = TypeReference {
            code: "boolean".to_string(),
            target_profiles: None,
        };
        assert_eq!(provider.type_ref_to_type_info(&bool_ref), TypeInfo::Boolean);

        let quantity_ref = TypeReference {
            code: "Quantity".to_string(),
            target_profiles: None,
        };
        assert_eq!(provider.type_ref_to_type_info(&quantity_ref), TypeInfo::Quantity);
    }
}