//! # OctoFHIR FHIRPath Implementation

#![allow(missing_docs)]
//!
//! A high-performance, spec-compliant implementation of the FHIRPath expression language
//! for FHIR resources. This implementation consolidates all functionality into a single
//! crate for easier maintenance while providing a clean, modular architecture.
//!
//! ## Overview
//!
//! FHIRPath is a path-based navigation and extraction language designed for FHIR resources.
//! This implementation provides:
//!
//! - **Complete FHIRPath 3.0 specification compliance**
//! - **High-performance evaluation engine**
//! - **Comprehensive error handling and diagnostics**
//! - **Integration with FHIR model providers**
//! - **Rich type system with precision temporal types**
//! - **UCUM unit support for quantities**
//! - **Comprehensive terminology provider system with tx.fhir.org integration**
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//!
//! - [`ast`] - Abstract syntax tree definitions
//! - [`core`] - Core types, errors, and value system
//! - [`parser`] - Expression parsing with nom
//! - [`evaluator`] - Expression evaluation engine
//! - [`registry`] - Function and operator registry
//! - [`diagnostics`] - Error reporting and diagnostics
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use octofhir_fhirpath::{FhirPathEngine, Collection};
//! use octofhir_fhir_model::EmptyModelProvider;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a FHIRPath engine with a model provider
//! let registry = octofhir_fhirpath::create_empty_registry();
//! let model_provider = Arc::new(EmptyModelProvider);
//! let engine = FhirPathEngine::new(Arc::new(registry), model_provider);
//!
//! // Parse and evaluate an expression
//! let expression = "Patient.name.family";
//! let result = engine.evaluate(expression, &Collection::empty()).await?;
//!
//! println!("Result: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Parser**: Complete FHIRPath syntax support using nom parser combinators
//! - **Type System**: Rich type system with temporal precision and UCUM quantities
//! - **Functions**: Comprehensive function library covering all FHIRPath operations
//! - **Error Handling**: Detailed error messages with source location tracking
//! - **Performance**: Optimized evaluation with efficient collection handling
//! - **ModelProvider**: Pluggable model provider system for different FHIR versions

#![deny(unsafe_code)]

// Core modules
pub mod ast;
pub mod core;

// Engine modules
pub mod evaluator;
pub mod parser;
// TODO: Registry will be reimplemented in new evaluator

// Support modules
pub mod diagnostics;
pub mod path;
pub mod typing;

// Re-export core types for convenience
pub use crate::core::model_provider::EmptyModelProvider;
pub use crate::core::{Collection, FhirPathError, FhirPathValue, ModelProvider, Result};


// Re-export path types for canonical path representation
pub use crate::path::{CanonicalPath, PathBuilder, PathParseError, PathSegment, path_utils};

// Re-export typing types for type resolution
pub use crate::typing::{
    TypeResolutionContext, TypeResolver, TypeResolverFactory, is_primitive_type, type_utils,
};

// Re-export main engine types (minimal for stub)
pub use crate::evaluator::{
    EvaluationContext,
    EvaluationResult,
    EvaluationResultWithMetadata,
    FhirPathEngine,
};
// Parser API exports - New unified API with clean naming
pub use crate::parser::{
    ParseResult,
    ParserConfig,
    ParserUseCase,

    // Types
    ParsingMode,
    get_errors,
    get_warnings,

    // Convenience functions
    is_valid,
    // Main parsing functions with clean names
    parse,
    parse_ast,
    parse_ast_with_mode,
    // Backward compatibility (legacy names)
    parse_expression,
    parse_multiple,
    parse_multiple_ast,
    parse_with_analysis,
    parse_with_config,

    parse_with_mode,
    recommend_mode,
    validate,
};
// Re-export the real function registry from evaluator
pub use crate::evaluator::FunctionRegistry;

/// Create empty registry for compatibility during Phase 1
pub fn create_empty_registry() -> FunctionRegistry {
    FunctionRegistry::new()
}

// Re-export AST types
pub use crate::ast::{BinaryOperator, ExpressionNode, LiteralValue, UnaryOperator};

// Re-export diagnostic types
pub use crate::diagnostics::{
    // New Ariadne-based diagnostic types
    AriadneDiagnostic,
    ColorScheme,
    Diagnostic,
    DiagnosticCode,
    DiagnosticEngine,
    DiagnosticFormatter,
    DiagnosticSeverity,
    RelatedDiagnostic,
    SourceInfo,
    SourceManager,
};

// Note: EmptyModelProvider is provided by external model provider dependencies for basic testing

/// Create a FhirPathEngine with EmptyModelProvider for testing and development
///
/// This is a convenience function for getting started quickly with FHIRPath
/// evaluation when you don't need full FHIR schema support.
///
/// ```rust,no_run
/// use octofhir_fhirpath::create_engine_with_empty_provider;
///
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// let engine = create_engine_with_empty_provider().await?;
///
/// let result = engine.evaluate("1 + 2", &octofhir_fhirpath::Collection::empty()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_engine_with_empty_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    // TODO: Replace with real registry when implemented
    let registry = Arc::new(create_empty_registry());
    let model_provider = Arc::new(EmptyModelProvider);

    FhirPathEngine::new(registry, model_provider).await
}

/// Main evaluation function for simple use cases
///
/// This function provides a simple interface for evaluating FHIRPath expressions
/// when you don't need the full engine configuration options.
///
/// ```rust,no_run
/// use octofhir_fhirpath::{evaluate, Collection};
/// use serde_json::json;
///
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// let patient = json!({
///     "resourceType": "Patient",
///     "name": [{"family": "Doe", "given": ["John"]}]
/// });
///
/// let context = Collection::single(octofhir_fhirpath::FhirPathValue::resource(patient));
/// let result = evaluate("Patient.name.family", &context).await?;
/// # Ok(())
/// # }
/// ```
pub async fn evaluate(expression: &str, context: &FhirPathValue) -> Result<FhirPathValue> {
    let engine = create_engine_with_empty_provider().await?;

    // Convert FhirPathValue to Collection
    let collection = match context {
        FhirPathValue::Collection(collection) => collection.clone(),
        FhirPathValue::Empty => Collection::empty(),
        single_value => Collection::single(single_value.clone()),
    };

    let eval_context = EvaluationContext::new(
        collection,
        engine.get_model_provider(),
        None, // TODO: Convert TerminologyService to TerminologyProvider
    )
    .await;

    let result = engine.evaluate(expression, &eval_context).await?;
    // Convert Collection to FhirPathValue for convenience function
    let values = result.value.into_vec();
    let fhir_value = match values.len() {
        0 => FhirPathValue::Empty,
        1 => values.into_iter().next().unwrap(),
        _ => FhirPathValue::Collection(Collection::from_values(values)),
    };
    Ok(fhir_value)
}

// Version information
/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The FHIRPath specification version this implementation targets
pub const FHIRPATH_VERSION: &str = "3.0.0";
