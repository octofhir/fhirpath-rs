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

//! Power function implementation

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

/// Power function - raises a number to an exponent
#[derive(Debug, Clone)]
pub struct PowerFunction;

impl Default for PowerFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("power", OperationType::Function)
            .description("Raises a number to the exponent power. Return type matches the more precise input type.")
            .parameter(
                "exponent",
                TypeConstraint::OneOf(vec![FhirPathType::Integer, FhirPathType::Decimal]),
                false,
            )
            .returns(TypeConstraint::OneOf(vec![FhirPathType::Integer, FhirPathType::Decimal]))
            .example("(2).power(3)")
            .example("(2.5).power(2)")
            .build()
    }

    fn extract_numeric_value(&self, value: &FhirPathValue) -> Result<NumericInput> {
        match value {
            FhirPathValue::Integer(i) => Ok(NumericInput::Integer(*i)),
            FhirPathValue::Decimal(d) => Ok(NumericInput::Decimal(*d)),
            FhirPathValue::Empty => Err(FhirPathError::TypeError {
                message: "power() requires numeric arguments, got Empty".to_string(),
            }),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    // Empty collections should be handled at a higher level, not here
                    Err(FhirPathError::TypeError {
                        message: "power() requires numeric arguments, got empty collection"
                            .to_string(),
                    })
                } else if c.len() == 1 {
                    self.extract_numeric_value(c.get(0).unwrap())
                } else {
                    Err(FhirPathError::TypeError {
                        message: "power() requires single numeric arguments, got collection with multiple items".to_string()
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "power() requires numeric arguments, got {}",
                    value.type_name()
                ),
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum NumericInput {
    Integer(i64),
    Decimal(Decimal),
}

impl NumericInput {
    fn to_f64(&self) -> f64 {
        match self {
            NumericInput::Integer(i) => *i as f64,
            NumericInput::Decimal(d) => d.to_f64().unwrap_or(0.0),
        }
    }
}

#[async_trait]
impl FhirPathOperation for PowerFunction {
    fn identifier(&self) -> &str {
        "power"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(PowerFunction::create_metadata);
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

        let base = match &context.input {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                self.extract_numeric_value(&context.input)?
            }
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
                        message: "power() can only be applied to single numeric values".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "power() can only be applied to numeric values, got {}",
                        context.input.type_name()
                    ),
                });
            }
        };

        // Handle empty argument cases
        if args[0].is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let exponent = match &args[0] {
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) if c.is_empty() => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c)
                if c.len() == 1 && matches!(c.get(0).unwrap(), FhirPathValue::Empty) =>
            {
                return Ok(FhirPathValue::Empty);
            }
            _ => self.extract_numeric_value(&args[0])?,
        };

        let base_f = base.to_f64();
        let exp_f = exponent.to_f64();
        let result_f = base_f.powf(exp_f);

        // Check for invalid results
        if !result_f.is_finite() {
            return Ok(FhirPathValue::Empty);
        }

        // Determine return type based on inputs and result
        match (&base, &exponent) {
            (NumericInput::Integer(_b), NumericInput::Integer(e)) => {
                // Integer to integer power
                if *e >= 0
                    && result_f == result_f.trunc()
                    && result_f >= i64::MIN as f64
                    && result_f <= i64::MAX as f64
                {
                    // Result fits in integer range and is a whole number
                    Ok(FhirPathValue::Integer(result_f as i64))
                } else {
                    // Result needs decimal representation
                    Ok(FhirPathValue::Decimal(
                        Decimal::try_from(result_f).unwrap_or_default(),
                    ))
                }
            }
            _ => {
                // Any decimal involvement results in decimal
                Ok(FhirPathValue::Decimal(
                    Decimal::try_from(result_f).unwrap_or_default(),
                ))
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
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        let base = match &context.input {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                match self.extract_numeric_value(&context.input) {
                    Ok(base) => base,
                    Err(e) => return Some(Err(e)),
                }
            }
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
                        message: "power() can only be applied to single numeric values".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::TypeError {
                    message: format!(
                        "power() can only be applied to numeric values, got {}",
                        context.input.type_name()
                    ),
                }));
            }
        };

        // Handle empty argument cases
        if args[0].is_empty() {
            return Some(Ok(FhirPathValue::Empty));
        }

        let exponent = match &args[0] {
            FhirPathValue::Empty => return Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) if c.is_empty() => return Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c)
                if c.len() == 1 && matches!(c.get(0).unwrap(), FhirPathValue::Empty) =>
            {
                return Some(Ok(FhirPathValue::Empty));
            }
            _ => match self.extract_numeric_value(&args[0]) {
                Ok(exp) => exp,
                Err(e) => return Some(Err(e)),
            },
        };

        let base_f = base.to_f64();
        let exp_f = exponent.to_f64();
        let result_f = base_f.powf(exp_f);

        // Check for invalid results
        if !result_f.is_finite() {
            return Some(Ok(FhirPathValue::Empty));
        }

        // Determine return type based on inputs and result
        match (&base, &exponent) {
            (NumericInput::Integer(_b), NumericInput::Integer(e)) => {
                // Integer to integer power
                if *e >= 0
                    && result_f == result_f.trunc()
                    && result_f >= i64::MIN as f64
                    && result_f <= i64::MAX as f64
                {
                    // Result fits in integer range and is a whole number
                    Some(Ok(FhirPathValue::Integer(result_f as i64)))
                } else {
                    // Result needs decimal representation
                    Some(Ok(FhirPathValue::Decimal(
                        Decimal::try_from(result_f).unwrap_or_default(),
                    )))
                }
            }
            _ => {
                // Any decimal involvement results in decimal
                Some(Ok(FhirPathValue::Decimal(
                    Decimal::try_from(result_f).unwrap_or_default(),
                )))
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
