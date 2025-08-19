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

//! Precision function implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Precision function - returns the number of digits of precision in a decimal value
#[derive(Debug, Clone)]
pub struct PrecisionFunction;

impl Default for PrecisionFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl PrecisionFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("precision", OperationType::Function)
            .description("Returns the number of digits of precision in a decimal value")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .example("(3.14).precision()")
            .example("(100.0).precision()")
            .build()
    }

    fn calculate_decimal_precision(&self, decimal: &rust_decimal::Decimal) -> i64 {
        // For FHIRPath precision, we need to return the number of decimal places
        // The scale() method returns the number of digits after the decimal point
        // This should preserve the original precision including trailing zeros
        decimal.scale() as i64
    }
}

#[async_trait]
impl FhirPathOperation for PrecisionFunction {
    fn identifier(&self) -> &str {
        "precision"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(PrecisionFunction::create_metadata);
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
            FhirPathValue::Integer(_) => {
                // Integers have 0 decimal places of precision
                Ok(FhirPathValue::Integer(0))
            }
            FhirPathValue::Decimal(d) => {
                let precision = self.calculate_decimal_precision(d);
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Date(date) => {
                // Use the built-in precision method from the temporal type
                let precision = date.precision_digits();
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::DateTime(datetime) => {
                // Use the built-in precision method from the temporal type
                let precision = datetime.precision_digits();
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Time(time) => {
                // Use the built-in time precision method from the temporal type
                let precision = time.precision_digits();
                Ok(FhirPathValue::Integer(precision))
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
                        message: "precision() can only be applied to single values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "precision() can only be applied to numeric, date, time, or datetime values, got {}",
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
            FhirPathValue::Integer(_) => {
                // Integers have 0 decimal places of precision
                Some(Ok(FhirPathValue::Integer(0)))
            }
            FhirPathValue::Decimal(d) => {
                let precision = self.calculate_decimal_precision(d);
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::Date(date) => {
                // Use the built-in precision method from the temporal type
                let precision = date.precision_digits();
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::DateTime(datetime) => {
                // Use the built-in precision method from the temporal type
                let precision = datetime.precision_digits();
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::Time(time) => {
                // Use the built-in time precision method from the temporal type
                let precision = time.precision_digits();
                Some(Ok(FhirPathValue::Integer(precision)))
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
                        message: "precision() can only be applied to single values".to_string(),
                    }))
                }
            }
            _ => Some(Err(FhirPathError::TypeError {
                message: format!(
                    "precision() can only be applied to numeric, date, time, or datetime values, got {}",
                    context.input.type_name()
                ),
            })),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
