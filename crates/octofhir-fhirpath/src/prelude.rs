//! Convenient re-exports for common FHIRPath usage.
//!
//! This module provides a single import for the most commonly used types
//! when working with FHIRPath expressions.
//!
//! # Example
//!
//! ```rust,no_run
//! use octofhir_fhirpath::prelude::*;
//! use octofhir_fhir_model::EmptyModelProvider;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<()> {
//! // Create engine
//! let registry = Arc::new(octofhir_fhirpath::create_function_registry());
//! let model_provider = Arc::new(EmptyModelProvider);
//! let engine = FhirPathEngine::new(registry, model_provider.clone()).await?;
//!
//! // Create context and evaluate
//! let context = EvaluationContext::new(Collection::empty(), model_provider, None, None, None);
//! let result = engine.evaluate("1 + 2", &context).await?;
//! # Ok(())
//! # }
//! ```

// Core types
pub use crate::core::{Collection, FhirPathError, FhirPathValue, Result};

// Engine types
pub use crate::evaluator::{EvaluationContext, EvaluationResult, FhirPathEngine, FunctionRegistry};

// Parser types
pub use crate::parser::{ParseResult, parse, parse_ast};

// AST types (commonly needed for advanced usage)
pub use crate::ast::ExpressionNode;

// Diagnostic types
pub use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
