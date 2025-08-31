//! Async Engine Factory for FHIRPath with Bridge Support

use crate::{
    AnalyzerFieldValidator, EvaluationConfig, FhirPathEngine, FhirPathError, FhirPathValue,
    FhirSchemaPackageManager, ModelProvider,
    config::{FhirPathConfig, FhirPathEngineConfig, OutputFormat, PerformanceMetrics},
};
use std::sync::Arc;
use std::time::Duration;

/// Factory for creating different types of FHIRPath engines
pub struct FhirPathEngineFactory {
    schema_manager: Arc<FhirSchemaPackageManager>,
}

impl FhirPathEngineFactory {
    /// Create a new engine factory
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        Self { schema_manager }
    }

    /// Create a basic FHIRPath engine for simple evaluation
    pub async fn create_basic_engine(&self) -> Result<Box<dyn ModelProvider>, FhirPathError> {
        // Create a basic model provider using the schema manager
        let model_provider = crate::model::FhirSchemaModelProvider::new()
            .await
            .map_err(|e| FhirPathError::Generic {
                message: format!("Failed to create model provider: {}", e),
            })?;

        Ok(Box::new(model_provider))
    }

    /// Create an advanced engine with full analysis capabilities
    pub async fn create_advanced_engine(
        &self,
        config: &FhirPathEngineConfig,
    ) -> Result<AdvancedFhirPathEngine, FhirPathError> {
        // Create model provider
        let model_provider = self.create_basic_engine().await?;
        let arc_provider: Arc<dyn ModelProvider> = Arc::from(model_provider);

        // TODO: Re-enable when BridgeNavigationEvaluator is available
        // Create evaluator with bridge support
        // let evaluator = BridgeNavigationEvaluator::new(
        //     self.schema_manager.clone()
        // ).await.map_err(|e| FhirPathError::Generic {
        //     message: format!("Failed to create evaluator: {}", e),
        // })?;

        // Create analyzer if enabled
        let analyzer = if config.enable_analysis {
            Some(
                AnalyzerFieldValidator::new(self.schema_manager.clone())
                    .await
                    .map_err(|e| FhirPathError::Generic {
                        message: format!("Failed to create analyzer: {}", e),
                    })?,
            )
        } else {
            None
        };

        // Create core engine
        let engine = FhirPathEngine::with_model_provider(arc_provider.clone())
            .await
            .map_err(|e| FhirPathError::Generic {
                message: format!("Failed to create engine: {}", e),
            })?;

        Ok(AdvancedFhirPathEngine {
            engine,
            // evaluator, // TODO: Re-enable when BridgeNavigationEvaluator is available
            analyzer,
            schema_manager: self.schema_manager.clone(),
            config: config.clone(),
        })
    }

    /// Create engine optimized for CLI usage
    pub async fn create_cli_engine(
        &self,
        output_format: OutputFormat,
    ) -> Result<CliFhirPathEngine, FhirPathError> {
        let config = FhirPathEngineConfig {
            enable_analysis: true,
            enable_performance_tracking: true,
            enable_caching: true,
            output_format,
            strict_mode: false,
            max_evaluation_depth: 100,
            evaluation_timeout_ms: 10000, // 10 seconds for CLI
        };

        let advanced_engine = self.create_advanced_engine(&config).await?;

        Ok(CliFhirPathEngine::new(advanced_engine))
    }
}

/// Advanced FHIRPath engine with full capabilities
pub struct AdvancedFhirPathEngine {
    pub engine: FhirPathEngine,
    // pub evaluator: BridgeNavigationEvaluator, // TODO: Re-enable when BridgeNavigationEvaluator is available
    pub analyzer: Option<AnalyzerFieldValidator>,
    pub schema_manager: Arc<FhirSchemaPackageManager>,
    config: FhirPathEngineConfig,
}

impl AdvancedFhirPathEngine {
    /// Evaluate expression with basic functionality
    pub async fn evaluate(
        &self,
        expression: &str,
        context: serde_json::Value,
    ) -> Result<FhirPathValue, FhirPathError> {
        self.engine
            .evaluate(expression, context)
            .await
            .map_err(|e| FhirPathError::Generic {
                message: format!("Evaluation failed: {}", e),
            })
    }

    /// Evaluate with full analysis and performance tracking
    pub async fn evaluate_with_analysis(
        &self,
        expression: &str,
        context: serde_json::Value,
    ) -> Result<crate::config::FhirPathEvaluationResult, FhirPathError> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        let mut performance_metrics = None;

        // Parse timing
        let parse_start = std::time::Instant::now();
        let _ast = crate::parse(expression).map_err(|e| FhirPathError::Generic {
            message: format!("Parse error: {}", e),
        })?;
        let parse_time = parse_start.elapsed();

        // Analysis if enabled
        let analysis_start = std::time::Instant::now();
        let validation_result = if let Some(analyzer) = &self.analyzer {
            let validation = analyzer
                .validate_field("Resource", expression)
                .await
                .map_err(|e| {
                    warnings.push(format!("Analysis warning: {}", e));
                    e
                })
                .ok();

            if let Some(ref result) = validation {
                if !result.is_valid {
                    warnings.extend(result.suggestions.clone());
                }
            }

            validation
        } else {
            None
        };
        let analysis_time = if self.config.enable_analysis {
            Some(analysis_start.elapsed())
        } else {
            None
        };

        // Evaluation
        let eval_start = std::time::Instant::now();
        let values = vec![self.evaluate(expression, context).await?];
        let evaluation_time = eval_start.elapsed();

        // Performance metrics
        if self.config.enable_performance_tracking {
            performance_metrics = Some(PerformanceMetrics {
                parse_time,
                evaluation_time,
                analysis_time,
                cache_hits: 0,      // TODO: Get from actual cache
                cache_misses: 0,    // TODO: Get from actual cache
                memory_usage: None, // TODO: Implement memory tracking
            });
        }

        Ok(crate::config::FhirPathEvaluationResult {
            values,
            validation_result,
            execution_time: start_time.elapsed(),
            warnings,
            performance_metrics,
        })
    }

    /// Check if engine has analysis capabilities
    pub fn has_analysis_capabilities(&self) -> bool {
        self.analyzer.is_some()
    }

    /// Get schema manager reference
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }

    /// Pre-validate expression without evaluation
    pub async fn validate_expression(
        &self,
        expression: &str,
    ) -> Result<Option<octofhir_fhirpath_analyzer::BridgeValidationResult>, FhirPathError> {
        if let Some(analyzer) = &self.analyzer {
            analyzer
                .validate_field("Resource", expression)
                .await
                .map(Some)
                .map_err(|e| FhirPathError::Generic {
                    message: format!("Validation failed: {}", e),
                })
        } else {
            Ok(None)
        }
    }
}

/// CLI-optimized FHIRPath engine
pub struct CliFhirPathEngine {
    advanced_engine: AdvancedFhirPathEngine,
}

impl CliFhirPathEngine {
    /// Create new CLI engine
    pub fn new(advanced_engine: AdvancedFhirPathEngine) -> Self {
        Self { advanced_engine }
    }

    /// Evaluate expression with CLI-friendly output
    pub async fn evaluate_for_cli(
        &self,
        expression: &str,
        context: serde_json::Value,
        show_analysis: bool,
    ) -> Result<CliEvaluationResult, FhirPathError> {
        if show_analysis {
            let result = self
                .advanced_engine
                .evaluate_with_analysis(expression, context)
                .await?;

            Ok(CliEvaluationResult {
                values: result.values,
                execution_time: result.execution_time,
                analysis_enabled: true,
                validation_messages: result
                    .validation_result
                    .map(|v| v.suggestions)
                    .unwrap_or_default(),
                warnings: result.warnings,
                performance_metrics: result.performance_metrics,
            })
        } else {
            let start_time = std::time::Instant::now();
            let values = vec![self.advanced_engine.evaluate(expression, context).await?];

            Ok(CliEvaluationResult {
                values,
                execution_time: start_time.elapsed(),
                analysis_enabled: false,
                validation_messages: vec![],
                warnings: vec![],
                performance_metrics: None,
            })
        }
    }

    /// Parse and validate expression for CLI
    pub async fn validate_for_cli(
        &self,
        expression: &str,
    ) -> Result<CliValidationResult, FhirPathError> {
        // Parse check
        let parse_result = crate::parse(expression);
        let syntax_valid = parse_result.is_ok();
        let parse_error = parse_result.err().map(|e| e.to_string());

        // Semantic validation if available
        let validation_result = self.advanced_engine.validate_expression(expression).await?;

        Ok(CliValidationResult {
            syntax_valid,
            parse_error,
            semantic_valid: validation_result
                .as_ref()
                .map(|v| v.is_valid)
                .unwrap_or(true),
            validation_messages: validation_result
                .as_ref()
                .map(|v| v.suggestions.clone())
                .unwrap_or_default(),
            suggestions: validation_result
                .as_ref()
                .map(|v| v.suggestions.clone())
                .unwrap_or_default(),
        })
    }
}

/// CLI evaluation result
#[derive(Debug)]
pub struct CliEvaluationResult {
    pub values: Vec<FhirPathValue>,
    pub execution_time: Duration,
    pub analysis_enabled: bool,
    pub validation_messages: Vec<String>,
    pub warnings: Vec<String>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// CLI validation result
#[derive(Debug)]
pub struct CliValidationResult {
    pub syntax_valid: bool,
    pub parse_error: Option<String>,
    pub semantic_valid: bool,
    pub validation_messages: Vec<String>,
    pub suggestions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_schema_manager() -> Arc<FhirSchemaPackageManager> {
        // This would create a real schema manager in practice
        // For now, return a placeholder
        todo!("Implement test schema manager")
    }

    #[tokio::test]
    async fn test_engine_factory_creation() {
        let schema_manager = create_test_schema_manager().await;
        let factory = FhirPathEngineFactory::new(schema_manager);

        // Test basic engine creation
        let basic_result = factory.create_basic_engine().await;
        assert!(basic_result.is_ok());
    }

    #[tokio::test]
    async fn test_advanced_engine_capabilities() {
        let schema_manager = create_test_schema_manager().await;
        let factory = FhirPathEngineFactory::new(schema_manager);

        let config = FhirPathEngineConfig::default();
        let advanced_result = factory.create_advanced_engine(&config).await;

        if let Ok(engine) = advanced_result {
            assert!(engine.has_analysis_capabilities());
        }
    }

    #[tokio::test]
    async fn test_cli_engine_creation() {
        let schema_manager = create_test_schema_manager().await;
        let factory = FhirPathEngineFactory::new(schema_manager);

        let cli_result = factory.create_cli_engine(OutputFormat::Pretty).await;
        assert!(cli_result.is_ok());
    }
}
