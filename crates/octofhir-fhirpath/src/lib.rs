// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! FHIRPath implementation in Rust
//!
//! A complete implementation of the FHIRPath expression language for FHIR resources
//! with high performance and FHIR compliance.

// Import workspace crates
pub use octofhir_fhirpath_ast as ast;
pub use octofhir_fhirpath_core as core;
pub use octofhir_fhirpath_diagnostics as diagnostics;
pub use octofhir_fhirpath_evaluator as evaluator;
pub use octofhir_fhirpath_parser as parser;
pub use octofhir_fhirpath_registry as registry;

pub mod config;
pub mod convenience;
pub mod engine_factory;
pub mod fhirpath;
pub mod mock_provider;
pub mod utils;

// Primary engine - use this for all new code
pub use octofhir_fhirpath_core::{Collection, FhirPathValue};
pub use octofhir_fhirpath_evaluator::{EvaluationConfig, EvaluationContext, FhirPathEngine};
pub use octofhir_fhirpath_parser::{ParseError, parse_expression as parse};
pub use octofhir_fhirpath_registry::{FunctionRegistry, create_standard_registry};

// Re-export from workspace crates
pub use octofhir_fhirpath_ast::{
    BinaryOpData, BinaryOperator, ConditionalData, ExpressionNode, FunctionCallData, LambdaData,
    LiteralValue, MethodCallData, UnaryOperator,
};
pub use octofhir_fhirpath_core::{EvaluationError, FhirPathError, Result};
pub use octofhir_fhirpath_diagnostics::{
    Diagnostic, DiagnosticBuilder, DiagnosticCode, DiagnosticReporter, DiagnosticSeverity,
};

// Registry re-exports - only what exists
pub use octofhir_fhirpath_registry::{PackageError, RegistryPackageManager};

// Re-export ModelProvider from core
pub use octofhir_fhirpath_core::ModelProvider;

// Re-export from local modules (minimal local integration code)
pub mod value_ext;

// Main API exports
pub use config::{
    FhirPathConfig, FhirPathConfigBuilder, FhirPathEngineConfig, FhirPathEvaluationResult,
    FhirVersion, OutputFormat, PerformanceMetrics, SchemaConfig,
};
pub use engine_factory::{
    AdvancedFhirPathEngine, CliEvaluationResult, CliFhirPathEngine, CliValidationResult,
};
pub use fhirpath::FhirPath;

// Convenience function exports
pub use convenience::{
    evaluate_boolean, evaluate_fhirpath, evaluate_fhirpath_with_analysis,
    evaluate_fhirpath_with_version, get_all_string_values, get_string_value, parse_expression,
    path_exists, validate_fhirpath,
};

// Re-export conversion utilities for easier access
pub use utils::{
    JsonResult, fhir_value_to_serde, from_json, parse_as_fhir_value, parse_json, parse_with_serde,
    reformat_json, serde_to_fhir_value, to_json,
};

// Re-export MockModelProvider for convenience in examples
pub use mock_provider::MockModelProvider;

/// Create a FhirPathEngine with enhanced MockModelProvider for testing
pub async fn create_engine_with_mock_provider()
-> octofhir_fhirpath_core::EvaluationResult<FhirPathEngine> {
    use std::sync::Arc;
    let registry = octofhir_fhirpath_registry::create_standard_registry().await;
    let model_provider = Arc::new(MockModelProvider::default());
    Ok(FhirPathEngine::new(Arc::new(registry), model_provider))
}

// Helper functions for error conversion (since we can't implement orphan traits)

/// Convert EvaluationError to FhirPathError
pub fn evaluation_error_to_fhirpath_error(err: EvaluationError) -> FhirPathError {
    FhirPathError::EvaluationError {
        message: err.to_string(),
        expression: None,
        location: None,
        error_type: None,
    }
}

/// Convert serde_json::Error to FhirPathError
pub fn json_error_to_fhirpath_error(err: serde_json::Error) -> FhirPathError {
    FhirPathError::EvaluationError {
        message: format!("JSON serialization error: {}", err),
        expression: None,
        location: None,
        error_type: Some("JsonError".to_string()),
    }
}
