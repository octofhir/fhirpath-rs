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

//! Select function implementation - transforms collection elements

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    lambda::{ExpressionEvaluator, LambdaContextBuilder, LambdaFunction},
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Select function - transforms each element in a collection
#[derive(Debug, Clone)]
pub struct SelectFunction;

impl Default for SelectFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("select", OperationType::Function)
            .description("Transforms each element in a collection using the provided expression")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("Patient.name.select(given)")
            .example("Bundle.entry.select(resource)")
            .example("telecom.select(value)")
            .build()
    }

    fn apply_transform(item: &FhirPathValue, transform: &FhirPathValue) -> Result<FhirPathValue> {
        match transform {
            // Mock transformation: if transform is a string, extract that field from object
            FhirPathValue::String(field_name) => match item {
                FhirPathValue::JsonValue(obj) => {
                    if let Some(value) = obj.get(field_name.as_ref()) {
                        Ok(FhirPathValue::from(value.clone()))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
                _ => Ok(FhirPathValue::Empty),
            },
            // If transform is a function name as string, apply simple transforms
            FhirPathValue::String(func_name) if func_name.as_ref() == "upper" => match item {
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase().into())),
                _ => Ok(item.clone()),
            },
            FhirPathValue::String(func_name) if func_name.as_ref() == "lower" => match item {
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_lowercase().into())),
                _ => Ok(item.clone()),
            },
            // Direct value transformation
            _ => Ok(transform.clone()),
        }
    }
}

#[async_trait]
impl FhirPathOperation for SelectFunction {
    fn identifier(&self) -> &str {
        "select"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(SelectFunction::create_metadata);
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

        let transform = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut transformed_items = Vec::new();

                for item in items.iter() {
                    // Create new context for each item
                    let _item_context = EvaluationContext::new(
                        item.clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );

                    // Apply transformation
                    let transformed = Self::apply_transform(item, transform)?;

                    // Add non-empty results to collection
                    match transformed {
                        FhirPathValue::Empty => {} // Skip empty results
                        FhirPathValue::Collection(inner_items) => {
                            transformed_items.extend(inner_items.iter().cloned());
                        }
                        single_result => transformed_items.push(single_result),
                    }
                }

                if transformed_items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if transformed_items.len() == 1 {
                    Ok(transformed_items.into_iter().next().unwrap())
                } else {
                    Ok(FhirPathValue::collection(transformed_items))
                }
            }
            single_item => {
                // Apply select to single item
                Self::apply_transform(single_item, transform)
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

        let transform = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut transformed_items = Vec::new();

                for item in items.iter() {
                    match Self::apply_transform(item, transform) {
                        Ok(transformed) => match transformed {
                            FhirPathValue::Empty => {}
                            FhirPathValue::Collection(inner_items) => {
                                transformed_items.extend(inner_items.iter().cloned());
                            }
                            single_result => transformed_items.push(single_result),
                        },
                        Err(e) => return Some(Err(e)),
                    }
                }

                if transformed_items.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if transformed_items.len() == 1 {
                    Some(Ok(transformed_items.into_iter().next().unwrap()))
                } else {
                    Some(Ok(FhirPathValue::collection(transformed_items)))
                }
            }
            single_item => Some(Self::apply_transform(single_item, transform)),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl LambdaFunction for SelectFunction {
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
                    // Create lambda context using LambdaContextBuilder
                    let lambda_context = LambdaContextBuilder::new(context)
                        .with_this(item.clone())
                        .with_index(index as i64)
                        .with_total(FhirPathValue::Integer(items.len() as i64))
                        .with_input(item.clone())
                        .build();

                    // Evaluate the transform expression in the lambda context
                    let transform_result = evaluator
                        .evaluate_expression(transform_expr, &lambda_context)
                        .await?;

                    // Collect the transformation result
                    match transform_result {
                        FhirPathValue::Collection(items) => {
                            // Filter out empty items when extending from collections
                            for item in items.iter() {
                                match item {
                                    FhirPathValue::Empty => {
                                        // Skip empty items
                                    }
                                    FhirPathValue::Collection(inner_items)
                                        if inner_items.is_empty() =>
                                    {
                                        // Skip empty collections
                                    }
                                    FhirPathValue::JsonValue(json_val)
                                        if json_val.as_json().is_null() =>
                                    {
                                        // Skip null JSON values
                                    }
                                    _ => {
                                        results.push(item.clone());
                                    }
                                }
                            }
                        }
                        other => {
                            // Filter out empty results and empty collections
                            match &other {
                                FhirPathValue::Empty => {
                                    // Skip empty values
                                }
                                FhirPathValue::Collection(items) if items.is_empty() => {
                                    // Skip empty collections
                                }
                                FhirPathValue::JsonValue(json_val)
                                    if json_val.as_json().is_null() =>
                                {
                                    // Skip null JSON values
                                }
                                _ => {
                                    results.push(other);
                                }
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
                // Apply select to single item using LambdaContextBuilder
                let lambda_context = LambdaContextBuilder::new(context)
                    .with_this(single_item.clone())
                    .with_index(0)
                    .with_total(FhirPathValue::Integer(1))
                    .with_input(single_item.clone())
                    .build();

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

    fn complexity_hint(&self) -> crate::operation::OperationComplexity {
        crate::operation::OperationComplexity::Linear
    }

    fn is_pure(&self) -> bool {
        true
    }
}
