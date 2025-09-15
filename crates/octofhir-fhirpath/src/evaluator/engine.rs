//! FHIRPath evaluation engine with clean architecture

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{
    ast::ExpressionNode,
    core::{Collection, FhirPathError, FhirPathValue, ModelProvider, Result},
    parser::parse_ast,
    registry::{FunctionRegistry, create_standard_registry},
    evaluator::{EvaluationContext, evaluator::{FhirPathEvaluator, Evaluator}},
};

use octofhir_fhir_model::TerminologyProvider;

/// Evaluation result with metrics and warnings
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Result collection (always a Collection per FHIRPath spec)
    pub value: Collection,
    /// Performance metrics
    pub metrics: EvaluationMetrics,
    /// Warnings generated during evaluation
    pub warnings: Vec<EvaluationWarning>,
}

/// Performance metrics for evaluation
#[derive(Debug, Clone, Default)]
pub struct EvaluationMetrics {
    /// Total evaluation time in microseconds
    pub total_time_us: u64,
    /// Parse time in microseconds
    pub parse_time_us: u64,
    /// Evaluation time in microseconds
    pub eval_time_us: u64,
    /// Number of function calls
    pub function_calls: usize,
    /// Number of model provider calls
    pub model_provider_calls: usize,
}

/// Warning generated during evaluation
#[derive(Debug, Clone)]
pub struct EvaluationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Source location if available
    pub location: Option<std::ops::Range<usize>>,
}

/// FHIRPath evaluation engine
pub struct FhirPathEngine {
    /// Core evaluator
    evaluator: FhirPathEvaluator,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Optional terminology provider
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    /// AST cache for performance
    ast_cache: RwLock<HashMap<String, Arc<ExpressionNode>>>,
}

impl FhirPathEngine {
    /// Create new engine with function registry and model provider
    pub async fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Result<Self> {
        Ok(Self {
            evaluator: FhirPathEvaluator::new(function_registry),
            model_provider,
            terminology_provider: None,
            ast_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Add terminology provider to engine
    pub fn with_terminology_provider(mut self, provider: Arc<dyn TerminologyProvider>) -> Self {
        self.terminology_provider = Some(provider);
        self
    }

    /// Get the function registry for introspection
    pub fn get_function_registry(&self) -> &Arc<FunctionRegistry> {
        self.evaluator.get_function_registry()
    }

    /// Evaluate AST directly (legacy compatibility method)
    pub async fn evaluate_ast(
        &mut self,
        ast: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();

        // Evaluate with providers
        let result = self.evaluator.evaluate(
            ast,
            context,
            self.model_provider.as_ref(),
            self.terminology_provider.as_ref().map(|t| t.as_ref()),
        ).await?;

        let elapsed = start_time.elapsed();
        let metrics = EvaluationMetrics {
            total_time_us: elapsed.as_micros() as u64,
            parse_time_us: 0,
            eval_time_us: elapsed.as_micros() as u64,
            function_calls: 0,
            model_provider_calls: 0,
        };

        Ok(EvaluationResult {
            value: result,
            metrics,
            warnings: Vec::new(),
        })
    }

    /// Plain fast evaluate - returns just the Collection result
    pub async fn evaluate_fast(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<Collection> {
        // Parse expression (with caching)
        let ast = self.parse_or_cached(expression)?;

        // Evaluate with providers - no timing overhead
        self.evaluator.evaluate(
            &ast,
            context,
            self.model_provider.as_ref(),
            self.terminology_provider.as_ref().map(|t| t.as_ref()),
        ).await
    }

    /// Evaluate expression with full metadata and metrics
    pub async fn evaluate_with_metadata(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();

        // Parse expression (with caching)
        let ast = self.parse_or_cached(expression)?;

        // Evaluate with providers
        let result = self.evaluator.evaluate(
            &ast,
            context,
            self.model_provider.as_ref(),
            self.terminology_provider.as_ref().map(|t| t.as_ref()),
        ).await?;

        let elapsed = start_time.elapsed();
        let metrics = EvaluationMetrics {
            total_time_us: elapsed.as_micros() as u64,
            parse_time_us: 0,
            eval_time_us: elapsed.as_micros() as u64,
            function_calls: 0,
            model_provider_calls: 0,
        };

        Ok(EvaluationResult {
            value: result,
            metrics,
            warnings: vec![],
        })
    }

    /// Legacy evaluate method (delegates to evaluate_with_metadata)
    pub async fn evaluate(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        self.evaluate_with_metadata(expression, context).await
    }

    /// Evaluate with variables
    pub async fn evaluate_with_variables(
        &mut self,
        expression: &str,
        collection: &Collection,
        variables: HashMap<String, FhirPathValue>,
    ) -> Result<Collection> {
        let mut context = EvaluationContext::new(
            collection.clone(),
            self.model_provider.clone(),
            self.terminology_provider.clone().map(|t| t as Arc<dyn TerminologyProvider>),
        ).await;

        for (name, value) in variables {
            let _ = context.set_user_variable(name, value);
        }

        let result = self.evaluate(expression, &context).await?;
        Ok(result.value)
    }

    /// Get cached AST or parse and cache expression
    fn parse_or_cached(&self, expression: &str) -> Result<Arc<ExpressionNode>> {
        // Check cache first
        if let Ok(cache) = self.ast_cache.read() {
            if let Some(ast) = cache.get(expression) {
                return Ok(ast.clone());
            }
        }

        // Parse expression
        let ast = parse_ast(expression)?;
        let ast_arc = Arc::new(ast);

        // Cache the result
        if let Ok(mut cache) = self.ast_cache.write() {
            cache.insert(expression.to_string(), ast_arc.clone());
        }

        Ok(ast_arc)
    }

    /// Get model provider
    pub fn get_model_provider(&self) -> Arc<dyn ModelProvider> {
        self.model_provider.clone()
    }

    /// Get terminology provider
    pub fn get_terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    /// Clear AST cache
    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self.ast_cache.write().map_err(|_| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0001,
                "Failed to write AST cache",
            )
        })?;
        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<HashMap<String, usize>> {
        let cache = self.ast_cache.read().map_err(|_| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0001,
                "Failed to read AST cache",
            )
        })?;

        let mut stats = HashMap::new();
        stats.insert("entries".to_string(), cache.len());
        Ok(stats)
    }
}

/// Create engine with mock provider for testing
pub async fn create_engine_with_mock_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;
    let registry = Arc::new(create_standard_registry().await);
    let provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, provider).await
}