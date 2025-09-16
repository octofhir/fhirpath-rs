//! Core types and abstractions for FHIRPath implementation

pub mod error;
pub mod error_code;
pub mod fhirpath_types;
pub mod model_provider;
pub mod temporal;
pub mod trace;
pub mod types;
pub mod value;
pub mod wrapped;

pub use error::*;
pub use fhirpath_types::*;
pub use temporal::*;
pub use trace::*;
pub use types::{
    CalendarUnit, Collection, CollectionWithMetadata, FhirPathValue, ResultWithMetadata,
    ValueSourceLocation, ValueTypeInfo, WrappedExtension, WrappedPrimitiveElement,
};
// TODO: Wrapped types reimplemented in Phase 1 - old wrapped module removed

// Re-export specific items from model_provider (avoiding utils conflict)
pub use model_provider::ModelProvider;

// Re-export specific items from value (avoiding utils conflict)
pub use value::JsonValueExt;

// Re-export utils modules with qualified names to avoid conflicts
pub use model_provider::utils as model_provider_utils;
pub use value::utils as value_utils;

// TODO: Wrapped types will be reimplemented in new evaluator

// Re-export path types from parent module
pub use crate::path::*;

// Re-export typing types from parent module
pub use crate::typing::*;
