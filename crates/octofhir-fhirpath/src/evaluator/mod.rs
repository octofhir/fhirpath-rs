//! FHIRPath expression evaluator
//!
//! This module provides the complete FHIRPath evaluation engine with registry-based
//! architecture for operators and functions.

// Core evaluator modules
pub mod evaluator;
pub mod context;
pub mod operator_registry;
pub mod function_registry;
pub mod engine;
pub mod operations;

// Backward compatibility stub (temporary)
pub mod stub;

// Re-export main types
pub use evaluator::{Evaluator, AsyncNodeEvaluator};
pub use context::{EvaluationContext, VariableStack, SystemVariables, EvaluationContextExt};
pub use operator_registry::{
    OperatorRegistry, OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity, create_default_operator_registry,
};
pub use function_registry::{
    FunctionRegistry, FunctionEvaluator, FunctionMetadata, FunctionSignature,
    FunctionParameter, FunctionCategory, create_default_function_registry,
    create_basic_function_registry,
};

// Re-export engine types
pub use engine::{FhirPathEngine, create_engine_with_mock_provider};

// Re-export stub types for backward compatibility
pub use stub::{
    EvaluationResult,
    EvaluationResultWithMetadata,
};