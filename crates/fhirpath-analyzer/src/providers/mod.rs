//! # Providers
//!
//! Provider abstractions for FHIR schema, function registry, and caching.

pub mod fhir_provider;
pub mod function_provider;
pub mod cache_provider;

// Re-export commonly used providers
pub use fhir_provider::FhirProvider;
pub use function_provider::FunctionProvider;
pub use cache_provider::CacheProvider;

// Mock provider for testing
pub use mock_provider::MockFhirProvider;

mod mock_provider {
    use super::fhir_provider::{FhirProvider, FhirProviderError, PropertyInfo};
    use async_trait::async_trait;
    use octofhir_fhirschema::prelude::FhirSchema;
    
    /// Simple mock FHIR provider for testing
    pub struct MockFhirProvider;
    
    impl MockFhirProvider {
        pub fn new() -> Self {
            Self
        }
    }
    
    #[async_trait]
    impl FhirProvider for MockFhirProvider {
        async fn get_schema(&self, _version: &str) -> Result<&FhirSchema, FhirProviderError> {
            Err(FhirProviderError::SchemaNotFound { 
                version: "mock".to_string() 
            })
        }
        
        async fn has_resource_type(&self, type_name: &str) -> Result<bool, FhirProviderError> {
            // Mock some common resource types
            Ok(matches!(type_name, "Patient" | "Observation" | "Bundle"))
        }
        
        async fn get_property_info(
            &self,
            type_name: &str,
            property_name: &str,
        ) -> Result<Option<PropertyInfo>, FhirProviderError> {
            // Mock some basic properties
            match (type_name, property_name) {
                ("Patient", "name") => Ok(Some(PropertyInfo {
                    name: "name".to_string(),
                    type_name: "HumanName".to_string(),
                    cardinality: (0, None),
                    is_choice_type: false,
                })),
                ("Patient", "birthDate") => Ok(Some(PropertyInfo {
                    name: "birthDate".to_string(),
                    type_name: "date".to_string(),
                    cardinality: (0, Some(1)),
                    is_choice_type: false,
                })),
                _ => Ok(None),
            }
        }
    }
}