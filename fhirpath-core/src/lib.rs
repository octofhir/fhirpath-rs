//! # FHIRPath Core Library
//!
//! A high-performance FHIRPath engine implementation in Rust.
//!
//! This library provides a complete implementation of the FHIRPath v2.0.0 specification
//! with a focus on performance, safety, and specification compliance.

pub mod ast;
pub mod engine;
pub mod error;
pub mod evaluator;
pub mod model;
pub mod parser;
pub mod types;


// Re-export commonly used types
pub use error::{FhirPathError, Result};
pub use evaluator::evaluate_expression;
pub use model::FhirPathValue;
pub use engine::FhirPathEngine;
pub use types::FhirTypeRegistry;
