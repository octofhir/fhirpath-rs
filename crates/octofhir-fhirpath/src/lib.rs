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
//! A complete implementation of the FHIRPath expression language for FHIR resources.

// Import workspace crates
pub use octofhir_fhirpath_analyzer as analyzer;
pub use octofhir_fhirpath_ast as ast;
pub use octofhir_fhirpath_core as core;
pub use octofhir_fhirpath_diagnostics as diagnostics;
pub use octofhir_fhirpath_evaluator as evaluator;
pub use octofhir_fhirpath_model as model;
pub use octofhir_fhirpath_parser as parser;
pub use octofhir_fhirpath_registry as registry;

pub mod utils;

// CLI module (includes server functionality) - optional
#[cfg(feature = "cli")]
pub mod cli;

// Primary engine - use this for all new code
pub use octofhir_fhirpath_evaluator::{EvaluationConfig, EvaluationContext, FhirPathEngine};
pub use octofhir_fhirpath_model::{
    FhirPathValue, JsonValue, SmartCollection, SmartCollectionBuilder,
};
pub use octofhir_fhirpath_parser::{ParseError, parse_expression as parse};
pub use octofhir_fhirpath_registry::{FunctionRegistry, create_standard_registry};

// Re-export from workspace crates
pub use octofhir_fhirpath_ast::{
    BinaryOpData, BinaryOperator, ConditionalData, ExpressionNode, FunctionCallData, LambdaData,
    LiteralValue, MethodCallData, UnaryOperator,
};
pub use octofhir_fhirpath_core::{FhirPathError, Result};
pub use octofhir_fhirpath_diagnostics::{
    Diagnostic, DiagnosticBuilder, DiagnosticCode, DiagnosticReporter, DiagnosticSeverity,
};

// Re-export ModelProvider from fhir-model-rs
pub use octofhir_fhirpath_model::ModelProvider;
pub use octofhir_fhirpath_model::fhir_model;

// Re-export from local modules (minimal local integration code)
pub mod value_ext;

// Re-export conversion utilities for easier access
pub use utils::{
    JsonResult, fhir_value_to_serde, from_json, parse_as_fhir_value, parse_json, parse_with_serde,
    reformat_json, serde_to_fhir_value, to_json,
};

// Re-export analyzer components
pub use octofhir_fhirpath_analyzer::{
    AnalysisContext, AnalysisResult, AnalysisSettings, AnalyzerConfig, FhirPathAnalyzer,
    SemanticInfo, ValidationError as AnalysisValidationError,
};

// Re-export MockModelProvider for convenience in examples
pub use octofhir_fhirpath_model::mock_provider::MockModelProvider;

/// Extended FHIRPath engine with optional analysis capabilities
pub struct FhirPathEngineWithAnalyzer {
    /// Core engine
    pub engine: FhirPathEngine,
    /// Optional analyzer
    pub analyzer: Option<FhirPathAnalyzer>,
    /// Model provider (kept as Arc for analyzer)
    #[allow(dead_code)]
    model_provider: std::sync::Arc<dyn ModelProvider>,
}

impl FhirPathEngineWithAnalyzer {
    /// Create engine without analyzer (maintains existing behavior)
    pub async fn new(
        model_provider: Box<dyn ModelProvider>,
    ) -> octofhir_fhirpath_core::EvaluationResult<Self> {
        let arc_provider: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::from(model_provider);
        let engine = FhirPathEngine::with_model_provider(arc_provider.clone()).await?;

        Ok(Self {
            engine,
            analyzer: None,
            model_provider: arc_provider,
        })
    }

    /// Create engine with analyzer enabled
    pub async fn with_analyzer(
        model_provider: Box<dyn ModelProvider>,
    ) -> octofhir_fhirpath_core::EvaluationResult<Self> {
        let arc_provider: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::from(model_provider);
        let analyzer = FhirPathAnalyzer::new(arc_provider.clone());
        let engine = FhirPathEngine::with_model_provider(arc_provider.clone()).await?;

        Ok(Self {
            engine,
            analyzer: Some(analyzer),
            model_provider: arc_provider,
        })
    }

    /// Create with custom analyzer configuration
    pub async fn with_analyzer_config(
        model_provider: Box<dyn ModelProvider>,
        analyzer_config: AnalyzerConfig,
    ) -> octofhir_fhirpath_core::EvaluationResult<Self> {
        let arc_provider: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::from(model_provider);
        let analyzer = FhirPathAnalyzer::with_config(arc_provider.clone(), analyzer_config);
        let engine = FhirPathEngine::with_model_provider(arc_provider.clone()).await?;

        Ok(Self {
            engine,
            analyzer: Some(analyzer),
            model_provider: arc_provider,
        })
    }

    /// Create engine with analyzer and function registry
    pub async fn with_full_analysis(
        model_provider: Box<dyn ModelProvider>,
        function_registry: std::sync::Arc<FunctionRegistry>,
    ) -> octofhir_fhirpath_core::EvaluationResult<Self> {
        let arc_provider: std::sync::Arc<dyn ModelProvider> = std::sync::Arc::from(model_provider);
        let analyzer =
            FhirPathAnalyzer::with_function_registry(arc_provider.clone(), function_registry);
        let engine = FhirPathEngine::with_model_provider(arc_provider.clone()).await?;

        Ok(Self {
            engine,
            analyzer: Some(analyzer),
            model_provider: arc_provider,
        })
    }

    /// Evaluate expression (same as existing engine)
    pub async fn evaluate(
        &self,
        expression: &str,
        context: serde_json::Value,
    ) -> octofhir_fhirpath_core::EvaluationResult<FhirPathValue> {
        self.engine.evaluate(expression, context).await
    }

    /// Evaluate with analysis (new capability)
    pub async fn evaluate_with_analysis(
        &self,
        expression: &str,
        context: serde_json::Value,
    ) -> octofhir_fhirpath_core::EvaluationResult<(FhirPathValue, Option<AnalysisResult>)> {
        // Perform analysis if analyzer is available
        let analysis = if let Some(analyzer) = &self.analyzer {
            Some(analyzer.analyze(expression).await.map_err(|e| {
                octofhir_fhirpath_core::EvaluationError::InvalidOperation {
                    message: format!("Analysis failed: {e}"),
                }
            })?)
        } else {
            None
        };

        // Evaluate expression normally
        let result = self.engine.evaluate(expression, context).await?;

        Ok((result, analysis))
    }

    /// Pre-validate expression without evaluation
    pub async fn validate_expression(
        &self,
        expression: &str,
    ) -> octofhir_fhirpath_core::EvaluationResult<Vec<AnalysisValidationError>> {
        if let Some(analyzer) = &self.analyzer {
            analyzer.validate(expression).await.map_err(|e| {
                octofhir_fhirpath_core::EvaluationError::InvalidOperation {
                    message: format!("Validation failed: {e}"),
                }
            })
        } else {
            Ok(vec![]) // No validation without analyzer
        }
    }

    /// Get analysis information without evaluation
    pub async fn analyze_expression(
        &self,
        expression: &str,
    ) -> octofhir_fhirpath_core::EvaluationResult<Option<AnalysisResult>> {
        if let Some(analyzer) = &self.analyzer {
            Ok(Some(analyzer.analyze(expression).await.map_err(|e| {
                octofhir_fhirpath_core::EvaluationError::InvalidOperation {
                    message: format!("Analysis failed: {e}"),
                }
            })?))
        } else {
            Ok(None)
        }
    }
}

// Delegate standard engine methods
impl std::ops::Deref for FhirPathEngineWithAnalyzer {
    type Target = FhirPathEngine;

    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}

impl std::ops::DerefMut for FhirPathEngineWithAnalyzer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}
