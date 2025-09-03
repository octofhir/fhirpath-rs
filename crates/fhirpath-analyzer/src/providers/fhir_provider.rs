//! # FHIR Provider
//!
//! Abstraction for FHIR schema and model access during analysis.

use async_trait::async_trait;
use octofhir_fhirschema::prelude::FhirSchema;

/// FHIR provider trait for schema access
#[async_trait]
pub trait FhirProvider: Send + Sync {
    /// Get FHIR schema for the specified version
    async fn get_schema(&self, version: &str) -> Result<&FhirSchema, FhirProviderError>;
    
    /// Check if a resource type exists
    async fn has_resource_type(&self, type_name: &str) -> Result<bool, FhirProviderError>;
    
    /// Get property information for a type
    async fn get_property_info(
        &self,
        type_name: &str,
        property_name: &str,
    ) -> Result<Option<PropertyInfo>, FhirProviderError>;
}

/// Property information
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub type_name: String,
    pub cardinality: (usize, Option<usize>),
    pub is_choice_type: bool,
}

/// FHIR provider errors
#[derive(Debug, thiserror::Error)]
pub enum FhirProviderError {
    #[error("Schema not found for version: {version}")]
    SchemaNotFound { version: String },
    
    #[error("Provider error: {message}")]
    ProviderError { message: String },
}