//! Engine Factory for FHIRPath - Simplified Version

use crate::{
    EvaluationConfig, FhirPathEngine, FhirPathError, FhirPathValue, MockModelProvider,
    ModelProvider,
    config::{FhirPathConfig, FhirPathEngineConfig, OutputFormat, PerformanceMetrics},
};
use std::sync::Arc;
use std::time::Duration;

/// Factory for creating FHIRPath engines
///
/// This is a simplified factory that creates engines using MockModelProvider only.
/// For full FHIR schema support, use the CLI crate instead.
pub struct FhirPathEngineFactory;

impl FhirPathEngineFactory {
    /// Create a basic FHIRPath engine for simple evaluation
    pub async fn create_basic_engine(&self) -> Result<Box<dyn ModelProvider>, FhirPathError> {
        let model_provider = MockModelProvider::default();
        Ok(Box::new(model_provider))
    }

    /// Create an engine with custom configuration
    pub async fn create_configured_engine(
        &self,
        _config: &FhirPathEngineConfig,
    ) -> Result<FhirPathEngine, FhirPathError> {
        // Create model provider
        let model_provider = Arc::new(MockModelProvider::default());

        // Create registry
        let registry = crate::create_standard_registry().await;

        // Create engine
        let engine = FhirPathEngine::new(Arc::new(registry), model_provider);

        Ok(engine)
    }
}

/// Advanced FHIRPath engine with additional capabilities
pub struct AdvancedFhirPathEngine {
    /// Core engine
    pub engine: FhirPathEngine,
    /// Configuration
    pub config: FhirPathEngineConfig,
}

impl AdvancedFhirPathEngine {
    /// Create a new advanced engine
    pub async fn new(config: FhirPathEngineConfig) -> Result<Self, FhirPathError> {
        let factory = FhirPathEngineFactory;
        let engine = factory.create_configured_engine(&config).await?;

        Ok(Self { engine, config })
    }

    /// Evaluate with detailed results
    pub async fn evaluate_detailed(
        &self,
        expression: &str,
        context: serde_json::Value,
    ) -> Result<CliEvaluationResult, FhirPathError> {
        let start_time = std::time::Instant::now();

        let result = self
            .engine
            .evaluate(expression, context)
            .await
            .map_err(crate::evaluation_error_to_fhirpath_error)?;
        let execution_time = start_time.elapsed();

        Ok(CliEvaluationResult {
            values: vec![result],
            execution_time,
            warnings: vec![],
            performance_metrics: None,
        })
    }
}

/// CLI evaluation result with detailed information

pub struct CliEvaluationResult {
    /// The evaluated values
    pub values: Vec<FhirPathValue>,
    /// Execution time
    pub execution_time: Duration,
    /// Warnings generated during evaluation
    pub warnings: Vec<String>,
    /// Performance metrics (if enabled)
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// CLI FHIRPath engine for command-line operations
pub struct CliFhirPathEngine {
    /// Core engine
    pub engine: FhirPathEngine,
    /// Configuration
    pub config: FhirPathConfig,
}

impl CliFhirPathEngine {
    /// Create a new CLI engine
    pub async fn new(config: FhirPathConfig) -> Result<Self, FhirPathError> {
        let model_provider = Arc::new(MockModelProvider::default());
        let registry = crate::create_standard_registry().await;
        let engine = FhirPathEngine::new(Arc::new(registry), model_provider);

        Ok(Self { engine, config })
    }

    /// Evaluate expression with CLI-specific formatting
    pub async fn evaluate_for_cli(
        &self,
        expression: &str,
        context: serde_json::Value,
        output_format: OutputFormat,
    ) -> Result<String, FhirPathError> {
        let result = self
            .engine
            .evaluate(expression, context)
            .await
            .map_err(crate::evaluation_error_to_fhirpath_error)?;

        match output_format {
            OutputFormat::Json => {
                serde_json::to_string_pretty(&result).map_err(crate::json_error_to_fhirpath_error)
            }
            OutputFormat::Raw => Ok(format!("{:?}", result)),
            OutputFormat::Pretty => Ok(self.format_pretty(&result)),
            OutputFormat::Table => Ok(self.format_table(&result)),
        }
    }

    fn format_pretty(&self, value: &FhirPathValue) -> String {
        format!("🎯 Result: {:?}", value)
    }

    fn format_table(&self, value: &FhirPathValue) -> String {
        format!("| Result |\n|--------|\n| {:?} |", value)
    }
}

/// CLI validation result
pub struct CliValidationResult {
    /// Whether the expression is valid
    pub is_valid: bool,
    /// Validation messages
    pub messages: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

impl CliValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            messages: vec![],
            suggestions: vec![],
        }
    }

    /// Create a failed validation result
    pub fn failed(message: String) -> Self {
        Self {
            is_valid: false,
            messages: vec![message],
            suggestions: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_engine_factory() {
        let factory = FhirPathEngineFactory;
        let provider = factory.create_basic_engine().await.unwrap();

        // Basic test - just ensure we can create a provider
        assert!(provider.is_resource_type("Patient").await.unwrap());
    }

    #[tokio::test]
    async fn test_advanced_engine() {
        let config = FhirPathEngineConfig::default();
        let engine = AdvancedFhirPathEngine::new(config).await.unwrap();

        let context = serde_json::json!({
            "resourceType": "Patient",
            "id": "test"
        });

        let result = engine
            .evaluate_detailed("Patient.id", context)
            .await
            .unwrap();
        assert!(!result.values.is_empty());
    }

    #[tokio::test]
    async fn test_cli_engine() {
        let config = FhirPathConfig::default();
        let cli_engine = CliFhirPathEngine::new(config).await.unwrap();

        let context = serde_json::json!({
            "resourceType": "Patient",
            "id": "test"
        });

        let result = cli_engine
            .evaluate_for_cli("Patient.id", context, OutputFormat::Raw)
            .await
            .unwrap();

        assert!(!result.is_empty());
    }
}
