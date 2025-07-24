//! Data model and value types for FHIRPath expressions
//!
//! This crate provides the core data types used in FHIRPath evaluation,
//! including the value model and FHIR resource wrappers.

#![warn(missing_docs)]

pub mod error;
pub mod value;
pub mod resource;
pub mod quantity;
pub mod types;
pub mod provider;

pub use error::{ModelError, Result};
pub use value::{FhirPathValue, Collection};
pub use resource::FhirResource;
pub use quantity::Quantity;
pub use types::TypeInfo;
pub use provider::{ModelProvider, FhirVersion};

// Re-export FHIR Schema types when async-schema feature is enabled
#[cfg(feature = "async-schema")]
pub mod schema;

#[cfg(feature = "async-schema")]
pub use schema::{FhirSchema, FhirSchemaProvider};