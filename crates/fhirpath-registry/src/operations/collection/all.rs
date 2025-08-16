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

//! All function implementation for FHIRPath

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    lambda::{ExpressionEvaluator, LambdaContextBuilder, LambdaFunction, LambdaUtils},
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// All function: returns true if criteria is true for all items in the collection
pub struct AllFunction;

impl Default for AllFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AllFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("all", OperationType::Function)
            .description("Returns true if the criteria evaluates to true for all items in the collection. Returns true for an empty collection.")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .example("Patient.name.all(use = 'official')")
            .example("Bundle.entry.all(resource.resourceType = 'Patient')")
            .example("telecom.all(system = 'phone')")
            .build()
    }

    fn to_boolean(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Empty => Ok(false),
            FhirPathValue::Boolean(b) => Ok(*b),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(false)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    Ok(true) // Non-empty collection is truthy
                }
            }
            _ => Ok(true), // Non-empty, non-boolean values are truthy
        }
    }
}

#[async_trait]
impl FhirPathOperation for AllFunction {
    fn identifier(&self) -> &str {
        "all"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AllFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: FhirPathOperation::identifier(self).to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Extract predicate - in proper lambda implementation, this would be an expression tree
        let predicate = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Empty collection - all() returns true for empty collections
                if items.is_empty() {
                    return Ok(FhirPathValue::Boolean(true));
                }

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this variable set to current item
                    let mut lambda_context = context.clone();
                    lambda_context.set_variable("$this".to_string(), item.clone());
                    lambda_context
                        .set_variable("$index".to_string(), FhirPathValue::Integer(index as i64));
                    let lambda_context = lambda_context.with_input(item.clone());

                    // Evaluate predicate in lambda context
                    let predicate_result = match predicate {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::String(s) if s.as_ref() == "true" => true,
                        FhirPathValue::String(s) if s.as_ref() == "false" => false,
                        _ => {
                            // Mock: if predicate is a string that matches a field in the item, check if that field exists
                            if let (
                                FhirPathValue::String(field_name),
                                FhirPathValue::JsonValue(obj),
                            ) = (predicate, item)
                            {
                                obj.as_object()
                                    .map(|o| o.contains_key(field_name.as_ref()))
                                    .unwrap_or(false)
                            } else {
                                Self::to_boolean(predicate)?
                            }
                        }
                    };

                    // If any item doesn't satisfy criteria, return false
                    if !predicate_result {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                // All items satisfied the criteria
                Ok(FhirPathValue::Boolean(true))
            }
            FhirPathValue::Empty => {
                // Empty collection - all() returns true
                Ok(FhirPathValue::Boolean(true))
            }
            single_item => {
                // Single item - check criteria against it
                let predicate_result = Self::to_boolean(predicate)?;
                Ok(FhirPathValue::Boolean(predicate_result))
            }
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: FhirPathOperation::identifier(self).to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        let predicate = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Some(Ok(FhirPathValue::Boolean(true)));
                }

                for item in items.iter() {
                    let predicate_result = match predicate {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::String(s) if s.as_ref() == "true" => true,
                        FhirPathValue::String(s) if s.as_ref() == "false" => false,
                        _ => {
                            if let (
                                FhirPathValue::String(field_name),
                                FhirPathValue::JsonValue(obj),
                            ) = (predicate, item)
                            {
                                obj.as_object()
                                    .map(|o| o.contains_key(field_name.as_ref()))
                                    .unwrap_or(false)
                            } else {
                                match Self::to_boolean(predicate) {
                                    Ok(b) => b,
                                    Err(e) => return Some(Err(e)),
                                }
                            }
                        }
                    };

                    if !predicate_result {
                        return Some(Ok(FhirPathValue::Boolean(false)));
                    }
                }

                Some(Ok(FhirPathValue::Boolean(true)))
            }
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Boolean(true))),
            single_item => match Self::to_boolean(predicate) {
                Ok(predicate_result) => Some(Ok(FhirPathValue::Boolean(predicate_result))),
                Err(e) => Some(Err(e)),
            },
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl LambdaFunction for AllFunction {
    fn identifier(&self) -> &str {
        "all"
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
                // Empty collection - all() returns true for empty collections
                if items.is_empty() {
                    return Ok(FhirPathValue::Boolean(true));
                }

                // Check predicate for each item
                for (index, item) in items.iter().enumerate() {
                    // Create lambda context using LambdaContextBuilder
                    let lambda_context = LambdaContextBuilder::new(context)
                        .with_this(item.clone())
                        .with_index(index as i64)
                        .with_total(FhirPathValue::Integer(items.len() as i64))
                        .with_input(item.clone())
                        .build();

                    // Evaluate predicate expression in lambda context
                    let predicate_result = evaluator
                        .evaluate_expression(predicate_expr, &lambda_context)
                        .await?;

                    // Check if predicate is true
                    if !LambdaUtils::to_boolean(&predicate_result) {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                // All items satisfied the criteria
                Ok(FhirPathValue::Boolean(true))
            }
            FhirPathValue::Empty => {
                // Empty collection - all() returns true
                Ok(FhirPathValue::Boolean(true))
            }
            single_item => {
                // Apply all to single item using LambdaContextBuilder
                let lambda_context = LambdaContextBuilder::new(context)
                    .with_this(single_item.clone())
                    .with_index(0)
                    .with_total(FhirPathValue::Integer(1))
                    .with_input(single_item.clone())
                    .build();

                let predicate_result = evaluator
                    .evaluate_expression(predicate_expr, &lambda_context)
                    .await?;

                Ok(FhirPathValue::Boolean(LambdaUtils::to_boolean(
                    &predicate_result,
                )))
            }
        }
    }

    fn supports_sync(&self) -> bool {
        false // Expression evaluation is inherently async
    }

    fn complexity_hint(&self) -> crate::operation::OperationComplexity {
        crate::operation::OperationComplexity::Linear // O(n) for collection checking
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
