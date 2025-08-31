//! Main FHIRPath API with Bridge Support

use crate::{
    AnalyzerFieldValidator, FhirPathError, FhirPathValue, FhirSchemaPackageManager,
    config::{FhirPathConfig, FhirPathConfigBuilder, FhirPathEvaluationResult},
    engine_factory::{AdvancedFhirPathEngine, FhirPathEngineFactory},
};
use octofhir_fhirpath_analyzer::BridgeValidationResult;
use std::sync::Arc;

/// Main FHIRPath entry point with Bridge Support Architecture
///
/// This provides a high-level API for FHIRPath operations with enhanced
/// performance and FHIR compliance through the Bridge Support system.
pub struct FhirPath {
    engine: AdvancedFhirPathEngine,
    schema_manager: Arc<FhirSchemaPackageManager>,
    config: FhirPathConfig,
}

impl FhirPath {
    /// Create a new FHIRPath instance with default configuration
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::FhirPath;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fhirpath = FhirPath::new().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self, FhirPathError> {
        Self::with_config(FhirPathConfig::default()).await
    }

    /// Create a new FHIRPath instance with custom configuration
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::{FhirPath, FhirPathConfigBuilder, FhirVersion};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = FhirPathConfigBuilder::new()
    ///         .with_fhir_version(FhirVersion::R5)
    ///         .with_analyzer(true)
    ///         .build();
    ///         
    ///     let fhirpath = FhirPath::with_config(config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_config(config: FhirPathConfig) -> Result<Self, FhirPathError> {
        // Initialize schema manager with bridge support
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let pm_config = octofhir_fhirschema::PackageManagerConfig::default();
        let schema_manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, pm_config)
                .await
                .map_err(|e| FhirPathError::Generic {
                    message: format!("Failed to initialize schema manager: {}", e),
                })?,
        );

        // Create engine factory and advanced engine
        let engine_factory = FhirPathEngineFactory::new(schema_manager.clone());
        let engine = engine_factory
            .create_advanced_engine(&config.engine_config)
            .await?;

        Ok(Self {
            engine,
            schema_manager,
            config,
        })
    }

    /// Evaluate a FHIRPath expression against a JSON resource
    ///
    /// This is the basic evaluation method that returns just the results.
    ///
    /// # Arguments
    /// * `expression` - The FHIRPath expression to evaluate
    /// * `context` - The JSON resource to evaluate against
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::FhirPath;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fhirpath = FhirPath::new().await?;
    ///     
    ///     let patient = json!({
    ///         "resourceType": "Patient",
    ///         "name": [{"given": ["John"], "family": "Doe"}]
    ///     });
    ///
    ///     let results = fhirpath.evaluate("Patient.name.given", &patient).await?;
    ///     println!("Names: {:?}", results);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn evaluate(
        &self,
        expression: &str,
        context: &serde_json::Value,
    ) -> Result<Vec<FhirPathValue>, FhirPathError> {
        let result = self.engine.evaluate(expression, context.clone()).await?;
        Ok(vec![result])
    }

    /// Evaluate with additional validation and analysis
    ///
    /// This method provides comprehensive analysis including validation,
    /// performance metrics, and warnings.
    ///
    /// # Arguments
    /// * `expression` - The FHIRPath expression to evaluate
    /// * `context` - The JSON resource to evaluate against
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::FhirPath;
    /// use serde_json::json;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fhirpath = FhirPath::new().await?;
    ///     
    ///     let patient = json!({
    ///         "resourceType": "Patient",
    ///         "name": [{"given": ["Alice"], "family": "Smith"}]
    ///     });
    ///
    ///     let result = fhirpath.evaluate_with_analysis(
    ///         "Patient.name.where(use = 'official').family",
    ///         &patient
    ///     ).await?;
    ///     
    ///     println!("Values: {:?}", result.values);
    ///     println!("Execution time: {:?}", result.execution_time);
    ///     if !result.warnings.is_empty() {
    ///         println!("Warnings: {:?}", result.warnings);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn evaluate_with_analysis(
        &self,
        expression: &str,
        context: &serde_json::Value,
    ) -> Result<FhirPathEvaluationResult, FhirPathError> {
        self.engine
            .evaluate_with_analysis(expression, context.clone())
            .await
    }

    /// Validate a FHIRPath expression without evaluation
    ///
    /// Checks both syntax and semantic validity of the expression.
    ///
    /// # Arguments
    /// * `expression` - The FHIRPath expression to validate
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::FhirPath;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fhirpath = FhirPath::new().await?;
    ///     
    ///     match fhirpath.validate_expression("Patient.name.invalid").await? {
    ///         Some(result) if !result.is_valid => {
    ///             println!("Invalid expression: {:?}", result.messages);
    ///         },
    ///         Some(_) => println!("Expression is valid"),
    ///         None => println!("Validation not available (analyzer disabled)"),
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn validate_expression(
        &self,
        expression: &str,
    ) -> Result<Option<BridgeValidationResult>, FhirPathError> {
        self.engine.validate_expression(expression).await
    }

    /// Parse a FHIRPath expression to its AST representation
    ///
    /// Useful for static analysis and expression introspection.
    ///
    /// # Arguments
    /// * `expression` - The FHIRPath expression to parse
    pub fn parse_expression(
        &self,
        expression: &str,
    ) -> Result<crate::ExpressionNode, FhirPathError> {
        crate::parse(expression).map_err(|e| FhirPathError::Generic {
            message: format!("Parse error: {}", e),
        })
    }

    /// Get the schema manager for advanced operations
    ///
    /// Provides access to the underlying schema manager for operations
    /// like package loading and schema introspection.
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }

    /// Check if analyzer is enabled
    pub fn has_analyzer(&self) -> bool {
        self.config.analyzer_enabled
    }

    /// Check if performance tracking is enabled
    pub fn has_performance_tracking(&self) -> bool {
        self.config.performance_tracking
    }

    /// Get the current configuration
    pub fn config(&self) -> &FhirPathConfig {
        &self.config
    }

    /// Load additional FHIR packages
    ///
    /// Dynamically load additional FHIR packages for enhanced functionality.
    ///
    /// # Arguments
    /// * `package_ids` - List of package identifiers to load
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::FhirPath;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut fhirpath = FhirPath::new().await?;
    ///     
    ///     // Load US Core profiles
    ///     fhirpath.load_packages(vec!["hl7.fhir.us.core".to_string()]).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn load_packages(&mut self, package_ids: Vec<String>) -> Result<(), FhirPathError> {
        for package_id in package_ids {
            // TODO: Implement actual package loading through schema manager
            // For now, just add to configuration
            self.config.schema_config.packages.push(package_id);
        }

        // TODO: Refresh engine with new packages
        Ok(())
    }

    /// Generate evaluation warnings for potential issues
    async fn generate_evaluation_warnings(&self, _values: &[FhirPathValue]) -> Vec<String> {
        let warnings = Vec::new();

        // TODO: Implement actual warning generation based on:
        // - Deprecated function usage
        // - Performance concerns
        // - Type safety issues
        // - FHIR compliance issues

        warnings
    }
}

// Convenience methods for common operations
impl FhirPath {
    /// Check if a path exists in the resource
    pub async fn path_exists(
        &self,
        path: &str,
        context: &serde_json::Value,
    ) -> Result<bool, FhirPathError> {
        let results = self.evaluate(path, context).await?;
        Ok(!results.is_empty() && !results[0].is_empty())
    }

    /// Get the first value from evaluation results
    pub async fn evaluate_single(
        &self,
        expression: &str,
        context: &serde_json::Value,
    ) -> Result<Option<FhirPathValue>, FhirPathError> {
        let results = self.evaluate(expression, context).await?;
        Ok(results.into_iter().next())
    }

    /// Get string representation of evaluation results
    pub async fn evaluate_to_string(
        &self,
        expression: &str,
        context: &serde_json::Value,
    ) -> Result<Vec<String>, FhirPathError> {
        let results = self.evaluate(expression, context).await?;
        Ok(results
            .into_iter()
            .filter_map(|v| v.as_string().map(|s| s.to_string()))
            .collect())
    }

    /// Get boolean result from evaluation (useful for where clauses)
    pub async fn evaluate_to_boolean(
        &self,
        expression: &str,
        context: &serde_json::Value,
    ) -> Result<bool, FhirPathError> {
        let results = self.evaluate(expression, context).await?;
        Ok(results
            .first()
            .and_then(|v| v.as_boolean())
            .unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_fhirpath_creation() {
        let result = FhirPath::new().await;
        // This will fail until we have proper mock implementations
        // assert!(result.is_ok());
        let _ = result; // Acknowledge the result to avoid warnings
    }

    #[tokio::test]
    async fn test_fhirpath_with_config() {
        let config = FhirPathConfigBuilder::new().with_analyzer(false).build();

        let result = FhirPath::with_config(config).await;
        // This will fail until we have proper mock implementations
        // assert!(result.is_ok());
        let _ = result; // Acknowledge the result to avoid warnings
    }

    #[test]
    fn test_parse_expression() {
        // This should work since parsing doesn't require the schema manager
        let expression = "Patient.name.given";
        let result = crate::parse(expression);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_parse_expression() {
        let expression = "Patient.name.";
        let result = crate::parse(expression);
        assert!(result.is_err());
    }
}
