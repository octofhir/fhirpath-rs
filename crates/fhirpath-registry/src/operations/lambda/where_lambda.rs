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

//! Lambda implementation of where function - filters collection based on expression predicate

use crate::{
    lambda::{ExpressionEvaluator, LambdaContextBuilder, LambdaFunction, LambdaUtils},
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
    operation::{FhirPathOperation, OperationComplexity},
    operations::EvaluationContext,
};
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Lambda-based Where function implementation
///
/// This implementation receives expression trees instead of pre-evaluated values,
/// allowing for proper lambda variable scoping with `$this`, `$index`, etc.
#[derive(Debug, Clone)]
pub struct WhereLambdaFunction;

impl WhereLambdaFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("where", OperationType::Function)
            .description(
                "Filters a collection based on a boolean predicate expression with lambda support",
            )
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .parameter("predicate", TypeConstraint::Any, false)
            .example("Patient.name.where($this.use = 'official')")
            .example("Bundle.entry.where($this.resource.resourceType = 'Patient')")
            .example("telecom.where($this.system = 'phone' and $this.value.exists())")
            .example("children().where($this.exists() and $index < 5)")
            .build()
    }
}

#[async_trait]
impl LambdaFunction for WhereLambdaFunction {
    fn identifier(&self) -> &str {
        "where"
    }

    async fn evaluate_lambda(
        &self,
        expressions: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        if expressions.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 1,
                actual: expressions.len(),
            });
        }

        let predicate_expr = &expressions[0];

        // Handle input based on type
        match &context.input {
            FhirPathValue::Collection(items) => {
                // Filter collection based on predicate
                let mut filtered_items = Vec::new();

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this variable set to current item
                    let lambda_context = LambdaContextBuilder::new(context)
                        .with_this(item.clone())
                        .with_index(index as i64)
                        .with_input(item.clone())
                        .build();

                    // Evaluate predicate expression in lambda context
                    let predicate_result = evaluator
                        .evaluate_expression(predicate_expr, &lambda_context)
                        .await?;

                    // Check if predicate is true
                    if LambdaUtils::to_boolean(&predicate_result) {
                        filtered_items.push(item.clone());
                    }
                }

                // Return results
                if filtered_items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(filtered_items))
                }
            }
            single_item => {
                // Apply where to single item
                let lambda_context = LambdaContextBuilder::new(context)
                    .with_this(single_item.clone())
                    .with_index(0)
                    .with_input(single_item.clone())
                    .build();

                let predicate_result = evaluator
                    .evaluate_expression(predicate_expr, &lambda_context)
                    .await?;

                if LambdaUtils::to_boolean(&predicate_result) {
                    Ok(single_item.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    fn supports_sync(&self) -> bool {
        false // Expression evaluation is inherently async
    }

    fn complexity_hint(&self) -> OperationComplexity {
        OperationComplexity::Linear // O(n) for collection filtering
    }

    fn expected_expression_count(&self) -> usize {
        1
    }

    fn validate_expressions(&self, expressions: &[ExpressionNode]) -> Result<()> {
        if expressions.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 1,
                actual: expressions.len(),
            });
        }
        Ok(())
    }
}

#[async_trait]
impl FhirPathOperation for WhereLambdaFunction {
    fn identifier(&self) -> &str {
        "where"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        use std::sync::OnceLock;
        static METADATA: OnceLock<OperationMetadata> = OnceLock::new();
        METADATA.get_or_init(|| WhereLambdaFunction::create_metadata())
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // This should not be called for lambda functions, but provide fallback
        // In practice, the engine should call evaluate_lambda instead
        Err(FhirPathError::EvaluationError {
            message: format!(
                "where() is a lambda function and should be called via evaluate_lambda, not evaluate. Got {} pre-evaluated args.",
                args.len()
            ),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn complexity_hint(&self) -> OperationComplexity {
        LambdaFunction::complexity_hint(self)
    }

    fn is_pure(&self) -> bool {
        LambdaFunction::is_pure(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};
    use octofhir_fhirpath_model::MockModelProvider;
    use serde_json::json;
    use std::sync::Arc;

    // Mock expression evaluator for testing
    struct MockExpressionEvaluator;

    #[async_trait]
    impl ExpressionEvaluator for MockExpressionEvaluator {
        async fn evaluate_expression(
            &self,
            expression: &ExpressionNode,
            context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            // Simple mock: if expression is a boolean literal, return it
            // For more complex expressions, return true if $this exists
            match expression {
                ExpressionNode::Literal(LiteralValue::Boolean(b)) => {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)]))
                }
                _ => {
                    // Mock: return true if $this variable exists and is not empty
                    if let Some(this_value) = context.get_variable("$this") {
                        match this_value {
                            FhirPathValue::Empty => {
                                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                                    false,
                                )]))
                            }
                            _ => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                                true,
                            )])),
                        }
                    } else {
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                            false,
                        )]))
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_where_lambda_basic() {
        let func = WhereLambdaFunction::new();
        let evaluator = MockExpressionEvaluator;

        // Create collection
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("item1".into()),
            FhirPathValue::String("item2".into()),
            FhirPathValue::String("item3".into()),
        ]);

        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let context = EvaluationContext::new(collection, registry, model_provider);

        // Test with true predicate - should return all items
        let expressions = vec![ExpressionNode::Literal(LiteralValue::Boolean(true))];
        let result = func
            .evaluate_lambda(&expressions, &context, &evaluator)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected collection, got {:?}", result),
        }

        // Test with false predicate - should return empty
        let expressions = vec![ExpressionNode::Literal(LiteralValue::Boolean(false))];
        let result = func
            .evaluate_lambda(&expressions, &context, &evaluator)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_where_lambda_single_item() {
        let func = WhereLambdaFunction::new();
        let evaluator = MockExpressionEvaluator;

        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let context = EvaluationContext::new(
            FhirPathValue::String("test".into()),
            registry,
            model_provider,
        );

        // True predicate should return the item
        let expressions = vec![ExpressionNode::Literal(LiteralValue::Boolean(true))];
        let result = func
            .evaluate_lambda(&expressions, &context, &evaluator)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("test".into()));

        // False predicate should return empty
        let expressions = vec![ExpressionNode::Literal(LiteralValue::Boolean(false))];
        let result = func
            .evaluate_lambda(&expressions, &context, &evaluator)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_where_lambda_invalid_args() {
        let func = WhereLambdaFunction::new();
        let evaluator = MockExpressionEvaluator;

        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let context = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        // No arguments
        let result = func.evaluate_lambda(&[], &context, &evaluator).await;
        assert!(result.is_err());

        // Too many arguments
        let expressions = vec![
            ExpressionNode::Literal(LiteralValue::Boolean(true)),
            ExpressionNode::Literal(LiteralValue::Boolean(false)),
        ];
        let result = func
            .evaluate_lambda(&expressions, &context, &evaluator)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_where_lambda_metadata() {
        let func = WhereLambdaFunction::new();

        assert_eq!(func.identifier(), "where");
        assert_eq!(func.expected_expression_count(), 1);
        assert_eq!(func.complexity_hint(), OperationComplexity::Linear);
        assert!(func.is_pure());
        assert!(!func.supports_sync());
    }

    #[test]
    fn test_where_lambda_as_operation() {
        let func = WhereLambdaFunction::new();

        // Test operation interface
        assert_eq!(func.identifier(), "where");
        assert_eq!(func.operation_type(), OperationType::Function);

        // Should be downcasted to lambda function
        let lambda_func = func.as_any().downcast_ref::<WhereLambdaFunction>();
        assert!(lambda_func.is_some());
    }
}
