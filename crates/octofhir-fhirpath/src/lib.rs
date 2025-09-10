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
//! - [`analyzer`] - Static analysis and validation
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
//! let registry = octofhir_fhirpath::create_standard_registry().await;
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
pub mod registry;

// Support modules
pub mod analyzer;
pub mod diagnostics;
pub mod path;
pub mod typing;
pub mod wrapped;

// Re-export core types for convenience
pub use crate::core::{Collection, FhirPathError, FhirPathValue, ModelProvider, Result};
pub use octofhir_fhir_model::EmptyModelProvider as MockModelProvider;

// Re-export wrapped types for rich metadata support
pub use crate::wrapped::{
    IntoPlain, IntoWrapped, ValueMetadata, WrappedCollection, WrappedValue, collection_utils,
};

// Re-export path types for canonical path representation
pub use crate::path::{CanonicalPath, PathBuilder, PathParseError, PathSegment, path_utils};

// Re-export typing types for type resolution
pub use crate::typing::{
    TypeResolutionContext, TypeResolver, TypeResolverFactory, is_primitive_type, type_utils,
};

// Re-export main engine types
pub use crate::evaluator::{
    BuiltinVariables,
    CacheEfficiency,
    CacheMetrics,
    CacheStats,
    // Composite evaluator types
    CompositeEvaluator,
    EngineConfig,
    // Context system
    EvaluationContext,
    EvaluationContextBuilder,
    EvaluationMetrics,
    // Performance and caching types
    EvaluationResult,
    EvaluationWarning,
    FhirPathEngine,
    MetadataAwareCollectionEvaluator,
    MetadataAwareEvaluator,
    MetadataAwareFunctionEvaluator,
    MetadataAwareNavigator,
    // Collection types
    MetadataCollectionEvaluator,
    // Evaluator types
    MetadataCoreEvaluator,
    MetadataFunctionEvaluator,
    // Navigator types
    MetadataNavigator,
    MetricsCollector,
    PerformanceLevel,
    PropertyDefinition,
    ServerApi,
    ServerContext,
    TerminologyService,
    TypeDefinition,
    TypeFactory,
    TypeKind,
    TypeResolutionStats,
    collection_ops,
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
pub use crate::registry::{
    AsyncFunction,
    // Terminology types and providers
    Coding,
    ConceptDesignation,
    ConceptDetails,
    ConceptProperty,
    ConceptTranslation,
    DefaultTerminologyProvider,
    FunctionCategory,
    FunctionContext,
    FunctionMetadata,
    FunctionRegistry,
    MockTerminologyProvider,
    ParameterMetadata,
    PropertyValue,
    SyncFunction,
    TerminologyProvider,
    TerminologyUtils,
    builder::FunctionBuilder,
    create_standard_registry,
    dispatcher::FunctionDispatcher,
};

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

// Note: MockModelProvider will be provided by external model provider dependencies

/// Create a FhirPathEngine with MockModelProvider for testing and development
///
/// This is a convenience function for getting started quickly with FHIRPath
/// evaluation when you don't need full FHIR schema support.
///
/// ```rust,no_run
/// use octofhir_fhirpath::create_engine_with_mock_provider;
///
/// # async fn example() -> octofhir_fhirpath::Result<()> {
/// let engine = create_engine_with_mock_provider().await?;
///
/// let result = engine.evaluate("1 + 2", &octofhir_fhirpath::Collection::empty()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_engine_with_mock_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    let registry = Arc::new(create_standard_registry().await);
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
    let mut engine = create_engine_with_mock_provider().await?;
    let eval_context = EvaluationContext::from_value(context.clone());
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
