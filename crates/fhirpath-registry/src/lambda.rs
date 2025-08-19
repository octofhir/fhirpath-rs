// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Lambda function architecture for FHIRPath expressions
//!
//! This module provides the lambda function trait and evaluation context
//! for functions that need to work with expression trees instead of
//! pre-evaluated arguments.

use crate::operation::{FhirPathOperation, OperationComplexity};
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rustc_hash::FxHashMap;

/// Trait for lambda functions that operate on expression trees
///
/// Lambda functions receive raw expression nodes instead of pre-evaluated values.
/// This allows them to control when and how expressions are evaluated, enabling
/// proper variable scoping for `$this`, `$index`, etc.
#[async_trait]
pub trait LambdaFunction: Send + Sync {
    /// Function identifier (same as normal functions)
    fn identifier(&self) -> &str;

    /// Evaluate lambda function with expression arguments
    ///
    /// # Arguments
    /// * `expressions` - Raw expression trees to evaluate in lambda context
    /// * `context` - Current evaluation context
    /// * `evaluator` - Callback to evaluate expressions in specific contexts
    ///
    /// # Returns
    /// The result of the lambda function evaluation
    async fn evaluate_lambda(
        &self,
        expressions: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue>;

    /// Check if this function supports sync evaluation
    fn supports_sync(&self) -> bool {
        false
    }

    /// Sync evaluation (optional)
    fn try_evaluate_lambda_sync(
        &self,
        _expressions: &[ExpressionNode],
        _context: &EvaluationContext,
        _evaluator: &dyn ExpressionEvaluator,
    ) -> Option<Result<FhirPathValue>> {
        None
    }

    /// Get complexity hint for optimization
    fn complexity_hint(&self) -> OperationComplexity {
        OperationComplexity::Linear
    }

    /// Validate expression arguments before evaluation
    fn validate_expressions(&self, expressions: &[ExpressionNode]) -> Result<()> {
        // Default implementation just checks count based on function requirements
        let expected_count = self.expected_expression_count();
        if expressions.len() != expected_count {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: expected_count,
                actual: expressions.len(),
            });
        }
        Ok(())
    }

    /// Get expected number of expression arguments
    fn expected_expression_count(&self) -> usize {
        1 // Most lambda functions take one expression
    }

    /// Check if function is pure (no side effects)
    fn is_pure(&self) -> bool {
        !matches!(self.identifier(), "trace" | "defineVariable")
    }
}

/// Trait for evaluating expressions in lambda contexts
///
/// This trait is implemented by the engine and allows lambda functions
/// to evaluate expressions with proper variable scoping.
#[async_trait]
pub trait ExpressionEvaluator: Send + Sync {
    /// Evaluate an expression with a specific context
    /// Evaluate expression with lambda variables (default implementation uses standard evaluation)
    async fn evaluate_expression_with_lambda_vars(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
        lambda_vars: &[(String, FhirPathValue)],
    ) -> Result<FhirPathValue> {
        // Default implementation: create temporary context with variables
        let mut temp_context = context.clone();
        for (name, value) in lambda_vars {
            temp_context.set_variable(name.clone(), value.clone());
        }
        self.evaluate_expression(expression, &temp_context).await
    }
    async fn evaluate_expression(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue>;

    /// Evaluate expression synchronously if possible
    fn try_evaluate_expression_sync(
        &self,
        _expression: &ExpressionNode,
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        None
    }
}

/// Lambda evaluation context builder
///
/// Helper for creating lambda contexts with proper variable scoping.
pub struct LambdaContextBuilder<'a> {
    base_context: &'a EvaluationContext,
    variables: FxHashMap<String, FhirPathValue>,
    input: Option<FhirPathValue>,
}

impl<'a> LambdaContextBuilder<'a> {
    /// Create new lambda context builder
    pub fn new(base_context: &'a EvaluationContext) -> Self {
        Self {
            base_context,
            variables: base_context.variables.clone(),
            input: None,
        }
    }

    /// Set lambda variable
    pub fn with_variable(mut self, name: String, value: FhirPathValue) -> Self {
        self.variables.insert(name, value);
        self
    }

    /// Set $this variable (current item in collection iteration)
    pub fn with_this(mut self, value: FhirPathValue) -> Self {
        // Variables are stored with the $ prefix
        self.variables.insert("$this".to_string(), value);
        self
    }

    /// Set $index variable (current index in collection iteration)
    pub fn with_index(mut self, index: i64) -> Self {
        // Variables are stored with the $ prefix
        self.variables
            .insert("$index".to_string(), FhirPathValue::Integer(index));
        self
    }

    /// Set $total variable (accumulator for aggregate functions)
    pub fn with_total(mut self, total: FhirPathValue) -> Self {
        self.variables.insert("$total".to_string(), total);
        self
    }

    /// Set input context (what expressions evaluate against)
    pub fn with_input(mut self, input: FhirPathValue) -> Self {
        self.input = Some(input);
        self
    }

    /// Build the lambda context
    pub fn build(self) -> EvaluationContext {
        let mut context = if let Some(input) = self.input {
            self.base_context.with_input(input)
        } else {
            self.base_context.clone()
        };

        // Add lambda variables
        for (name, value) in self.variables {
            context.set_variable(name, value);
        }

        context
    }
}

/// Utility functions for lambda evaluation
pub struct LambdaUtils;

impl LambdaUtils {
    /// Convert a value to boolean for predicate evaluation
    pub fn to_boolean(value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Empty => false,
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    false
                } else if items.len() == 1 {
                    // For single-item collections, extract the boolean value
                    Self::to_boolean(items.get(0).unwrap())
                } else {
                    true // Multi-item collections are truthy
                }
            }
            _ => true, // Non-empty values are truthy
        }
    }

    /// Evaluate a predicate expression for each item in a collection
    pub async fn evaluate_collection_predicate<F, Fut>(
        items: &[FhirPathValue],
        expression: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
        mut handler: F,
    ) -> Result<Vec<FhirPathValue>>
    where
        F: FnMut(&FhirPathValue, bool, usize) -> Fut + Send,
        Fut: std::future::Future<Output = Option<FhirPathValue>> + Send,
    {
        let mut results = Vec::new();

        for (index, item) in items.iter().enumerate() {
            // Create lambda context for this item
            let lambda_context = LambdaContextBuilder::new(context)
                .with_this(item.clone())
                .with_index(index as i64)
                .with_input(item.clone())
                .build();

            // Evaluate predicate
            let predicate_result = evaluator
                .evaluate_expression(expression, &lambda_context)
                .await?;

            let is_true = Self::to_boolean(&predicate_result);

            // Let handler decide what to do with this result
            if let Some(result) = handler(item, is_true, index).await {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Evaluate a transformation expression for each item in a collection
    pub async fn evaluate_collection_transform(
        items: &[FhirPathValue],
        expression: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        for (index, item) in items.iter().enumerate() {
            // Create lambda context for this item
            let lambda_context = LambdaContextBuilder::new(context)
                .with_this(item.clone())
                .with_index(index as i64)
                .with_input(item.clone())
                .build();

            // Evaluate transformation
            let result = evaluator
                .evaluate_expression(expression, &lambda_context)
                .await?;

            // Collect results
            match result {
                FhirPathValue::Collection(sub_items) => {
                    results.extend(sub_items.iter().cloned());
                }
                FhirPathValue::Empty => {
                    // Skip empty results
                }
                single_value => {
                    results.push(single_value);
                }
            }
        }

        Ok(results)
    }

    /// Aggregate over a collection using an expression
    pub async fn evaluate_collection_aggregate(
        items: &[FhirPathValue],
        expression: &ExpressionNode,
        initial: FhirPathValue,
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        let mut accumulator = initial;

        for (index, item) in items.iter().enumerate() {
            // Create lambda context for this item
            let lambda_context = LambdaContextBuilder::new(context)
                .with_this(item.clone())
                .with_index(index as i64)
                .with_total(accumulator.clone())
                .with_input(item.clone())
                .build();

            // Evaluate aggregate expression
            let result = evaluator
                .evaluate_expression(expression, &lambda_context)
                .await?;

            // Update accumulator
            accumulator = result;
        }

        Ok(accumulator)
    }
}

/// Wrapper to make regular operations compatible with lambda interface
///
/// This allows the engine to treat all operations uniformly while maintaining
/// the distinction between regular and lambda functions.
pub struct LambdaOperationWrapper {
    inner: Box<dyn FhirPathOperation>,
}

impl LambdaOperationWrapper {
    /// Wrap a regular operation
    pub fn wrap(operation: Box<dyn FhirPathOperation>) -> Self {
        Self { inner: operation }
    }

    /// Check if the wrapped operation is actually a lambda function
    pub fn is_lambda_function(&self) -> bool {
        // Check if this is one of the known lambda functions
        matches!(
            self.inner.identifier(),
            "where" | "select" | "all" | "any" | "exists" | "aggregate" | "repeat" | "sort"
        )
    }
}

#[async_trait]
impl FhirPathOperation for LambdaOperationWrapper {
    fn identifier(&self) -> &str {
        self.inner.identifier()
    }

    fn operation_type(&self) -> crate::metadata::OperationType {
        self.inner.operation_type()
    }

    fn metadata(&self) -> &crate::metadata::OperationMetadata {
        self.inner.metadata()
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        self.inner.evaluate(args, context).await
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        self.inner.try_evaluate_sync(args, context)
    }

    fn supports_sync(&self) -> bool {
        self.inner.supports_sync()
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        self.inner.validate_args(args)
    }

    fn arg_count_range(&self) -> (usize, Option<usize>) {
        self.inner.arg_count_range()
    }

    fn is_pure(&self) -> bool {
        self.inner.is_pure()
    }

    fn complexity_hint(&self) -> OperationComplexity {
        self.inner.complexity_hint()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::MockModelProvider;

    use std::sync::Arc;

    #[tokio::test]
    async fn test_lambda_context_builder() {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let base_context = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        let item = FhirPathValue::String("test".into());
        let lambda_context = LambdaContextBuilder::new(&base_context)
            .with_this(item.clone())
            .with_index(0)
            .build();

        assert_eq!(
            lambda_context.get_variable("$this"),
            Some(&FhirPathValue::String("test".into()))
        );
        assert_eq!(
            lambda_context.get_variable("$index"),
            Some(&FhirPathValue::Integer(0))
        );
    }

    #[test]
    fn test_lambda_utils_to_boolean() {
        assert!(!LambdaUtils::to_boolean(&FhirPathValue::Empty));
        assert!(LambdaUtils::to_boolean(&FhirPathValue::Boolean(true)));
        assert!(!LambdaUtils::to_boolean(&FhirPathValue::Boolean(false)));
        assert!(LambdaUtils::to_boolean(&FhirPathValue::String(
            "test".into()
        )));
        assert!(!LambdaUtils::to_boolean(&FhirPathValue::collection(vec![])));
        assert!(LambdaUtils::to_boolean(&FhirPathValue::collection(vec![
            FhirPathValue::String("item".into())
        ])));
    }
}
