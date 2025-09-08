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
    core::{FhirPathValue, ModelProvider, Result},
    evaluator::EvaluationContext,
    typing::TypeResolver,
    wrapped::{WrappedCollection, WrappedValue},
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

/// Metadata-aware trait for navigating through values with rich metadata propagation
///
/// This trait provides metadata-aware navigation that preserves type information,
/// path contexts, and other metadata throughout property access and indexing operations.
/// It is the foundation for providing rich error messages and improved CLI output.
#[async_trait]
pub trait MetadataAwareNavigator {
    /// Navigate to a property with metadata preservation and type resolution
    ///
    /// # Arguments
    /// * `source` - The wrapped source value to navigate from
    /// * `property` - The property name to access
    /// * `resolver` - Type resolver for accurate FHIR type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The property values with updated metadata
    async fn navigate_property_with_metadata(
        &self,
        source: &WrappedValue,
        property: &str,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Navigate to an indexed element with metadata preservation
    ///
    /// # Arguments
    /// * `source` - The wrapped source value to navigate from
    /// * `index` - The zero-based index to access
    /// * `resolver` - Type resolver for element type information
    ///
    /// # Returns
    /// * `Option<WrappedValue>` - The indexed value with updated metadata
    async fn navigate_index_with_metadata(
        &self,
        source: &WrappedValue,
        index: usize,
        resolver: &TypeResolver,
    ) -> Result<Option<WrappedValue>>;

    /// Navigate through multiple property steps with metadata
    ///
    /// # Arguments
    /// * `source` - The wrapped source value to navigate from
    /// * `path_segments` - The property path to follow (e.g., ["name", "given"])
    /// * `resolver` - Type resolver for type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The final navigation results with metadata
    async fn navigate_path_with_metadata(
        &self,
        source: &WrappedValue,
        path_segments: &[&str],
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;
}

/// Metadata-aware trait for evaluating expressions with rich metadata propagation
///
/// This trait provides metadata-aware expression evaluation that preserves type information,
/// path contexts, and other metadata throughout the evaluation process. It is the foundation
/// for providing accurate error messages and improved output with real FHIR types.
#[async_trait]
pub trait MetadataAwareEvaluator {
    /// Evaluate an expression with metadata propagation
    ///
    /// # Arguments
    /// * `expr` - The AST expression node to evaluate
    /// * `context` - The evaluation context
    /// * `resolver` - Type resolver for accurate FHIR type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The evaluation result with metadata
    async fn evaluate_with_metadata(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Initialize root evaluation context with metadata
    ///
    /// # Arguments
    /// * `root_data` - The root data for evaluation
    /// * `resolver` - Type resolver for root type detection
    ///
    /// # Returns
    /// * `WrappedCollection` - The root context with metadata
    async fn initialize_root_context(
        &self,
        root_data: &crate::core::Collection,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;
}

/// Metadata-aware trait for evaluating functions with rich metadata propagation
///
/// This trait provides metadata-aware function evaluation that preserves type information
/// and path contexts through function calls, enabling accurate result type resolution.
#[async_trait]
pub trait MetadataAwareFunctionEvaluator {
    /// Execute a function call with metadata-aware arguments and results
    ///
    /// # Arguments
    /// * `name` - The function name to call
    /// * `args` - The evaluated arguments with metadata
    /// * `context` - The evaluation context
    /// * `resolver` - Type resolver for result type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The function result with metadata
    async fn call_function_with_metadata(
        &mut self,
        name: &str,
        args: &[WrappedCollection],
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Execute a method call on a wrapped object with metadata
    ///
    /// # Arguments
    /// * `object` - The wrapped object to call the method on
    /// * `method` - The method name to call
    /// * `args` - The evaluated arguments with metadata
    /// * `context` - The evaluation context
    /// * `resolver` - Type resolver for result type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The method result with metadata
    async fn call_method_with_metadata(
        &mut self,
        object: &WrappedCollection,
        method: &str,
        args: &[WrappedCollection],
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;
}

/// Trait for evaluating collection operations with rich metadata propagation
#[async_trait]
pub trait MetadataAwareCollectionEvaluator {
    /// Create a collection from individual wrapped elements
    ///
    /// # Arguments
    /// * `elements` - The wrapped elements to include in the collection
    /// * `resolver` - Type resolver for collection type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The resulting collection with metadata
    async fn create_collection_with_metadata(
        &self,
        elements: Vec<WrappedCollection>,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Union two wrapped collections according to FHIRPath semantics
    ///
    /// # Arguments
    /// * `left` - The left collection with metadata
    /// * `right` - The right collection with metadata
    /// * `resolver` - Type resolver for result type information
    ///
    /// # Returns
    /// * `WrappedCollection` - The union result with metadata
    async fn union_collections_with_metadata(
        &self,
        left: &WrappedCollection,
        right: &WrappedCollection,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Filter a wrapped collection using a condition with metadata preservation
    ///
    /// # Arguments
    /// * `collection` - The wrapped collection to filter
    /// * `condition` - The condition expression to apply
    /// * `context` - The evaluation context
    /// * `resolver` - Type resolver for result metadata
    ///
    /// # Returns
    /// * `WrappedCollection` - The filtered collection with preserved metadata
    async fn filter_collection_with_metadata(
        &mut self,
        collection: &WrappedCollection,
        condition: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Check if a wrapped collection contains a specific wrapped value
    ///
    /// # Arguments
    /// * `collection` - The wrapped collection to search
    /// * `value` - The wrapped value to find
    ///
    /// # Returns
    /// * `bool` - True if the value is found in the collection
    fn contains_wrapped_value(&self, collection: &WrappedCollection, value: &WrappedValue) -> bool;
}

/// Standard trait for evaluating lambda expressions with metadata propagation
///
/// This trait provides lambda evaluation that preserves type information
/// and path contexts through lambda operations like where(), select(), and aggregate functions.
#[async_trait]
pub trait LambdaEvaluator {
    /// Evaluate a lambda expression against a wrapped collection
    ///
    /// # Arguments
    /// * `lambda` - The lambda expression to evaluate
    /// * `collection` - The wrapped collection to evaluate against
    /// * `context` - The current evaluation context
    /// * `resolver` - Type resolver for result metadata
    ///
    /// # Returns
    /// * `WrappedCollection` - The lambda evaluation result with metadata
    async fn evaluate_lambda(
        &mut self,
        lambda: &crate::ast::LambdaNode,
        collection: &WrappedCollection,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection>;

    /// Evaluate a lambda expression for each item in a wrapped collection
    ///
    /// # Arguments
    /// * `lambda` - The lambda expression to evaluate
    /// * `collection` - The wrapped collection to iterate over
    /// * `context` - The current evaluation context
    /// * `resolver` - Type resolver for result metadata
    ///
    /// # Returns
    /// * `Vec<WrappedCollection>` - Results for each collection item with metadata
    async fn map_lambda(
        &mut self,
        lambda: &crate::ast::LambdaNode,
        collection: &WrappedCollection,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<Vec<WrappedCollection>>;

    /// Create a child context with lambda variable bindings
    ///
    /// # Arguments
    /// * `parent_context` - The parent evaluation context
    /// * `lambda_param` - The lambda parameter name (e.g., "$this")
    /// * `param_value` - The wrapped value to bind to the parameter
    /// * `resolver` - Type resolver for context metadata
    ///
    /// # Returns
    /// * `EvaluationContext` - The child context with bindings
    async fn create_lambda_context(
        &self,
        parent_context: &EvaluationContext,
        lambda_param: Option<&str>,
        param_value: &WrappedValue,
        resolver: &TypeResolver,
    ) -> Result<EvaluationContext>;
}
