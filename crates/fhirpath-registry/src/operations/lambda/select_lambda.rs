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

//! Select lambda function implementation - transforms collection elements using expression trees

use crate::{LambdaFunction, ExpressionEvaluator, FhirPathOperation};
use crate::metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType};
use crate::operation::OperationComplexity;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_ast::ExpressionNode;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Select lambda function - transforms each element in a collection using expression evaluation
#[derive(Debug, Clone)]
pub struct SelectLambdaFunction;

impl SelectLambdaFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("select", OperationType::Function)
            .description("Transforms each element in a collection using the provided lambda expression")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("Patient.name.select(given)")
            .example("Bundle.entry.select(resource)")
            .example("telecom.select(value)")
            .example("items.select($this.name)")
            .build()
    }
}

#[async_trait]
impl LambdaFunction for SelectLambdaFunction {
    fn identifier(&self) -> &str {
        "select"
    }

    async fn evaluate_lambda(
        &self,
        expressions: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        // Validate we have exactly one expression (the transform expression)
        if expressions.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 1,
                actual: expressions.len(),
            });
        }

        let transform_expr = &expressions[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this variable set to current item
                    let mut lambda_context = context.clone();
                    lambda_context.set_variable("$this".to_string(), item.clone());
                    lambda_context.set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
                    lambda_context.set_variable("$total".to_string(), FhirPathValue::Integer(items.len() as i64));
                    
                    // Set the current item as the input context for the expression
                    let lambda_context = lambda_context.with_input(item.clone());

                    // Evaluate the transform expression in the lambda context
                    let transform_result = evaluator
                        .evaluate_expression(transform_expr, &lambda_context)
                        .await?;

                    // Collect the transformation result
                    match transform_result {
                        FhirPathValue::Collection(items) => {
                            results.extend(items.iter().cloned());
                        }
                        other => {
                            if !matches!(other, FhirPathValue::Empty) {
                                results.push(other);
                            }
                        }
                    }
                }

                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            single_item => {
                // Apply select to single item
                let mut lambda_context = context.clone();
                lambda_context.set_variable("$this".to_string(), single_item.clone());
                lambda_context.set_variable("$index".to_string(), FhirPathValue::Integer(0));
                lambda_context.set_variable("$total".to_string(), FhirPathValue::Integer(1));
                
                let lambda_context = lambda_context.with_input(single_item.clone());

                let transform_result = evaluator
                    .evaluate_expression(transform_expr, &lambda_context)
                    .await?;

                Ok(transform_result)
            }
        }
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

    fn expected_expression_count(&self) -> usize {
        1
    }

    fn complexity_hint(&self) -> OperationComplexity {
        OperationComplexity::Linear
    }

    fn is_pure(&self) -> bool {
        true
    }
}

#[async_trait]
impl FhirPathOperation for SelectLambdaFunction {
    fn identifier(&self) -> &str {
        "select"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SelectLambdaFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // This should not be called for lambda functions - they should be routed to evaluate_lambda
        Err(FhirPathError::EvaluationError {
            message: format!(
                "select() is a lambda function and should be called via evaluate_lambda, not evaluate. Got {} pre-evaluated args.",
                args.len()
            ),
        })
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        Some(Err(FhirPathError::EvaluationError {
            message: format!(
                "select() is a lambda function and should be called via evaluate_lambda, not evaluate. Got {} pre-evaluated args.",
                args.len()
            ),
        }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_ast::{ExpressionNode, PropertyData};
    use std::sync::Arc;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use crate::FhirPathRegistry;

    // Mock evaluator for testing
    struct MockEvaluator;

    #[async_trait]
    impl ExpressionEvaluator for MockEvaluator {
        async fn evaluate_expression(
            &self,
            expression: &ExpressionNode,
            context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            // Simple mock: if it's a property access, return the property from the input
            match expression {
                ExpressionNode::Property(prop_data) => {
                    match &context.input {
                        FhirPathValue::JsonValue(obj) => {
                            if let Some(value) = obj.get(&prop_data.name) {
                                Ok(FhirPathValue::from(value.clone()))
                            } else {
                                Ok(FhirPathValue::Empty)
                            }
                        }
                        _ => Ok(FhirPathValue::Empty)
                    }
                }
                _ => Ok(FhirPathValue::Empty)
            }
        }
    }

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_select_lambda_basic() {
        let select_fn = SelectLambdaFunction::new();
        let evaluator = MockEvaluator;

        // Create test data: collection of objects with name property
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::JsonValue(serde_json::json!({"name": "John"}).into()),
            FhirPathValue::JsonValue(serde_json::json!({"name": "Jane"}).into()),
        ]);

        let context = create_test_context(collection);

        // Create expression: select name property
        let name_expr = ExpressionNode::Property(PropertyData {
            name: "name".to_string(),
        });

        let result = select_fn
            .evaluate_lambda(&[name_expr], &context, &evaluator)
            .await
            .unwrap();

        // Should return collection of names
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_select_lambda_single_item() {
        let select_fn = SelectLambdaFunction::new();
        let evaluator = MockEvaluator;

        let single_item = FhirPathValue::JsonValue(serde_json::json!({"name": "John"}).into());
        let context = create_test_context(single_item);

        let name_expr = ExpressionNode::Property(PropertyData {
            name: "name".to_string(),
        });

        let result = select_fn
            .evaluate_lambda(&[name_expr], &context, &evaluator)
            .await
            .unwrap();

        // Should return the transformed single item
        assert!(!matches!(result, FhirPathValue::Empty));
    }

    #[tokio::test]
    async fn test_select_lambda_validation() {
        let select_fn = SelectLambdaFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let evaluator = MockEvaluator;

        // Test with wrong number of expressions
        let result = select_fn.evaluate_lambda(&[], &context, &evaluator).await;
        assert!(result.is_err());

        let name_expr = ExpressionNode::Property(PropertyData {
            name: "name".to_string(),
        });
        let result = select_fn
            .evaluate_lambda(&[name_expr.clone(), name_expr], &context, &evaluator)
            .await;
        assert!(result.is_err());
    }
}