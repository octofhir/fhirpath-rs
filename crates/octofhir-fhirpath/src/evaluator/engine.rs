//! Real FHIRPath evaluation engine implementation
//!
//! This module provides the actual FhirPathEngine implementation that replaces
//! the stub, using the new registry-based evaluator architecture.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::trace::SharedTraceProvider;
use crate::core::{FhirPathValue, ModelProvider, Result};
use crate::parser;
use octofhir_fhir_model::TerminologyProvider;

use super::context::EvaluationContext;
use super::evaluator::Evaluator;
use super::function_registry::{FunctionRegistry, create_basic_function_registry};
use super::operator_registry::{OperatorRegistry, create_standard_operator_registry};
use super::stub::{EvaluationResult, EvaluationResultWithMetadata};

/// Real FHIRPath evaluation engine with registry-based architecture
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

    /// Auto-prepend resource type if expression doesn't start with capital letter
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
            Ok(format!("{}.{}", resource_type, expression))
        } else {
            // No resource type found, use expression as-is
            Ok(expression.to_string())
        }
    }

    /// Extract resource type from evaluation context
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
        // Parse the expression as-is without modification
        let ast = parser::parse_ast(expression)?;

        // Evaluate using the real evaluator
        self.evaluate_ast(&ast, context).await
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
        // Parse the expression
        let ast = parser::parse_ast(expression)?;

        // Evaluate with metadata using the real evaluator
        self.evaluate_ast_with_metadata(&ast, context).await
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

    let registry = Arc::new(create_basic_function_registry());
    let provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, provider).await
}
