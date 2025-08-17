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

//! Square root function implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal;
use rust_decimal::prelude::ToPrimitive;

/// Square root function
#[derive(Debug, Clone)]
pub struct SqrtFunction;

impl Default for SqrtFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SqrtFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("sqrt", OperationType::Function)
            .description("Returns the square root of a numeric value")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .example("(9).sqrt()")
            .example("(16.0).sqrt()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SqrtFunction {
    fn identifier(&self) -> &str {
        "sqrt"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(SqrtFunction::create_metadata);
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
            FhirPathValue::Integer(n) => {
                if *n < 0 {
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "Cannot take square root of negative number".to_string(),
                    })
                } else {
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(n.to_f64().unwrap_or(0.0).sqrt())
                            .unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Decimal(n) => {
                if n.is_sign_negative() {
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "Cannot take square root of negative number".to_string(),
                    })
                } else {
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(n.to_f64().unwrap_or(0.0).sqrt())
                            .unwrap_or_default(),
                    ))
                }
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
                        message: "sqrt() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "sqrt() can only be applied to numeric values, got {}",
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
            FhirPathValue::Integer(n) => {
                if *n < 0 {
                    Some(Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "Cannot take square root of negative number".to_string(),
                    }))
                } else {
                    Some(Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(n.to_f64().unwrap_or(0.0).sqrt())
                            .unwrap_or_default(),
                    )))
                }
            }
            FhirPathValue::Decimal(n) => {
                if n.is_sign_negative() {
                    Some(Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "Cannot take square root of negative number".to_string(),
                    }))
                } else {
                    Some(Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(n.to_f64().unwrap_or(0.0).sqrt())
                            .unwrap_or_default(),
                    )))
                }
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
                        message: "sqrt() can only be applied to single numeric values".to_string(),
                    }))
                }
            }
            _ => Some(Err(FhirPathError::TypeError {
                message: format!(
                    "sqrt() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            })),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
