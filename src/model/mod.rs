//! Data model and value types for FHIRPath expressions
//!
//! This crate provides the core data types used in FHIRPath evaluation,
//! including the value model and FHIR resource wrappers.

#![warn(missing_docs)]

pub mod error;
pub mod json_arc;
pub mod lazy;
pub mod provider;
pub mod quantity;
pub mod resource;
pub mod string_intern;
pub mod types;
pub mod value;

pub use error::{ModelError, Result};
pub use json_arc::{ArcJsonValue, ArrayView};
pub use lazy::{LazyCollection, LazyIterator, ToLazy};
pub use provider::{FhirVersion, ModelProvider};
pub use quantity::Quantity;
pub use resource::FhirResource;
pub use string_intern::{InternerStats, global_interner_stats, intern_string};
pub use types::TypeInfo;
pub use value::{Collection, FhirPathValue, ValueRef};

// Re-export FHIR Schema types when async-schema feature is enabled
#[cfg(feature = "async-schema")]
pub mod schema;

#[cfg(feature = "async-schema")]
pub use schema::{FhirSchema, FhirSchemaProvider};
