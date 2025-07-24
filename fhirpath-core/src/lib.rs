//! # FHIRPath Core Library
//!
//! A high-performance FHIRPath engine implementation in Rust.
//!
//! This library provides a complete implementation of the FHIRPath v2.0.0 specification
//! with a focus on performance, safety, and specification compliance.

// Re-export AST from fhirpath-ast crate
pub use fhirpath_ast as ast;
pub mod engine;
pub mod error;
pub mod evaluator;
pub mod value_ext;
pub mod parser;
pub mod registry;
pub mod types;

// Re-export commonly used types
pub use error::{FhirPathError, Result};
pub use evaluator::{evaluate_expression, EvaluationContext};
pub use fhirpath_model::{TypeInfo};
pub use value_ext::{FhirPathValue, FhirResource};
pub use engine::FhirPathEngine;
pub use types::FhirTypeRegistry;
pub use registry::{FhirPathFunction, FhirPathOperator, FunctionRegistry, OperatorRegistry};
