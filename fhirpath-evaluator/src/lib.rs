// FHIRPath Expression Evaluator
//
// This crate provides the main evaluation functionality for FHIRPath expressions.
// It implements the evaluation engine following the FHIRPath specification.

#[warn(missing_docs)]

mod context;
mod engine;
mod error;

pub use context::EvaluationContext;
pub use engine::FhirPathEngine;
pub use error::{EvaluationError, EvaluationResult};
