//! FHIRPath expression evaluator
//!
//! This module provides the comprehensive evaluation engine for FHIRPath expressions with
//! multi-method support, performance metrics, AST caching, and async model provider integration.

// Core evaluation modules
pub mod cache;
pub mod config;
pub mod context;
pub mod engine;
pub mod lambda;
pub mod metrics;
pub mod scoping;

// Re-export the comprehensive context system
pub use context::{
    BuiltinVariables, EvaluationContext, EvaluationContextBuilder, PropertyDefinition, ServerApi,
    ServerContext, TerminologyService, TypeDefinition, TypeFactory, TypeKind,
};

// Re-export the main engine types
pub use engine::{EvaluationResult, EvaluationWarning, FhirPathEngine};

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

// Re-export lambda evaluation types
pub use lambda::{LambdaEvaluator, LambdaExpressionEvaluator, SortCriterion};
