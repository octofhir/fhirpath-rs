//! FHIRPath implementation in Rust
//!
//! A complete implementation of FHIRPath expression language for FHIR resources.

pub mod ast;
pub mod diagnostics;
pub mod evaluator;
pub mod model;
pub mod parser;
pub mod registry;

// Re-export main types
pub use evaluator::{EvaluationContext, FhirPathEngine};
pub use model::FhirPathValue;
pub use parser::{ParseError, parse_expression as parse};
pub use registry::FunctionRegistry;

// Re-export from fhirpath-core
pub mod engine;
pub mod error;
pub mod types;
pub mod value_ext;

pub use engine::*;
pub use error::*;
pub use types::*;
