//! New FHIRPath Evaluation Engine using CompositeEvaluator
//!
//! This module provides a new implementation of FhirPathEngine that uses
//! the modular CompositeEvaluator architecture for better performance and maintainability.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{
    ast::ExpressionNode,
    core::{Collection, FhirPathError, FhirPathValue, ModelProvider, Result},
    parser::parse_ast,
    path::CanonicalPath,
    registry::{FunctionRegistry, create_standard_registry},
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue, collection_utils},
};

use super::{
    CollectionEvaluatorImpl, CoreEvaluator, FunctionEvaluatorImpl, LambdaEvaluatorImpl, Navigator,
    OperatorEvaluatorImpl, composite::CompositeEvaluator, config::EngineConfig,
    context::EvaluationContext, metrics::EvaluationMetrics,
};

/// Result of expression evaluation with metrics and warnings
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Resulting collection (always a Collection per FHIRPath spec)
    pub value: Collection,
    /// Performance metrics
    pub metrics: EvaluationMetrics,
    /// Any warnings generated during evaluation
    pub warnings: Vec<EvaluationWarning>,
    /// Type resolution statistics
    pub type_stats: TypeResolutionStats,
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

// EnhancedEvaluationResult removed - we now always use EvaluationResult with metadata

/// Statistics about type resolution during evaluation
#[derive(Debug, Clone, Default)]
pub struct TypeResolutionStats {
    /// Number of types resolved via ModelProvider
    pub types_resolved: usize,
    /// Number of types that fell back to inference
    pub types_inferred: usize,
    /// Number of paths constructed
    pub paths_constructed: usize,
    /// Number of cache hits for type resolution
    pub cache_hits: usize,
}

/// FHIRPath evaluation engine using CompositeEvaluator architecture
pub struct FhirPathEngine {
    /// Composite evaluator that orchestrates all evaluation concerns
    evaluator: CompositeEvaluator,
    /// Engine configuration
    config: EngineConfig,
    /// AST cache for frequently used expressions
    ast_cache: RwLock<HashMap<String, Arc<ExpressionNode>>>,
    /// Terminology service for %terminologies built-in variable
    terminology_service: Option<Arc<dyn crate::evaluator::context::TerminologyService>>,
}

impl FhirPathEngine {
    /// Create new engine with function registry and model provider
    pub async fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Result<Self> {
        // Use R4 as default FHIR version
        Self::new_with_fhir_version(function_registry, model_provider, "r4").await
    }

    /// Create new engine with specific FHIR version for terminology services
    pub async fn new_with_fhir_version(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        fhir_version: &str,
    ) -> Result<Self> {
        // Create specialized evaluators
        let core_evaluator = Box::new(CoreEvaluator::new());
        let navigator = Box::new(Navigator::new());
        let function_evaluator = Box::new(FunctionEvaluatorImpl::new(
            function_registry.clone(),
            model_provider.clone(),
        ));
        let operator_evaluator = Box::new(OperatorEvaluatorImpl::new());
        let collection_evaluator = Box::new(CollectionEvaluatorImpl::new());
        let lambda_evaluator = Box::new(LambdaEvaluatorImpl::new());

        let evaluator = CompositeEvaluator::new(
            core_evaluator,
            navigator,
            function_evaluator,
            operator_evaluator,
            collection_evaluator,
            lambda_evaluator,
            model_provider,
            function_registry,
            EngineConfig::default(),
        )
        .await;

        // Create terminology service with the specified FHIR version
        let terminology_service = Some(Arc::new(
            crate::registry::ConcreteTerminologyService::with_fhir_version(fhir_version),
        )
            as Arc<dyn crate::evaluator::context::TerminologyService>);

        Ok(Self {
            evaluator,
            config: EngineConfig::default(),
            ast_cache: RwLock::new(HashMap::new()),
            terminology_service,
        })
    }

    /// Create engine with custom configuration
    pub async fn with_config(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        config: EngineConfig,
    ) -> Result<Self> {
        // Use R4 as default FHIR version
        Self::with_config_and_fhir_version(function_registry, model_provider, config, "r4").await
    }

    /// Create engine with custom configuration and FHIR version
    pub async fn with_config_and_fhir_version(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        config: EngineConfig,
        fhir_version: &str,
    ) -> Result<Self> {
        // Create specialized evaluators
        let core_evaluator = Box::new(CoreEvaluator::new());
        let navigator = Box::new(Navigator::new());
        let function_evaluator = Box::new(FunctionEvaluatorImpl::new(
            function_registry.clone(),
            model_provider.clone(),
        ));
        let operator_evaluator = Box::new(OperatorEvaluatorImpl::new());
        let collection_evaluator = Box::new(CollectionEvaluatorImpl::new());
        let lambda_evaluator = Box::new(LambdaEvaluatorImpl::new());

        let evaluator = CompositeEvaluator::new(
            core_evaluator,
            navigator,
            function_evaluator,
            operator_evaluator,
            collection_evaluator,
            lambda_evaluator,
            model_provider,
            function_registry,
            config.clone(),
        )
        .await;

        // Create terminology service with the specified FHIR version
        let terminology_service = Some(Arc::new(
            crate::registry::ConcreteTerminologyService::with_fhir_version(fhir_version),
        )
            as Arc<dyn crate::evaluator::context::TerminologyService>);

        Ok(Self {
            evaluator,
            config,
            ast_cache: RwLock::new(HashMap::new()),
            terminology_service,
        })
    }

    /// Evaluate expression with comprehensive context support (always uses wrapped/metadata evaluation)
    pub async fn evaluate(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();
        let mut type_stats = TypeResolutionStats::default();

        // Auto-detect FHIR context and transform root expressions
        let processed_expression = self.auto_detect_fhir_context(expression, context).await?;

        // Parse expression (with caching) - using processed expression
        let ast = self.parse_or_cached(&processed_expression)?;

        // Setup context with terminology service if available
        let context_with_terminology = self.setup_context_with_terminology(context);

        let wrapped_values = self
            .evaluate_ast_with_metadata(&ast, &context_with_terminology, &mut type_stats)
            .await?;

        let elapsed = start_time.elapsed();
        let metrics = EvaluationMetrics {
            total_time_us: elapsed.as_micros() as u64,
            parse_time_us: 0, // TODO: track parsing time separately
            eval_time_us: elapsed.as_micros() as u64,
            function_calls: 0, // TODO: track function calls
            model_provider_calls: type_stats.types_resolved,
            service_calls: 0,      // TODO: track service calls
            memory_allocations: 0, // TODO: track memory allocations
        };

        // Convert WrappedCollection to Collection (always a Collection per FHIRPath spec)
        // Use the collection_utils to preserve ordering information from metadata
        let result_collection = crate::wrapped::collection_utils::to_plain_collection(wrapped_values);

        Ok(EvaluationResult {
            value: result_collection,
            metrics,
            warnings: vec![], // TODO: collect warnings during evaluation
            type_stats,
        })
    }

    /// Evaluate expression and return results with type information and metadata preserved
    /// This is ideal for API responses that need type information
    pub async fn evaluate_with_metadata(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<crate::core::CollectionWithMetadata> {
        use crate::core::{CollectionWithMetadata, ResultWithMetadata, ValueTypeInfo};

        let start_time = std::time::Instant::now();
        let mut type_stats = TypeResolutionStats::default();

        // Auto-detect FHIR context and transform root expressions
        let processed_expression = self.auto_detect_fhir_context(expression, context).await?;

        // Parse expression (with caching) - using processed expression
        let ast = self.parse_or_cached(&processed_expression)?;

        // Setup context with terminology service if available
        let context_with_terminology = self.setup_context_with_terminology(context);

        let wrapped_values = self
            .evaluate_ast_with_metadata(&ast, &context_with_terminology, &mut type_stats)
            .await?;

        // Convert WrappedCollection to CollectionWithMetadata preserving all type information
        let mut results = Vec::new();
        for wrapped_value in wrapped_values {
            let type_info = ValueTypeInfo {
                type_name: wrapped_value.value.type_name().to_string(),
                expected_return_type: Some(wrapped_value.metadata.fhir_type.clone()),
                cardinality: Some("0..1".to_string()),
                constraints: Vec::new(),
                is_fhir_type: matches!(
                    wrapped_value.value,
                    crate::core::FhirPathValue::Resource(_)
                ),
                namespace: wrapped_value
                    .metadata
                    .resource_type
                    .as_ref()
                    .map(|_| "FHIR".to_string()),
            };

            // Create metadata JSON with path information
            let metadata_json = serde_json::json!({
                "path": wrapped_value.metadata.path.to_string(),
                "fhir_type": wrapped_value.metadata.fhir_type,
                "resource_type": wrapped_value.metadata.resource_type,
                "index": wrapped_value.metadata.index
            });

            let result = ResultWithMetadata::new(wrapped_value.value, type_info)
                .with_metadata(metadata_json);
            results.push(result);
        }

        Ok(CollectionWithMetadata::from_results(results))
    }

    /// Evaluate expression with variables
    pub async fn evaluate_with_variables(
        &mut self,
        expression: &str,
        collection: &Collection,
        variables: HashMap<String, FhirPathValue>,
        _builtin_variables: Option<HashMap<String, FhirPathValue>>,
        _terminology_service: Option<Arc<dyn crate::evaluator::TerminologyService>>,
    ) -> Result<Collection> {
        let mut context = EvaluationContext::new(collection.clone());

        // Add variables to context
        for (name, value) in variables {
            context.set_variable(name, value);
        }

        let result = self.evaluate(expression, &context).await?;
        // Return the Collection directly (already a Collection per FHIRPath spec)
        Ok(result.value)
    }

    /// Evaluate pre-parsed AST for maximum performance
    pub async fn evaluate_ast(
        &mut self,
        ast: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Setup context with terminology service if available
        let context_with_terminology = self.setup_context_with_terminology(context);

        // Dispatch to appropriate evaluator based on expression type
        self.dispatch_evaluation(ast, &context_with_terminology)
            .await
    }

    /// Dispatch evaluation to the unified metadata-aware evaluator
    fn dispatch_evaluation<'a>(
        &'a mut self,
        expr: &'a ExpressionNode,
        context: &'a EvaluationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + 'a>>
    {
        Box::pin(async move {
            // Use unified metadata-aware evaluation and convert result to plain FhirPathValue
            let wrapped_result = self.evaluator.evaluate_with_metadata(expr, context).await?;

            // Convert wrapped collection to plain FhirPathValue
            match wrapped_result.len() {
                0 => Ok(FhirPathValue::Empty),
                1 => Ok(wrapped_result.into_iter().next().unwrap().value),
                _ => Ok(FhirPathValue::Collection(
                    wrapped_result.into_iter().map(|w| w.value).collect(),
                )),
            }
        })
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

    /// Get engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
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

    /// Evaluate expression with comprehensive metadata
    // evaluate_with_metadata removed - now all evaluation uses metadata via the main evaluate() method

    async fn evaluate_ast_with_metadata(
        &mut self,
        ast: &ExpressionNode,
        context: &EvaluationContext,
        type_stats: &mut TypeResolutionStats,
    ) -> Result<WrappedCollection> {
        if let Ok(wrapped_result) = self.evaluator.evaluate_with_metadata(ast, context).await {
            type_stats.types_resolved += wrapped_result.len();
            Ok(wrapped_result)
        } else {
            let plain_result = self.evaluate_ast(ast, context).await?;
            let wrapped_result = self.wrap_plain_result(plain_result).await?;
            type_stats.types_inferred += wrapped_result.len();
            Ok(wrapped_result)
        }
    }

    async fn wrap_plain_result(&self, result: FhirPathValue) -> Result<WrappedCollection> {
        use crate::typing::type_utils;

        match result {
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            FhirPathValue::Collection(values) => {
                let wrapped_values: Vec<WrappedValue> = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let fhir_type = type_utils::fhirpath_value_to_fhir_type(&value);
                        let resource_type = self.extract_resource_type(&value);
                        let path = if let Some(ref res_type) = resource_type {
                            CanonicalPath::root(res_type.clone())
                        } else {
                            CanonicalPath::parse(&format!("[{}]", i)).unwrap()
                        };
                        let metadata = ValueMetadata {
                            fhir_type: resource_type.as_ref().unwrap_or(&fhir_type).clone(),
                            resource_type: resource_type.clone(),
                            path,
                            index: if resource_type.is_some() { None } else { Some(i) },
                            is_ordered: None,
                        };
                        WrappedValue::new(value, metadata)
                    })
                    .collect();
                Ok(wrapped_values)
            }
            single_value => {
                let fhir_type = type_utils::fhirpath_value_to_fhir_type(&single_value);
                let resource_type = self.extract_resource_type(&single_value);
                let path = if let Some(ref res_type) = resource_type {
                    CanonicalPath::root(res_type.clone())
                } else {
                    CanonicalPath::empty()
                };
                let metadata = ValueMetadata {
                    fhir_type: resource_type.as_ref().unwrap_or(&fhir_type).clone(),
                    resource_type: resource_type.clone(),
                    path,
                    index: None,
                    is_ordered: None,
                };
                Ok(collection_utils::single(WrappedValue::new(
                    single_value,
                    metadata,
                )))
            }
        }
    }

    /// Extract resourceType from FhirPathValue if it's a FHIR resource
    fn extract_resource_type(&self, value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::JsonValue(json) | FhirPathValue::Resource(json) => {
                if let serde_json::Value::Object(obj) = json.as_ref() {
                    obj.get("resourceType")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get model provider from evaluator
    pub fn get_model_provider(&self) -> Arc<dyn ModelProvider> {
        self.evaluator.model_provider().clone()
    }

    /// Get function registry from evaluator  
    pub fn get_function_registry(&self) -> Arc<FunctionRegistry> {
        self.evaluator.function_registry().clone()
    }

    /// Setup evaluation context with terminology service and %terminologies variable
    fn setup_context_with_terminology(&self, context: &EvaluationContext) -> EvaluationContext {
        let mut context_with_terminology = context.clone();

        if let Some(ref terminology_service) = self.terminology_service {
            // Set the terminology service in builtin_variables
            context_with_terminology.builtin_variables.terminologies =
                Some(terminology_service.clone());

            // Set the %terminologies variable as a TypeInfoObject so it can be referenced
            let terminologies_var = crate::core::FhirPathValue::TypeInfoObject {
                namespace: "fhir".to_string(),
                name: "terminologies".to_string(),
            };
            context_with_terminology.set_variable("%terminologies".to_string(), terminologies_var);
        }

        context_with_terminology
    }

    /// Auto-detect FHIR context and transform root expressions
    /// This enables property validation for expressions like "name.given1" by
    /// transforming them to "Patient.name.given1" based on the input resourceType
    async fn auto_detect_fhir_context(
        &self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<String> {
        // Only process non-empty expressions
        if expression.trim().is_empty() {
            return Ok(expression.to_string());
        }

        // Extract resourceType from input JSON if available
        let resource_type = self.extract_resource_type_from_context(context);
        
        if let Some(resource_type) = resource_type {
            // Check if this is a root expression that doesn't already start with a resource type
            if self.is_root_expression_without_context(expression, &resource_type) {
                // Transform "name.given" to "Patient.name.given"
                return Ok(format!("{}.{}", resource_type, expression.trim()));
            }
        }

        // Return original expression if no transformation needed
        Ok(expression.to_string())
    }

    /// Extract resourceType from the evaluation context's start_context JSON
    fn extract_resource_type_from_context(&self, context: &EvaluationContext) -> Option<String> {
        // Get the first item in the collection (the root resource)
        if let Some(first_value) = context.start_context.first() {
            if let crate::core::FhirPathValue::JsonValue(json_val) = first_value {
                // Try to extract resourceType property from JSON
                if let Some(resource_type) = json_val.get("resourceType") {
                    if let Some(resource_type_str) = resource_type.as_str() {
                        return Some(resource_type_str.to_string());
                    }
                }
            }
        }
        None
    }

    /// Check if this is a root expression that needs FHIR context transformation
    /// Returns true for expressions like "name.given" but false for "Patient.name.given"
    fn is_root_expression_without_context(&self, expression: &str, resource_type: &str) -> bool {
        let trimmed = expression.trim();
        
        // Skip if already has resource type prefix
        if trimmed.starts_with(&format!("{}.", resource_type)) {
            return false;
        }
        
        // Skip if starts with other common FHIR resource types (basic check)
        let common_resource_types = [
            "Patient", "Observation", "Encounter", "Practitioner", 
            "Organization", "Medication", "Bundle", "DiagnosticReport",
            "Condition", "Procedure", "AllergyIntolerance", "Device"
        ];
        
        for res_type in &common_resource_types {
            if trimmed.starts_with(&format!("{}.", res_type)) {
                return false;
            }
        }
        
        // Skip expressions that start with special characters or functions
        if trimmed.starts_with('(') || trimmed.starts_with('"') || trimmed.starts_with('\'') {
            return false;
        }
        
        // Skip expressions that are resolve() calls or contain resolve()
        if trimmed.contains("resolve(") {
            return false;
        }
        
        // This looks like a root property access that needs context
        true
    }
}

/// Helper function to create engine with empty provider for testing
pub async fn create_engine_with_mock_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;
    let registry = Arc::new(create_standard_registry().await);
    let provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, provider).await
}
