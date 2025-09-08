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
    registry::{create_standard_registry, FunctionRegistry},
};

use super::{
    config::EngineConfig,
    context::EvaluationContext,
    metrics::EvaluationMetrics,
    traits::CompositeEvaluator,
    CoreEvaluator, Navigator, OperatorEvaluatorImpl, 
    FunctionEvaluatorImpl, CollectionEvaluatorImpl, LambdaEvaluatorImpl,
};

/// Result of expression evaluation with metrics and warnings
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Resulting value from evaluation
    pub value: FhirPathValue,
    /// Performance metrics
    pub metrics: EvaluationMetrics,
    /// Any warnings generated during evaluation
    pub warnings: Vec<EvaluationWarning>,
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

/// FHIRPath evaluation engine using CompositeEvaluator architecture
pub struct FhirPathEngine {
    /// Composite evaluator that orchestrates all evaluation concerns
    evaluator: CompositeEvaluator,
    /// Engine configuration
    config: EngineConfig,
    /// AST cache for frequently used expressions
    ast_cache: RwLock<HashMap<String, Arc<ExpressionNode>>>,
}

impl FhirPathEngine {
    /// Create new engine with function registry and model provider
    pub async fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Result<Self> {
        
        // Create specialized evaluators
        let core_evaluator = Box::new(CoreEvaluator::new());
        let navigator = Box::new(Navigator::new());
        let function_evaluator = Box::new(FunctionEvaluatorImpl::new(function_registry));
        let operator_evaluator = Box::new(OperatorEvaluatorImpl::new());
        let collection_evaluator = Box::new(CollectionEvaluatorImpl::new());
        let lambda_evaluator = Box::new(LambdaEvaluatorImpl::new());
        
        // Create composite evaluator
        let evaluator = CompositeEvaluator::new(
            core_evaluator,
            navigator,
            function_evaluator,
            operator_evaluator,
            collection_evaluator,
            lambda_evaluator,
            model_provider,
        );
        
        Ok(Self {
            evaluator,
            config: EngineConfig::default(),
            ast_cache: RwLock::new(HashMap::new()),
        })
    }
    
    /// Create engine with custom configuration
    pub async fn with_config(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        config: EngineConfig,
    ) -> Result<Self> {
        let mut engine = Self::new(function_registry, model_provider).await?;
        engine.config = config;
        Ok(engine)
    }
    
    /// Evaluate expression with comprehensive context support
    pub async fn evaluate(
        &mut self,
        expression: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let start_time = std::time::Instant::now();
        
        // Parse expression (with caching)
        let ast = self.parse_or_cached(expression)?;
        
        // Evaluate using composite evaluator (validation happens during evaluation)
        let value = self.evaluate_ast(&ast, context).await?;
        
        let elapsed = start_time.elapsed();
        let metrics = EvaluationMetrics {
            total_time_us: elapsed.as_micros() as u64,
            parse_time_us: 0, // TODO: track parsing time separately
            eval_time_us: elapsed.as_micros() as u64,
            function_calls: 0, // TODO: track function calls
            model_provider_calls: 0, // TODO: track model provider calls
            service_calls: 0, // TODO: track service calls
            memory_allocations: 0, // TODO: track memory allocations
        };
        
        // Always wrap result in Collection for proper serialization
        let collection = self.value_to_collection(value);
        let collection_value = FhirPathValue::Collection(collection.into_vec());
        
        Ok(EvaluationResult {
            value: collection_value,
            metrics,
            warnings: vec![], // TODO: collect warnings during evaluation
        })
    }
    
    /// Evaluate expression with simple context (Collection)
    pub async fn evaluate_simple(
        &mut self,
        expression: &str,
        collection: &Collection,
    ) -> Result<Collection> {
        let context = EvaluationContext::new(collection.clone());
        let result = self.evaluate(expression, &context).await?;
        Ok(self.value_to_collection(result.value))
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
        Ok(self.value_to_collection(result.value))
    }
    
    /// Evaluate pre-parsed AST for maximum performance
    pub async fn evaluate_ast(
        &mut self,
        ast: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Dispatch to appropriate evaluator based on expression type
        self.dispatch_evaluation(ast, context).await
    }

    /// Dispatch evaluation to the appropriate specialized evaluator
    fn dispatch_evaluation<'a>(
        &'a mut self,
        expr: &'a ExpressionNode,
        context: &'a EvaluationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + 'a>> {
        Box::pin(async move {
        match expr {
            // Property access - delegate to navigator
            ExpressionNode::PropertyAccess(node) => {
                let object_value = self.dispatch_evaluation(&node.object, context).await?;
                self.evaluator.navigator.navigate_property(&object_value, &node.property, &*self.evaluator.model_provider)
            },
            
            // Index access - delegate to navigator  
            ExpressionNode::IndexAccess(node) => {
                let object_value = self.dispatch_evaluation(&node.object, context).await?;
                let index_value = self.dispatch_evaluation(&node.index, context).await?;
                
                // Convert index to usize
                match index_value {
                    FhirPathValue::Integer(i) if i >= 0 => {
                        self.evaluator.navigator.navigate_index(&object_value, i as usize)
                    },
                    _ => Ok(FhirPathValue::Empty),
                }
            },
            
            // Binary operations - delegate to operator evaluator
            ExpressionNode::BinaryOperation(node) => {
                let left_value = self.dispatch_evaluation(&node.left, context).await?;
                let right_value = self.dispatch_evaluation(&node.right, context).await?;
                self.evaluator.operator_evaluator.evaluate_binary_op(&left_value, &node.operator, &right_value)
            },
            
            // Unary operations - delegate to operator evaluator
            ExpressionNode::UnaryOperation(node) => {
                let operand_value = self.dispatch_evaluation(&node.operand, context).await?;
                self.evaluator.operator_evaluator.evaluate_unary_op(&node.operator, &operand_value)
            },
            
            // Function calls - delegate to function evaluator
            ExpressionNode::FunctionCall(node) => {
                let mut arg_values = Vec::new();
                for arg in &node.arguments {
                    arg_values.push(self.dispatch_evaluation(arg, context).await?);
                }
                let mut func_eval = &mut *self.evaluator.function_evaluator;
                func_eval.call_function(&node.name, &arg_values, context).await
            },
            
            // Method calls - delegate to function evaluator  
            ExpressionNode::MethodCall(node) => {
                let object_value = self.dispatch_evaluation(&node.object, context).await?;
                let mut arg_values = Vec::new();
                for arg in &node.arguments {
                    arg_values.push(self.dispatch_evaluation(arg, context).await?);
                }
                let mut func_eval = &mut *self.evaluator.function_evaluator;
                func_eval.call_method(&object_value, &node.method, &arg_values, context).await
            },
            
            // Collection literals - delegate to collection evaluator
            ExpressionNode::Collection(node) => {
                let mut element_values = Vec::new();
                for element in &node.elements {
                    element_values.push(self.dispatch_evaluation(element, context).await?);
                }
                Ok(self.evaluator.collection_evaluator.create_collection(element_values))
            },
            
            // Identifier - validate resource type if applicable, then delegate to core evaluator
            ExpressionNode::Identifier(identifier) => {
                // Check if this identifier represents a resource type that should be validated
                self.validate_identifier_resource_type(identifier, context).await?;
                
                self.evaluator.core_evaluator.evaluate(expr, context).await
            },
            
            // Basic expressions (literals, other nodes) - delegate to core evaluator  
            _ => {
                self.evaluator.core_evaluator.evaluate(expr, context).await
            }
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
    
    /// Helper to convert FhirPathValue to Collection for backward compatibility
    fn value_to_collection(&self, value: FhirPathValue) -> Collection {
        match value {
            FhirPathValue::Empty => Collection::empty(),
            FhirPathValue::Collection(vec) => Collection::from_values(vec),
            single_value => Collection::single(single_value),
        }
    }
    
    /// Get engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<HashMap<String, usize>> {
        let cache = self.ast_cache.read().map_err(|_| {
            FhirPathError::evaluation_error(crate::core::error_code::FP0001, "Failed to read AST cache")
        })?;
        
        let mut stats = HashMap::new();
        stats.insert("entries".to_string(), cache.len());
        Ok(stats)
    }
    
    /// Clear AST cache
    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self.ast_cache.write().map_err(|_| {
            FhirPathError::evaluation_error(crate::core::error_code::FP0001, "Failed to write AST cache")
        })?;
        cache.clear();
        Ok(())
    }
    
    /// Validate identifier if it represents a resource type
    async fn validate_identifier_resource_type(
        &self,
        identifier: &crate::ast::IdentifierNode,
        context: &EvaluationContext,
    ) -> Result<()> {
        let name = &identifier.name;
        
        // Only validate if identifier starts with capital letter (potential resource type)
        if let Some(first_char) = name.chars().next() {
            if first_char.is_uppercase() {
                // Check if this is a valid resource type
                if self.evaluator.model_provider.resource_type_exists(name).unwrap_or(false) {
                    // This is a resource type - validate against context data
                    if let Some(resource_type_from_data) = self.extract_resource_type_from_context(context) {
                        if name != &resource_type_from_data {
                            return Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0002,
                                format!(
                                    "Resource type mismatch: expression expects '{}' but input data has '{}'",
                                    name, resource_type_from_data
                                ),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Extract resource type from evaluation context data
    fn extract_resource_type_from_context(&self, context: &EvaluationContext) -> Option<String> {
        // Get the first value from start context
        let root_value = context.start_context.first()?;
        
        // Extract resourceType from Resource or JsonValue
        match root_value {
            FhirPathValue::Resource(json) => {
                json.as_object()?
                    .get("resourceType")?
                    .as_str()
                    .map(|s| s.to_string())
            },
            FhirPathValue::JsonValue(json) => {
                json.as_object()?
                    .get("resourceType")?
                    .as_str()
                    .map(|s| s.to_string())
            },
            _ => None,
        }
    }
}

/// Helper function to create engine with empty provider for testing
pub async fn create_engine_with_mock_provider() -> Result<FhirPathEngine> {
    use octofhir_fhir_model::EmptyModelProvider;
    let registry = Arc::new(create_standard_registry().await);
    let provider = Arc::new(EmptyModelProvider);
    FhirPathEngine::new(registry, provider).await
}