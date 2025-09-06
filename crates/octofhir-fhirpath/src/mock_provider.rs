//! MockModelProvider - Basic mock for type checker testing
//!
//! This provides minimal ModelProvider implementation for testing.
//! In production, we use FhirSchemaModelProvider for complete FHIR schema support.

// Re-export EmptyModelProvider as MockModelProvider for testing
pub use octofhir_fhir_model::EmptyModelProvider as MockModelProvider;
