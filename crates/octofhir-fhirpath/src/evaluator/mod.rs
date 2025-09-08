//! FHIRPath expression evaluator
//!
//! This module provides the comprehensive evaluation engine for FHIRPath expressions with
//! multi-method support, performance metrics, AST caching, and async model provider integration.

// Core evaluation modules
pub mod cache;
pub mod config;
pub mod context;
pub mod engine;
pub mod metrics;
pub mod scoping;

// New modular evaluator architecture
pub mod collections;
pub mod composite;
pub mod core;
pub mod functions;
pub mod lambdas;
pub mod metadata_collections;
pub mod metadata_core;
pub mod metadata_functions;
pub mod metadata_navigator;
pub mod navigator;
pub mod operators;
pub mod traits;

// Re-export the comprehensive context system
pub use context::{
    BuiltinVariables, EvaluationContext, EvaluationContextBuilder, PropertyDefinition, ServerApi,
    ServerContext, TerminologyService, TypeDefinition, TypeFactory, TypeKind,
};

// Re-export the main engine types
pub use engine::{EvaluationResult, EvaluationWarning};

// Re-export configuration types
pub use config::EngineConfig;

// Re-export metrics types
pub use metrics::{EvaluationMetrics, MetricsCollector, PerformanceLevel};

// Re-export cache types
pub use cache::{CacheEfficiency, CacheMetrics, CacheStats};

// Re-export scoping types
pub use scoping::{
    LambdaContext, LambdaExpression, ScopeId, ScopeInfo, ScopeManager, ScopeType, VariableScope,
};

// Re-export lambda evaluation types (legacy)
// pub use lambda::{LambdaEvaluator, LambdaExpressionEvaluator, SortCriterion};

// Re-export new modular evaluator types
pub use collections::CollectionEvaluatorImpl;
pub use composite::CompositeEvaluator;
pub use core::CoreEvaluator;
pub use functions::FunctionEvaluatorImpl;
pub use lambdas::LambdaEvaluatorImpl;
pub use metadata_collections::{MetadataCollectionEvaluator, collection_ops};
pub use metadata_core::MetadataCoreEvaluator;
pub use metadata_functions::MetadataFunctionEvaluator;
pub use metadata_navigator::MetadataNavigator;
pub use navigator::Navigator;
// Re-export the main engine type
pub use engine::{FhirPathEngine, TypeResolutionStats, create_engine_with_mock_provider};
pub use operators::OperatorEvaluatorImpl;
pub use traits::{
    CollectionEvaluator, ExpressionEvaluator, FunctionEvaluator, LambdaEvaluator,
    MetadataAwareCollectionEvaluator, MetadataAwareEvaluator, MetadataAwareFunctionEvaluator,
    MetadataAwareNavigator, OperatorEvaluator, ValueNavigator,
};
