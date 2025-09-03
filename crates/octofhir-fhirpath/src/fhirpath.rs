//! Main FHIRPath API - Simplified Version

use crate::{
    FhirPathError, FhirPathValue, MockModelProvider,
    config::{FhirPathConfig, FhirPathConfigBuilder, FhirPathEvaluationResult},
    engine_factory::{AdvancedFhirPathEngine, FhirPathEngineFactory},
};
use std::sync::Arc;

/// Main FHIRPath entry point
///
/// This provides a high-level API for FHIRPath operations using MockModelProvider.
/// For full FHIR schema support, use the CLI crate instead.
pub struct FhirPath {
    engine: AdvancedFhirPathEngine,
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
    ///     // Use fhirpath for evaluation...
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self, FhirPathError> {
        let config = FhirPathConfig::default();
        Self::with_config(config).await
    }

    /// Create a new FHIRPath instance with custom configuration
    ///
    /// # Arguments
    /// * `config` - The configuration to use
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::{FhirPath, FhirPathConfigBuilder, FhirVersion};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = FhirPathConfigBuilder::new()
    ///         .with_fhir_version(FhirVersion::R5)
    ///         .build();
    ///
    ///     let fhirpath = FhirPath::with_config(config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_config(config: FhirPathConfig) -> Result<Self, FhirPathError> {
        let engine_config = config.to_engine_config();
        let engine = AdvancedFhirPathEngine::new(engine_config).await?;

        Ok(Self { engine, config })
    }

    /// Get the current configuration
    pub fn config(&self) -> &FhirPathConfig {
        &self.config
    }

    /// Evaluate a FHIRPath expression
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
    ///     let result = fhirpath.evaluate("Patient.name.given", patient).await?;
    ///     println!("Values: {:?}", result.values);
    ///     Ok(())
    /// }
    /// ```
    pub async fn evaluate(
        &self,
        expression: &str,
        context: serde_json::Value,
    ) -> Result<FhirPathEvaluationResult, FhirPathError> {
        let start_time = std::time::Instant::now();

        // Parse expression first for validation
        let _ast = crate::parse(expression)?;

        // Evaluate
        let result = self
            .engine
            .engine
            .evaluate(expression, context)
            .await
            .map_err(crate::evaluation_error_to_fhirpath_error)?;
        let execution_time = start_time.elapsed();

        Ok(FhirPathEvaluationResult {
            values: vec![result],
            execution_time,
            warnings: vec![],
            performance_metrics: None,
        })
    }

    /// Parse a FHIRPath expression
    ///
    /// # Arguments
    /// * `expression` - The FHIRPath expression to parse
    ///
    /// # Example
    /// ```no_run
    /// use octofhir_fhirpath::FhirPath;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let fhirpath = FhirPath::new().await?;
    ///     let ast = fhirpath.parse_expression("Patient.name.given")?;
    ///     println!("AST: {:?}", ast);
    ///     Ok(())
    /// }
    /// ```
    pub fn parse_expression(
        &self,
        expression: &str,
    ) -> Result<crate::ast::ExpressionNode, FhirPathError> {
        crate::parse(expression)
    }

    /// Check if an expression has valid syntax
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
    ///     if fhirpath.is_valid_syntax("Patient.name.given")? {
    ///         println!("Valid syntax");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn is_valid_syntax(&self, expression: &str) -> Result<bool, FhirPathError> {
        match self.parse_expression(expression) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Evaluate multiple expressions against the same context
    ///
    /// This is more efficient than calling evaluate multiple times
    /// when you have the same context.
    ///
    /// # Arguments
    /// * `expressions` - The FHIRPath expressions to evaluate
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
    ///         "name": [{"given": ["John"], "family": "Doe"}],
    ///         "active": true
    ///     });
    ///
    ///     let expressions = vec!["Patient.name.given", "Patient.name.family", "Patient.active"];
    ///     let results = fhirpath.evaluate_batch(expressions, patient).await?;
    ///
    ///     for (expr, result) in results {
    ///         println!("{}: {:?}", expr, result.values);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn evaluate_batch(
        &self,
        expressions: Vec<&str>,
        context: serde_json::Value,
    ) -> Result<Vec<(String, FhirPathEvaluationResult)>, FhirPathError> {
        let mut results = Vec::new();

        for expression in expressions {
            let result = self.evaluate(expression, context.clone()).await?;
            results.push((expression.to_string(), result));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_fhirpath_new() {
        let fhirpath = FhirPath::new().await.unwrap();
        assert!(fhirpath.config().fhir_version == crate::config::FhirVersion::R4);
    }

    #[tokio::test]
    async fn test_fhirpath_with_config() {
        let config = FhirPathConfigBuilder::new()
            .with_fhir_version(crate::config::FhirVersion::R5)
            .build();

        let fhirpath = FhirPath::with_config(config).await.unwrap();
        assert!(fhirpath.config().fhir_version == crate::config::FhirVersion::R5);
    }

    #[tokio::test]
    async fn test_evaluate_simple() {
        let fhirpath = FhirPath::new().await.unwrap();

        let patient = json!({
            "resourceType": "Patient",
            "id": "test-patient"
        });

        let result = fhirpath.evaluate("Patient.id", patient).await.unwrap();
        assert!(!result.values.is_empty());
    }

    #[tokio::test]
    async fn test_parse_expression() {
        let fhirpath = FhirPath::new().await.unwrap();

        // Valid expression
        let ast = fhirpath.parse_expression("Patient.name.given");
        assert!(ast.is_ok());

        // Invalid expression
        let invalid_ast = fhirpath.parse_expression("Patient.name(");
        assert!(invalid_ast.is_err());
    }

    #[tokio::test]
    async fn test_is_valid_syntax() {
        let fhirpath = FhirPath::new().await.unwrap();

        assert!(fhirpath.is_valid_syntax("Patient.name").unwrap());
        assert!(!fhirpath.is_valid_syntax("Patient.name(").unwrap());
    }

    #[tokio::test]
    async fn test_evaluate_batch() {
        let fhirpath = FhirPath::new().await.unwrap();

        let patient = json!({
            "resourceType": "Patient",
            "id": "test-patient",
            "active": true
        });

        let expressions = vec!["Patient.id", "Patient.active", "Patient.resourceType"];
        let results = fhirpath.evaluate_batch(expressions, patient).await.unwrap();

        assert_eq!(results.len(), 3);
        for (_, result) in results {
            assert!(!result.values.is_empty());
        }
    }
}
