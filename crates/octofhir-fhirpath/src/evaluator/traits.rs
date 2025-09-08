//! Core evaluation traits for the new FHIRPath engine architecture
//!
//! This module provides the foundational traits that define how different aspects
//! of FHIRPath expression evaluation are handled. The design emphasizes:
//! - Direct FhirPathValue operations without unnecessary conversions
//! - Clear separation of concerns through focused traits
//! - Async-first design with efficient synchronous fallbacks
//! - Zero-copy operations where possible

use async_trait::async_trait;

use crate::{
    ast::ExpressionNode,
    core::{FhirPathError, FhirPathValue, ModelProvider, Result},
    evaluator::EvaluationContext,
};

// Note: We always depend on ModelProvider trait - no direct implementations
// The engine will provide the appropriate ModelProvider implementation

/// Core trait for evaluating FHIRPath expressions
///
/// This trait defines the fundamental evaluation interface that all evaluators must implement.
/// It operates directly on FhirPathValue to minimize conversions and maximize performance.
#[async_trait]
pub trait ExpressionEvaluator {
    /// Evaluate an expression node in the given context
    ///
    /// # Arguments
    /// * `expr` - The AST expression node to evaluate
    /// * `context` - The evaluation context containing variables and services
    ///
    /// # Returns
    /// * `FhirPathValue` - The result value (may be Empty, single value, or Collection)
    async fn evaluate(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;
    
    /// Check if this evaluator can handle the given expression type
    ///
    /// This allows for efficient dispatch to specialized evaluators.
    fn can_evaluate(&self, expr: &ExpressionNode) -> bool;
    
    /// Get the name of this evaluator for debugging and metrics
    fn evaluator_name(&self) -> &'static str;
}

/// Trait for navigating through FhirPathValue structures
///
/// Handles property access, indexing, and path-based navigation operations.
/// This trait encapsulates the logic for traversing FHIR resource structures.
pub trait ValueNavigator {
    /// Navigate to a property within a value
    ///
    /// # Arguments
    /// * `value` - The source value to navigate from
    /// * `property` - The property name to access
    /// * `provider` - Model provider for schema information
    ///
    /// # Returns
    /// * `FhirPathValue` - The property value (Empty if not found)
    fn navigate_property(
        &self,
        value: &FhirPathValue,
        property: &str,
        provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue>;

    /// Navigate to an indexed element within a collection or array
    ///
    /// # Arguments
    /// * `value` - The source value (must be indexable)
    /// * `index` - The zero-based index to access
    ///
    /// # Returns
    /// * `FhirPathValue` - The indexed value (Empty if out of bounds)
    fn navigate_index(&self, value: &FhirPathValue, index: usize) -> Result<FhirPathValue>;

    /// Navigate through a complex path expression
    ///
    /// # Arguments
    /// * `value` - The source value to navigate from
    /// * `path` - The path expression to follow
    /// * `provider` - Model provider for schema information
    ///
    /// # Returns
    /// * `FhirPathValue` - The final navigation result
    fn navigate_path(
        &self,
        value: &FhirPathValue,
        path: &str,
        provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue>;
}

/// Trait for evaluating function and method calls
///
/// This trait handles the dispatch and execution of FHIRPath functions,
/// both built-in and user-defined, with proper parameter validation.
#[async_trait]
pub trait FunctionEvaluator {
    /// Execute a function call with the given arguments
    ///
    /// # Arguments
    /// * `name` - The function name to call
    /// * `args` - The evaluated arguments to pass to the function
    /// * `context` - The evaluation context
    ///
    /// # Returns
    /// * `FhirPathValue` - The function result
    async fn call_function(
        &mut self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;

    /// Execute a method call on an object with arguments
    ///
    /// # Arguments
    /// * `object` - The object to call the method on
    /// * `method` - The method name to call
    /// * `args` - The evaluated arguments to pass to the method
    /// * `context` - The evaluation context
    ///
    /// # Returns
    /// * `FhirPathValue` - The method result
    async fn call_method(
        &mut self,
        object: &FhirPathValue,
        method: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;

    /// Check if a function with the given name exists
    fn has_function(&self, name: &str) -> bool;

    /// Get function metadata for validation and help
    fn get_function_metadata(&self, name: &str) -> Option<&crate::registry::FunctionMetadata>;
}

/// Trait for evaluating operators and type operations
///
/// Handles binary operations, unary operations, type casts, and type checks
/// with proper FHIRPath semantics.
pub trait OperatorEvaluator {
    /// Evaluate a binary operation between two values
    ///
    /// # Arguments
    /// * `left` - The left operand value
    /// * `operator` - The binary operator to apply
    /// * `right` - The right operand value
    ///
    /// # Returns
    /// * `FhirPathValue` - The operation result
    fn evaluate_binary_op(
        &self,
        left: &FhirPathValue,
        operator: &crate::ast::BinaryOperator,
        right: &FhirPathValue,
    ) -> Result<FhirPathValue>;

    /// Evaluate a unary operation on a value
    ///
    /// # Arguments
    /// * `operator` - The unary operator to apply
    /// * `operand` - The operand value
    ///
    /// # Returns
    /// * `FhirPathValue` - The operation result
    fn evaluate_unary_op(
        &self,
        operator: &crate::ast::UnaryOperator,
        operand: &FhirPathValue,
    ) -> Result<FhirPathValue>;

    /// Perform type casting on a value
    ///
    /// # Arguments
    /// * `value` - The value to cast
    /// * `target_type` - The target type name
    ///
    /// # Returns
    /// * `FhirPathValue` - The cast result (Empty if cast fails)
    fn cast_to_type(&self, value: &FhirPathValue, target_type: &str) -> Result<FhirPathValue>;

    /// Check if a value is of the specified type
    ///
    /// # Arguments
    /// * `value` - The value to check
    /// * `target_type` - The type name to check against
    ///
    /// # Returns
    /// * `bool` - True if the value is of the specified type
    fn is_of_type(&self, value: &FhirPathValue, target_type: &str) -> bool;
}

/// Trait for evaluating collection operations
///
/// Handles collection literals, unions, filtering, and other collection-specific operations
/// while maintaining proper FHIRPath collection semantics.
#[async_trait]
pub trait CollectionEvaluator {
    /// Create a collection from individual elements
    ///
    /// # Arguments
    /// * `elements` - The elements to include in the collection
    ///
    /// # Returns
    /// * `FhirPathValue` - The resulting collection (Empty if no elements)
    fn create_collection(&self, elements: Vec<FhirPathValue>) -> FhirPathValue;

    /// Union two values according to FHIRPath semantics
    ///
    /// # Arguments
    /// * `left` - The left value or collection
    /// * `right` - The right value or collection
    ///
    /// # Returns
    /// * `FhirPathValue` - The union result
    fn union_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> FhirPathValue;

    /// Filter a collection using a condition
    ///
    /// # Arguments
    /// * `collection` - The collection to filter
    /// * `condition` - The condition expression to apply
    /// * `context` - The evaluation context
    ///
    /// # Returns
    /// * `FhirPathValue` - The filtered collection
    async fn filter_collection(
        &mut self,
        collection: &FhirPathValue,
        condition: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;

    /// Get the length of a value (1 for single values, n for collections, 0 for empty)
    fn value_length(&self, value: &FhirPathValue) -> usize;

    /// Check if a collection contains a specific value
    fn contains_value(&self, collection: &FhirPathValue, value: &FhirPathValue) -> bool;
}

/// Trait for evaluating lambda expressions
///
/// Handles lambda expressions with proper variable scoping and context management
/// for operations like where(), select(), and aggregate functions.
#[async_trait]
pub trait LambdaEvaluator {
    /// Evaluate a lambda expression against a collection
    ///
    /// # Arguments
    /// * `lambda` - The lambda expression to evaluate
    /// * `collection` - The collection to evaluate against
    /// * `context` - The current evaluation context
    ///
    /// # Returns
    /// * `FhirPathValue` - The lambda evaluation result
    async fn evaluate_lambda(
        &mut self,
        lambda: &crate::ast::LambdaNode,
        collection: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;

    /// Evaluate a lambda expression for each item in a collection
    ///
    /// # Arguments
    /// * `lambda` - The lambda expression to evaluate
    /// * `collection` - The collection to iterate over
    /// * `context` - The current evaluation context
    ///
    /// # Returns
    /// * `Vec<FhirPathValue>` - Results for each collection item
    async fn map_lambda(
        &mut self,
        lambda: &crate::ast::LambdaNode,
        collection: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<Vec<FhirPathValue>>;

    /// Create a child context with lambda variable bindings
    ///
    /// # Arguments
    /// * `parent_context` - The parent evaluation context
    /// * `lambda_param` - The lambda parameter name (e.g., "$this")
    /// * `param_value` - The value to bind to the parameter
    ///
    /// # Returns
    /// * `EvaluationContext` - The child context with bindings
    fn create_lambda_context(
        &self,
        parent_context: &EvaluationContext,
        lambda_param: Option<&str>,
        param_value: &FhirPathValue,
    ) -> EvaluationContext;
}

/// Composite evaluator that orchestrates all evaluation concerns
///
/// This is the main evaluator that coordinates between specialized evaluators
/// to provide complete FHIRPath expression evaluation capabilities.
pub struct CompositeEvaluator {
    /// Core evaluator for basic expressions
    pub core_evaluator: Box<dyn ExpressionEvaluator + Send + Sync>,
    /// Navigator for property and index access
    pub navigator: Box<dyn ValueNavigator + Send + Sync>,
    /// Function and method call evaluator
    pub function_evaluator: Box<dyn FunctionEvaluator + Send + Sync>,
    /// Operator and type operation evaluator
    pub operator_evaluator: Box<dyn OperatorEvaluator + Send + Sync>,
    /// Collection operation evaluator
    pub collection_evaluator: Box<dyn CollectionEvaluator + Send + Sync>,
    /// Lambda expression evaluator
    pub lambda_evaluator: Box<dyn LambdaEvaluator + Send + Sync>,
    /// Model provider for FHIR schema information and reference resolution
    pub model_provider: std::sync::Arc<dyn ModelProvider>,
}

impl CompositeEvaluator {
    /// Create a new composite evaluator with all specialized evaluators
    pub fn new(
        core_evaluator: Box<dyn ExpressionEvaluator + Send + Sync>,
        navigator: Box<dyn ValueNavigator + Send + Sync>,
        function_evaluator: Box<dyn FunctionEvaluator + Send + Sync>,
        operator_evaluator: Box<dyn OperatorEvaluator + Send + Sync>,
        collection_evaluator: Box<dyn CollectionEvaluator + Send + Sync>,
        lambda_evaluator: Box<dyn LambdaEvaluator + Send + Sync>,
        model_provider: std::sync::Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            core_evaluator,
            navigator,
            function_evaluator,
            operator_evaluator,
            collection_evaluator,
            lambda_evaluator,
            model_provider,
        }
    }
}

#[async_trait]
impl ExpressionEvaluator for CompositeEvaluator {
    async fn evaluate(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Dispatch to the appropriate specialized evaluator based on expression type
        match expr {
            ExpressionNode::Literal(_) | ExpressionNode::Identifier(_) | ExpressionNode::Variable(_) => {
                self.core_evaluator.evaluate(expr, context).await
            }
            ExpressionNode::PropertyAccess(_) | ExpressionNode::IndexAccess(_) | ExpressionNode::Path(_) => {
                self.evaluate_navigation_expr(expr, context).await
            }
            ExpressionNode::FunctionCall(_) | ExpressionNode::MethodCall(_) => {
                self.evaluate_function_expr(expr, context).await
            }
            ExpressionNode::BinaryOperation(_) | ExpressionNode::UnaryOperation(_) | 
            ExpressionNode::TypeCast(_) | ExpressionNode::TypeCheck(_) => {
                self.evaluate_operator_expr(expr, context).await
            }
            ExpressionNode::Collection(_) | ExpressionNode::Union(_) | ExpressionNode::Filter(_) => {
                self.evaluate_collection_expr(expr, context).await
            }
            ExpressionNode::Lambda(_) => {
                // For standalone lambda, we need a collection to apply it to
                let empty_collection = FhirPathValue::Empty;
                self.lambda_evaluator.evaluate_lambda(
                    match expr {
                        ExpressionNode::Lambda(lambda) => lambda,
                        _ => unreachable!(),
                    },
                    &empty_collection,
                    context,
                ).await
            }
            ExpressionNode::Parenthesized(inner) => {
                self.evaluate(inner, context).await
            }
        }
    }

    fn can_evaluate(&self, _expr: &ExpressionNode) -> bool {
        // Composite evaluator can handle all expression types
        true
    }

    fn evaluator_name(&self) -> &'static str {
        "CompositeEvaluator"
    }
}

impl CompositeEvaluator {
    /// Handle navigation expressions (property access, indexing, path navigation)
    async fn evaluate_navigation_expr(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            ExpressionNode::PropertyAccess(prop) => {
                let object_value = self.evaluate(&prop.object, context).await?;
                self.navigator.navigate_property(&object_value, &prop.property, 
                    &*self.model_provider)
            }
            ExpressionNode::IndexAccess(idx) => {
                let object_value = self.evaluate(&idx.object, context).await?;
                let index_value = self.evaluate(&idx.index, context).await?;
                
                // Convert index to usize
                match index_value {
                    FhirPathValue::Integer(i) if i >= 0 => {
                        self.navigator.navigate_index(&object_value, i as usize)
                    }
                    _ => Ok(FhirPathValue::Empty), // Invalid index
                }
            }
            ExpressionNode::Path(path) => {
                let base_value = self.evaluate(&path.base, context).await?;
                self.navigator.navigate_path(&base_value, &path.path, 
                    &*self.model_provider)
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                format!("Invalid navigation expression: {:?}", expr),
            )),
        }
    }

    /// Handle function and method call expressions
    async fn evaluate_function_expr(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            ExpressionNode::FunctionCall(func) => {
                // Evaluate all arguments first
                let mut args = Vec::new();
                for arg in &func.arguments {
                    args.push(self.evaluate(arg, context).await?);
                }
                
                self.function_evaluator.call_function(&func.name, &args, context).await
            }
            ExpressionNode::MethodCall(method) => {
                let object_value = self.evaluate(&method.object, context).await?;
                
                // Evaluate all arguments
                let mut args = Vec::new();
                for arg in &method.arguments {
                    args.push(self.evaluate(arg, context).await?);
                }
                
                self.function_evaluator.call_method(&object_value, &method.method, &args, context).await
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                format!("Invalid function expression: {:?}", expr),
            )),
        }
    }

    /// Handle operator expressions (binary ops, unary ops, type ops)
    async fn evaluate_operator_expr(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            ExpressionNode::BinaryOperation(bin) => {
                let left_value = self.evaluate(&bin.left, context).await?;
                let right_value = self.evaluate(&bin.right, context).await?;
                
                self.operator_evaluator.evaluate_binary_op(&left_value, &bin.operator, &right_value)
            }
            ExpressionNode::UnaryOperation(un) => {
                let operand_value = self.evaluate(&un.operand, context).await?;
                
                self.operator_evaluator.evaluate_unary_op(&un.operator, &operand_value)
            }
            ExpressionNode::TypeCast(cast) => {
                let expr_value = self.evaluate(&cast.expression, context).await?;
                
                self.operator_evaluator.cast_to_type(&expr_value, &cast.target_type)
            }
            ExpressionNode::TypeCheck(check) => {
                let expr_value = self.evaluate(&check.expression, context).await?;
                let is_type = self.operator_evaluator.is_of_type(&expr_value, &check.target_type);
                
                Ok(FhirPathValue::Boolean(is_type))
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                format!("Invalid operator expression: {:?}", expr),
            )),
        }
    }

    /// Handle collection expressions (collections, unions, filters)
    async fn evaluate_collection_expr(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            ExpressionNode::Collection(coll) => {
                let mut elements = Vec::new();
                for element in &coll.elements {
                    elements.push(self.evaluate(element, context).await?);
                }
                
                Ok(self.collection_evaluator.create_collection(elements))
            }
            ExpressionNode::Union(union) => {
                let left_value = self.evaluate(&union.left, context).await?;
                let right_value = self.evaluate(&union.right, context).await?;
                
                Ok(self.collection_evaluator.union_values(&left_value, &right_value))
            }
            ExpressionNode::Filter(filter) => {
                let base_value = self.evaluate(&filter.base, context).await?;
                
                self.collection_evaluator.filter_collection(&base_value, &filter.condition, context).await
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                format!("Invalid collection expression: {:?}", expr),
            )),
        }
    }
}