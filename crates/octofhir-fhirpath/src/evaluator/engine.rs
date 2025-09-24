//! Real FHIRPath evaluation engine implementation
//!
//! This module provides the actual FhirPathEngine implementation that replaces
//! the stub, using the new registry-based evaluator architecture.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::trace::SharedTraceProvider;
use crate::core::{FhirPathValue, ModelProvider, Result};
use crate::parser;

use async_trait::async_trait;
use octofhir_fhir_model::{
    CompiledExpression, ErrorSeverity, EvaluationResult as ModelEvaluationResult,
    FhirPathConstraint, FhirPathEvaluator, TerminologyProvider, ValidationError,
    ValidationProvider, ValidationResult, Variables,
};
use serde_json::Value as JsonValue;

use super::context::EvaluationContext;
use super::evaluator::Evaluator;
use super::function_registry::{FunctionRegistry, create_function_registry};
use super::operator_registry::{OperatorRegistry, create_standard_operator_registry};
use super::result::{EvaluationResult, EvaluationResultWithMetadata};

/// FHIRPath evaluation engine with registry-based architecture
pub struct FhirPathEngine {
    /// The core evaluator
    evaluator: Evaluator,
    /// Function registry for introspection
    function_registry: Arc<FunctionRegistry>,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Optional terminology provider
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    /// Optional trace provider
    trace_provider: Option<SharedTraceProvider>,
    /// Optional validation provider for profile validation
    validation_provider: Option<Arc<dyn ValidationProvider>>,
}

impl FhirPathEngine {
    /// Create new engine with function registry and model provider
    pub async fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Result<Self> {
        // Create standard operator registry
        let operator_registry = Arc::new(create_standard_operator_registry());

        // Create the evaluator
        let evaluator = Evaluator::new(
            operator_registry,
            function_registry.clone(),
            model_provider.clone(),
            None,
        );

        Ok(Self {
            evaluator,
            function_registry,
            model_provider,
            terminology_provider: None,
            trace_provider: None,
            validation_provider: None,
        })
    }

    /// Create new engine with custom registries
    pub async fn new_with_registries(
        operator_registry: Arc<OperatorRegistry>,
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Result<Self> {
        let evaluator = Evaluator::new(
            operator_registry,
            function_registry.clone(),
            model_provider.clone(),
            None,
        );

        Ok(Self {
            evaluator,
            function_registry,
            model_provider,
            terminology_provider: None,
            trace_provider: None,
            validation_provider: None,
        })
    }

    /// Add terminology provider to engine
    pub fn with_terminology_provider(mut self, provider: Arc<dyn TerminologyProvider>) -> Self {
        self.terminology_provider = Some(provider.clone());
        self.evaluator = self.evaluator.with_terminology_provider(provider);
        self
    }

    /// Add trace provider to engine
    pub fn with_trace_provider(mut self, provider: SharedTraceProvider) -> Self {
        self.trace_provider = Some(provider.clone());
        self.evaluator = self.evaluator.with_trace_provider(provider);
        self
    }

    /// Add validation provider to engine
    pub fn with_validation_provider(mut self, provider: Arc<dyn ValidationProvider>) -> Self {
        self.validation_provider = Some(provider);
        self
    }

    /// Get the function registry for introspection
    pub fn get_function_registry(&self) -> &Arc<FunctionRegistry> {
        &self.function_registry
    }

    /// Get model provider
    pub fn get_model_provider(&self) -> Arc<dyn ModelProvider> {
        self.model_provider.clone()
    }

    /// Get terminology provider
    pub fn get_terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    /// Get trace provider
    pub fn get_trace_provider(&self) -> Option<SharedTraceProvider> {
        self.trace_provider.clone()
    }

    /// Get validation provider
    pub fn get_validation_provider(&self) -> Option<Arc<dyn ValidationProvider>> {
        self.validation_provider.clone()
    }

    /// Auto-prepend resource type if expression doesn't start with capital letter
    #[allow(dead_code)]
    async fn maybe_prepend_resource_type(
        &self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<String> {
        // Check if expression already starts with capital letter (explicit resource type)
        if expression
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
            return Ok(expression.to_string());
        }

        // Try to auto-extract resource type from input
        if let Some(resource_type) = self.extract_resource_type_from_context(context).await? {
            // Prepend the resource type
            Ok(format!("{resource_type}.{expression}"))
        } else {
            // No resource type found, use expression as-is
            Ok(expression.to_string())
        }
    }

    /// Extract resource type from evaluation context
    #[allow(dead_code)]
    async fn extract_resource_type_from_context(
        &self,
        context: &EvaluationContext,
    ) -> Result<Option<String>> {
        for item in context.input_collection().iter() {
            if let FhirPathValue::Resource(json, _, _) = item {
                if let Some(resource_type) = json.get("resourceType").and_then(|rt| rt.as_str()) {
                    // Validate that this is a known FHIR resource type
                    if self.model_provider.get_type(resource_type).await.is_ok() {
                        return Ok(Some(resource_type.to_string()));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Evaluate expression
    pub async fn evaluate(
        &self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Start evaluation

        // Parse the expression as-is without modification
        let ast = parser::parse_ast(expression)?;

        // Evaluate using the real evaluator
        let result = self.evaluate_ast(&ast, context).await;

        // Server logs evaluation outcome; engine remains silent

        result
    }

    /// Evaluate AST directly
    pub async fn evaluate_ast(
        &self,
        ast: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        self.evaluator.evaluate_node(ast, context).await
    }

    /// Evaluate expression with metadata
    pub async fn evaluate_with_metadata(
        &self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResultWithMetadata> {
        // Start evaluation

        // Parse the expression
        let ast = parser::parse_ast(expression)?;

        // Evaluate with metadata using the real evaluator
        let result = self.evaluate_ast_with_metadata(&ast, context).await;

        // Server logs evaluation outcome; engine remains silent

        result
    }

    /// Evaluate AST with metadata
    pub async fn evaluate_ast_with_metadata(
        &self,
        ast: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResultWithMetadata> {
        self.evaluator
            .evaluate_node_with_metadata(ast, context)
            .await
    }
}

/// Create engine with mock provider for testing (transitional compatibility)
pub async fn create_engine_with_mock_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;

    let registry = Arc::new(create_function_registry());
    let provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, provider).await
}

// === FhirPathEvaluator trait implementation ===

#[async_trait]
impl FhirPathEvaluator for FhirPathEngine {
    /// Core evaluation method
    async fn evaluate(
        &self,
        expression: &str,
        context: &JsonValue,
    ) -> octofhir_fhir_model::Result<ModelEvaluationResult> {
        // Convert JsonValue to our Collection format
        let collection = crate::core::Collection::from_json_resource(
            context.clone(),
            Some(self.model_provider.clone()),
        )
        .await
        .map_err(|e| octofhir_fhir_model::ModelError::evaluation_error(e.to_string()))?;

        // Create evaluation context
        let eval_context = EvaluationContext::new(
            collection,
            self.model_provider.clone(),
            self.terminology_provider.clone(),
            self.validation_provider.clone(),
            self.trace_provider.clone(),
        )
        .await;

        // Evaluate using our internal engine
        let result = self
            .evaluate(expression, &eval_context)
            .await
            .map_err(|e| octofhir_fhir_model::ModelError::evaluation_error(e.to_string()))?;

        // Convert to ModelEvaluationResult
        Ok(result.to_evaluation_result())
    }

    /// Evaluate with variables
    async fn evaluate_with_variables(
        &self,
        expression: &str,
        context: &JsonValue,
        variables: &Variables,
    ) -> octofhir_fhir_model::Result<ModelEvaluationResult> {
        // Convert JsonValue to our Collection format
        let collection = crate::core::Collection::from_json_resource(
            context.clone(),
            Some(self.model_provider.clone()),
        )
        .await
        .map_err(|e| octofhir_fhir_model::ModelError::evaluation_error(e.to_string()))?;

        // Create evaluation context with variables
        let eval_context = EvaluationContext::new(
            collection,
            self.model_provider.clone(),
            self.terminology_provider.clone(),
            self.validation_provider.clone(),
            self.trace_provider.clone(),
        )
        .await;

        // Add variables to context
        // TODO: Implement proper conversion from ModelEvaluationResult to FhirPathValue
        for name in variables.keys() {
            // For now, skip variable conversion - this needs proper implementation
            // let fhir_value = FhirPathValue::from_evaluation_result(value);
            // eval_context.add_variable(name.clone(), fhir_value);
            eprintln!("Warning: Variable {name} not added - conversion not implemented");
        }

        // Evaluate using our internal engine
        let result = self
            .evaluate(expression, &eval_context)
            .await
            .map_err(|e| octofhir_fhir_model::ModelError::evaluation_error(e.to_string()))?;

        // Convert to ModelEvaluationResult
        Ok(result.to_evaluation_result())
    }

    /// Compile an expression for reuse
    async fn compile(&self, expression: &str) -> octofhir_fhir_model::Result<CompiledExpression> {
        // Parse the expression to validate it
        match crate::parser::parse_ast(expression) {
            Ok(_ast) => {
                // For now, we'll store the original expression as compiled form
                // In a real implementation, this could be serialized AST or optimized form
                Ok(CompiledExpression::new(
                    expression.to_string(),
                    expression.to_string(), // TODO: Store actual compiled form
                    true,
                ))
            }
            Err(e) => Ok(CompiledExpression::invalid(
                expression.to_string(),
                e.to_string(),
            )),
        }
    }

    /// Validate expression syntax
    async fn validate_expression(
        &self,
        expression: &str,
    ) -> octofhir_fhir_model::Result<ValidationResult> {
        match crate::parser::parse_ast(expression) {
            Ok(_ast) => Ok(ValidationResult::success()),
            Err(e) => {
                let error = ValidationError::new(format!("Syntax error: {e}"))
                    .with_code("SYNTAX_ERROR".to_string());
                Ok(ValidationResult::with_errors(vec![error]))
            }
        }
    }

    /// Get the ModelProvider for this evaluator
    fn model_provider(&self) -> &dyn octofhir_fhir_model::ModelProvider {
        self.model_provider.as_ref()
    }

    /// Get the ValidationProvider for this evaluator (if available)
    fn validation_provider(&self) -> Option<&dyn ValidationProvider> {
        self.validation_provider.as_ref().map(|p| p.as_ref())
    }

    /// Validate FHIR constraints
    async fn validate_constraints(
        &self,
        resource: &JsonValue,
        constraints: &[FhirPathConstraint],
    ) -> octofhir_fhir_model::Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for constraint in constraints {
            // Convert resource to collection for evaluation
            let collection = crate::core::Collection::from_json_resource(
                resource.clone(),
                Some(self.model_provider.clone()),
            )
            .await
            .map_err(|e| octofhir_fhir_model::ModelError::evaluation_error(e.to_string()))?;

            let eval_context = EvaluationContext::new(
                collection,
                self.model_provider.clone(),
                self.terminology_provider.clone(),
                self.validation_provider.clone(),
                self.trace_provider.clone(),
            )
            .await;

            // Evaluate the constraint expression using internal engine method
            match FhirPathEngine::evaluate(self, &constraint.expression, &eval_context).await {
                Ok(result) => {
                    // Check if the result is truthy
                    if !result.to_boolean() {
                        let error = ValidationError::new(constraint.description.clone())
                            .with_code(constraint.key.clone())
                            .with_location(constraint.expression.clone());

                        match constraint.severity {
                            ErrorSeverity::Error | ErrorSeverity::Fatal => {
                                errors.push(error);
                            }
                            ErrorSeverity::Warning => {
                                let warning = octofhir_fhir_model::ValidationWarning::new(
                                    constraint.description.clone(),
                                )
                                .with_code(constraint.key.clone())
                                .with_location(constraint.expression.clone());
                                warnings.push(warning);
                            }
                            ErrorSeverity::Information => {
                                // Info level - add as warning but don't fail validation
                                let warning = octofhir_fhir_model::ValidationWarning::new(
                                    constraint.description.clone(),
                                )
                                .with_code(constraint.key.clone())
                                .with_location(constraint.expression.clone());
                                warnings.push(warning);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Evaluation error - treat as constraint failure
                    let error = ValidationError::new(format!("Constraint evaluation failed: {e}"))
                        .with_code(constraint.key.clone())
                        .with_location(constraint.expression.clone());
                    errors.push(error);
                }
            }
        }

        let mut result = if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::with_errors(errors)
        };

        for warning in warnings {
            result = result.with_warning(warning);
        }

        Ok(result)
    }

    /// Check if the evaluator supports a specific feature
    fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "compilation" => true,
            "variables" => true,
            "constraints" => true,
            "terminology" => self.terminology_provider.is_some(),
            "tracing" => self.trace_provider.is_some(),
            _ => false,
        }
    }
}
