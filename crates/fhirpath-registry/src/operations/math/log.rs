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

//! Logarithm function implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Logarithm function with specified base
#[derive(Debug, Clone)]
pub struct LogFunction;

impl Default for LogFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl LogFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("log", OperationType::Function)
            .description("Returns the logarithm of the input with the specified base")
            .parameter(
                "base",
                TypeConstraint::OneOf(vec![FhirPathType::Integer, FhirPathType::Decimal]),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .example("(100).log(10)")
            .example("(8).log(2)")
            .build()
    }

    fn extract_numeric_value(&self, value: &FhirPathValue) -> Result<Option<f64>> {
        match value {
            FhirPathValue::Integer(i) => Ok(Some(*i as f64)),
            FhirPathValue::Decimal(d) => Ok(Some(d.to_f64().unwrap_or(0.0))),
            FhirPathValue::Empty => Ok(None), // Empty returns None to indicate empty result
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(None) // Empty collection returns None per FHIRPath spec
                } else if c.len() == 1 {
                    self.extract_numeric_value(c.first().unwrap())
                } else {
                    Err(FhirPathError::TypeError {
                        message: "log() requires single numeric argument".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: "log() requires numeric arguments".to_string(),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for LogFunction {
    fn identifier(&self) -> &str {
        "log"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(LogFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let input_value = match &context.input {
            FhirPathValue::Integer(n) => *n as f64,
            FhirPathValue::Decimal(n) => n.to_f64().unwrap_or(0.0),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(
                        c.first().unwrap().clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );
                    return self.evaluate(args, &item_context).await;
                } else {
                    return Err(FhirPathError::TypeError {
                        message: "log() can only be applied to single numeric values".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "log() can only be applied to numeric values, got {}",
                        context.input.type_name()
                    ),
                });
            }
        };

        let base_value = match self.extract_numeric_value(&args[0])? {
            Some(val) => val,
            None => return Ok(FhirPathValue::Empty), // If base is empty, result is empty per FHIRPath spec
        };

        // Check for invalid inputs
        if input_value <= 0.0 {
            return Ok(FhirPathValue::Empty); // log of non-positive number is undefined
        }
        if base_value <= 0.0 || base_value == 1.0 {
            return Ok(FhirPathValue::Empty); // invalid base
        }

        let result = input_value.log(base_value);
        if result.is_finite() {
            Ok(FhirPathValue::Decimal(
                Decimal::try_from(result).unwrap_or_default(),
            ))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        let input_value = match &context.input {
            FhirPathValue::Integer(n) => *n as f64,
            FhirPathValue::Decimal(n) => n.to_f64().unwrap_or(0.0),
            FhirPathValue::Empty => return Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    return Some(Ok(FhirPathValue::Empty));
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(
                        c.first().unwrap().clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );
                    return self.try_evaluate_sync(args, &item_context);
                } else {
                    return Some(Err(FhirPathError::TypeError {
                        message: "log() can only be applied to single numeric values".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::TypeError {
                    message: format!(
                        "log() can only be applied to numeric values, got {}",
                        context.input.type_name()
                    ),
                }));
            }
        };

        let base_value = match self.extract_numeric_value(&args[0]) {
            Ok(Some(val)) => val,
            Ok(None) => return Some(Ok(FhirPathValue::Empty)), // If base is empty, result is empty per FHIRPath spec
            Err(e) => return Some(Err(e)),
        };

        // Check for invalid inputs
        if input_value <= 0.0 {
            return Some(Ok(FhirPathValue::Empty)); // log of non-positive number is undefined
        }
        if base_value <= 0.0 || base_value == 1.0 {
            return Some(Ok(FhirPathValue::Empty)); // invalid base
        }

        let result = input_value.log(base_value);
        if result.is_finite() {
            Some(Ok(FhirPathValue::Decimal(
                Decimal::try_from(result).unwrap_or_default(),
            )))
        } else {
            Some(Ok(FhirPathValue::Empty))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
