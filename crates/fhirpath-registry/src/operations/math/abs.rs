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

//! Absolute value function implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Absolute value function
#[derive(Debug, Clone)]
pub struct AbsFunction;

impl Default for AbsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AbsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("abs", OperationType::Function)
            .description("Returns the absolute value of a numeric value")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal,
            ]))
            .example("(-5).abs()")
            .example("(-3.14).abs()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for AbsFunction {
    fn identifier(&self) -> &str {
        "abs"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AbsFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => Ok(FhirPathValue::Integer(n.abs())),
            FhirPathValue::Decimal(n) => Ok(FhirPathValue::Decimal(n.abs())),
            FhirPathValue::Quantity(q) => {
                let abs_value = q.value.abs();
                Ok(FhirPathValue::quantity(abs_value, q.unit.clone()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(
                        c.first().unwrap().clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );
                    self.evaluate(args, &item_context).await
                } else {
                    Err(FhirPathError::TypeError {
                        message: "abs() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "abs() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            }));
        }

        match &context.input {
            FhirPathValue::Integer(n) => Some(Ok(FhirPathValue::Integer(n.abs()))),
            FhirPathValue::Decimal(n) => Some(Ok(FhirPathValue::Decimal(n.abs()))),
            FhirPathValue::Quantity(q) => {
                let abs_value = q.value.abs();
                Some(Ok(FhirPathValue::quantity(abs_value, q.unit.clone())))
            }
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(
                        c.first().unwrap().clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );
                    self.try_evaluate_sync(args, &item_context)
                } else {
                    Some(Err(FhirPathError::TypeError {
                        message: "abs() can only be applied to single numeric values".to_string(),
                    }))
                }
            }
            _ => Some(Err(FhirPathError::TypeError {
                message: format!(
                    "abs() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            })),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
