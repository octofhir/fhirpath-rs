//! FHIRPath expression evaluator
//!
//! This module provides the complete FHIRPath evaluation engine with registry-based
//! architecture for operators and functions.

// Core evaluator modules
pub mod context;
mod element_type_cache;
#[cfg(test)]
mod element_type_cache_tests;
pub mod engine;
pub mod environment_variables;
#[allow(clippy::module_inception)]
pub mod evaluator;
pub mod factory_variable;
pub mod function_registry;
pub mod functions;
pub mod lambda_hoisting;
pub mod metadata_collector;
mod model_prepared;
pub mod operations;
pub mod operator_registry;
pub mod plan;
pub mod prepared;
pub mod quantity_utils;
pub mod result;
pub mod server_variable;
pub mod terminologies_variable;
mod vm;

#[cfg(test)]
mod terminologies_variable_integration_test;

// Note: stub module removed - now using complete evaluation engine

// Re-export main types
pub use context::EvaluationContext;
pub use environment_variables::{EnvironmentVariables, EnvironmentVariablesBuilder};
pub use evaluator::{AsyncNodeEvaluator, Evaluator};
pub use function_registry::{
    FunctionCategory, FunctionMetadata, FunctionParameter, FunctionRegistry, FunctionSignature,
    create_function_registry,
};
pub use metadata_collector::{
    CacheStats, EvaluationSummary, MetadataCollector, NodeEvaluationInfo, PerformanceMetrics,
    SourceLocation, TraceEvent, TypeResolutionInfo, TypeResolutionSource,
};
pub use operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorRegistry,
    OperatorSignature, create_standard_operator_registry,
};

// Re-export engine types
pub use engine::{FhirPathEngine, create_engine_with_mock_provider};

// Re-export result types
pub use result::{EvaluationResult, EvaluationResultWithMetadata};
mod value_set;
