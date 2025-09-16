//! Stub implementation of FHIRPath evaluator for CLI/dev tools compatibility
//!
//! This stub provides the minimum implementation needed to keep CLI and dev tools
//! working during the evaluator redesign. It returns dummy results but maintains
//! the exact same public API.

use std::sync::Arc;
use std::time::Duration;

use crate::ast::ExpressionNode;
use crate::core::{Collection, ModelProvider, Result};
use crate::FunctionRegistry;
use octofhir_fhir_model::TerminologyProvider;

/// Stub evaluation context for CLI compatibility
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    input: Collection,
    model_provider: Arc<dyn ModelProvider>,
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
}

impl EvaluationContext {
    /// Create new evaluation context
    pub async fn new(
        input: Collection,
        model_provider: Arc<dyn ModelProvider>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    ) -> Self {
        Self {
            input,
            model_provider,
            terminology_provider,
        }
    }

    /// Get input collection
    pub fn input_collection(&self) -> &Collection {
        &self.input
    }

    /// Get model provider
    pub fn model_provider(&self) -> Arc<dyn ModelProvider> {
        self.model_provider.clone()
    }

    /// Get terminology provider
    pub fn terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    /// Check if terminology provider is available
    pub fn has_terminology_provider(&self) -> bool {
        self.terminology_provider.is_some()
    }

    /// Get terminology provider (legacy method name for compatibility)
    pub fn get_terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    /// Get variable value (stub implementation)
    pub fn get_variable(&self, _name: &str) -> Option<&crate::core::FhirPathValue> {
        // STUB: No variables supported yet
        None
    }

    /// Get root context (stub implementation)
    pub fn get_root_context(&self) -> &Collection {
        // STUB: Return input as root context
        &self.input
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Set user variable (stub implementation)
    pub fn set_user_variable(&mut self, _name: String, _value: crate::core::FhirPathValue) -> Result<()> {
        // STUB: No variable setting supported yet
        Ok(())
    }
}

/// Stub evaluation result for CLI compatibility
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Result collection (always a Collection per FHIRPath spec)
    pub value: Collection,
}

/// Stub evaluation result with metadata for CLI debugging
#[derive(Debug, Clone)]
pub struct EvaluationResultWithMetadata {
    /// Result collection
    pub value: Collection,
    /// Evaluation metadata (stub)
    pub metadata: EvaluationMetadata,
}

/// Stub evaluation metadata for CLI debugging
#[derive(Debug, Clone)]
pub struct EvaluationMetadata {
    /// Execution time (always zero for stub)
    pub execution_time: Duration,
    /// Number of operations (always zero for stub)
    pub operation_count: usize,
}

impl Default for EvaluationMetadata {
    fn default() -> Self {
        Self {
            execution_time: Duration::ZERO,
            operation_count: 0,
        }
    }
}

/// Stub FHIRPath evaluation engine
pub struct FhirPathEngine {
    function_registry: Arc<FunctionRegistry>,
    model_provider: Arc<dyn ModelProvider>,
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
}

impl FhirPathEngine {
    /// Create new engine with function registry and model provider
    pub async fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Result<Self> {
        Ok(Self {
            function_registry,
            model_provider,
            terminology_provider: None,
        })
    }

    /// Add terminology provider to engine
    pub fn with_terminology_provider(mut self, provider: Arc<dyn TerminologyProvider>) -> Self {
        self.terminology_provider = Some(provider);
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

    /// Evaluate expression - STUB IMPLEMENTATION
    /// Returns empty collection for now, but maintains API compatibility
    pub async fn evaluate(
        &self,
        expression: &str,
        _context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // STUB: Parse the expression to validate syntax
        let _ast = crate::parser::parse_ast(expression)?;

        // STUB: Return empty result for now
        // TODO: Replace with real evaluation in Phase 1+
        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    }

    /// Evaluate AST directly - STUB IMPLEMENTATION
    pub async fn evaluate_ast(
        &self,
        _ast: &ExpressionNode,
        _context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // STUB: Return empty result for now
        // TODO: Replace with real evaluation in Phase 1+
        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    }

    /// Evaluate expression with metadata - STUB IMPLEMENTATION
    /// CRITICAL: This method is used by CLI and dev tools for debugging
    pub async fn evaluate_with_metadata(
        &self,
        expression: &str,
        _context: &EvaluationContext,
    ) -> Result<EvaluationResultWithMetadata> {
        // STUB: Parse the expression to validate syntax
        let _ast = crate::parser::parse_ast(expression)?;

        // STUB: Return empty result with dummy metadata
        // TODO: Replace with real evaluation and metadata collection in Phase 6
        Ok(EvaluationResultWithMetadata {
            value: Collection::empty(),
            metadata: EvaluationMetadata::default(),
        })
    }

    /// Evaluate AST with metadata - STUB IMPLEMENTATION
    pub async fn evaluate_ast_with_metadata(
        &self,
        _ast: &ExpressionNode,
        _context: &EvaluationContext,
    ) -> Result<EvaluationResultWithMetadata> {
        // STUB: Return empty result with dummy metadata
        // TODO: Replace with real evaluation and metadata collection in Phase 6
        Ok(EvaluationResultWithMetadata {
            value: Collection::empty(),
            metadata: EvaluationMetadata::default(),
        })
    }
}

/// STUB: Create engine with mock provider for testing
pub async fn create_engine_with_mock_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;
    // TODO: Replace with real registry when implemented
    let registry = Arc::new(crate::FunctionRegistry::new());
    let provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, provider).await
}