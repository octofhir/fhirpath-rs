//\! FHIRPath Expression Evaluator
//\!
//\! This module provides clean, focused evaluation functionality for FHIRPath expressions.
//\! It implements both traditional AST interpretation and high-performance bytecode VM execution
//\! with automatic hybrid strategy selection for optimal performance.

#[warn(missing_docs)]
mod context;
mod engine;
mod error;
mod shared_context;

// Essential evaluation functionality - clean and focused
pub use context::{EvaluationContext, VariableScope};
pub use engine::FhirPathEngine;
pub use error::{EvaluationError, EvaluationResult};
pub use shared_context::{
    ContextInheritance, FunctionClosureOptimizer, SharedContextBuilder, SharedEvaluationContext,
};
