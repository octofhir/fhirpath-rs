//! FHIRPath expression evaluator
//!
//! Clean evaluator architecture for FHIRPath expressions

// Core evaluation modules
pub mod cache;
pub mod config;
pub mod context_manager;
pub mod metrics;

// New clean evaluator architecture
pub mod evaluator;
pub mod engine;
pub mod lambda_functions;

// Support modules that remain useful
pub mod choice_types;
pub mod choice_types_performance_tests;
pub mod property_navigator;
pub mod property_navigator_choice_tests;
pub mod real_fhir_data_tests;

// Re-export the context system
pub use context_manager::{ContextManager, EvaluationContext, EvaluationContextBuilder};

// Re-export the main engine types
pub use engine::{EvaluationResult, EvaluationWarning};

// Re-export configuration types
pub use config::EngineConfig;

// Re-export metrics types
pub use metrics::{EvaluationMetrics, MetricsCollector, PerformanceLevel};

// Re-export cache types
pub use cache::{CacheEfficiency, CacheMetrics, CacheStats};

// Re-export new clean evaluator
pub use evaluator::{Evaluator, FhirPathEvaluator};
pub use engine::{FhirPathEngine, create_engine_with_mock_provider};
pub use lambda_functions::{
    LambdaFunctionEvaluator, where_function_metadata, aggregate_function_metadata, define_variable_function_metadata
};

// Re-export choice type support
pub use choice_types::{ChoiceTypeDetector, ChoiceProperty, ChoiceResolution};
pub use property_navigator::PropertyNavigator;