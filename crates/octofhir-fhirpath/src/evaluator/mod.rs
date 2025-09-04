//! FHIRPath expression evaluator
//!
//! This module provides the comprehensive evaluation engine for FHIRPath expressions with
//! multi-method support, performance metrics, AST caching, and async model provider integration.

// Core evaluation modules
pub mod context;
pub mod engine;
pub mod config;
pub mod metrics;
pub mod cache;
pub mod scoping;
pub mod lambda;

// Re-export the comprehensive context system
pub use context::{
    EvaluationContext, EvaluationContextBuilder, BuiltinVariables, ServerContext,
    TypeFactory, TypeDefinition, TypeKind, PropertyDefinition,
    TerminologyService, ServerApi,
};

// Re-export the main engine types
pub use engine::{
    FhirPathEngine, EvaluationResult, EvaluationWarning,
};

// Re-export configuration types
pub use config::EngineConfig;

// Re-export metrics types
pub use metrics::{
    EvaluationMetrics, MetricsCollector, PerformanceLevel,
};

// Re-export cache types
pub use cache::{
    CacheStats, CacheMetrics, CacheEfficiency,
};

// Re-export scoping types
pub use scoping::{
    ScopeManager, VariableScope, ScopeType, LambdaExpression, LambdaContext, 
    ScopeId, ScopeInfo,
};

// Re-export lambda evaluation types
pub use lambda::{
    LambdaEvaluator, LambdaExpressionEvaluator, SortCriterion,
};
