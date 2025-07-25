//! # FHIRPath Core Library  
//!
//! A high-performance FHIRPath engine implementation in Rust.
//!
//! This library provides a complete implementation of the FHIRPath v2.0.0 specification
//! with a focus on performance, safety, and specification compliance.

// Re-export from component crates
pub use fhirpath_ast as ast;
pub use fhirpath_evaluator as evaluator;
pub use fhirpath_registry as registry;

pub mod engine;
pub mod error;
pub mod value_ext;
pub mod parser;
pub mod types;
pub mod debug_test;
pub mod lambda_test;

// Re-export commonly used types
pub use error::{FhirPathError, Result};
pub use fhirpath_model::{TypeInfo, FhirPathValue};
pub use value_ext::FhirResource;
pub use engine::FhirPathEngine;
pub use types::FhirTypeRegistry;

// Re-export registry types
pub use fhirpath_registry::{FhirPathFunction, FhirPathOperator, FunctionRegistry, OperatorRegistry};

// Re-export evaluator types  
pub use fhirpath_evaluator::{EvaluationContext, EvaluationResult};
