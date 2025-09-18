//! FHIRPath expression evaluator
//!
//! This module provides the complete FHIRPath evaluation engine with registry-based
//! architecture for operators and functions.

// Core evaluator modules
pub mod context;
pub mod engine;
pub mod environment_variables;
pub mod evaluator;
pub mod function_registry;
pub mod functions;
pub mod metadata_collector;
pub mod operations;
pub mod operator_registry;
pub mod terminologies_variable;

#[cfg(test)]
mod terminologies_variable_integration_test;

// Backward compatibility stub (temporary)
pub mod stub;

// Re-export main types
pub use context::{EvaluationContext, EvaluationContextExt, SystemVariables, VariableStack};
pub use environment_variables::{EnvironmentVariables, EnvironmentVariablesBuilder};
pub use evaluator::{AsyncNodeEvaluator, Evaluator};
pub use function_registry::{
    FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter, FunctionRegistry,
    FunctionSignature, create_basic_function_registry, create_comprehensive_function_registry,
    create_standard_function_registry,
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

// Re-export stub types for backward compatibility
pub use stub::{EvaluationResult, EvaluationResultWithMetadata};
