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

//! Round function implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Round function - rounds to nearest integer
#[derive(Debug, Clone)]
pub struct RoundFunction;

impl Default for RoundFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl RoundFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("round", OperationType::Function)
            .description("Returns the nearest integer to the input (banker's rounding), or rounded to specified precision")
            .parameter("precision", TypeConstraint::Specific(FhirPathType::Integer), true)
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer,
                FhirPathType::Decimal
            ]))
            .example("(1.5).round()")
            .example("(2.5).round()")
            .example("(1.2345).round(2)")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for RoundFunction {
    fn identifier(&self) -> &str {
        "round"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(RoundFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() > 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let precision = if args.is_empty() {
            None
        } else {
            match &args[0] {
                FhirPathValue::Integer(p) => Some(*p),
                FhirPathValue::Collection(c) if c.len() == 1 => match c.first().unwrap() {
                    FhirPathValue::Integer(p) => Some(*p),
                    _ => {
                        return Err(FhirPathError::TypeError {
                            message: "round() precision argument must be an integer".to_string(),
                        });
                    }
                },
                _ => {
                    return Err(FhirPathError::TypeError {
                        message: "round() precision argument must be an integer".to_string(),
                    });
                }
            }
        };

        match &context.input {
            FhirPathValue::Integer(n) => {
                match precision {
                    None => Ok(FhirPathValue::Integer(*n)), // Already integer
                    Some(_) => Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*n))), // Return as decimal for consistency
                }
            }
            FhirPathValue::Decimal(n) => match precision {
                None => Ok(FhirPathValue::Integer(n.round().to_i64().unwrap_or(0))),
                Some(p) => {
                    if p < 0 {
                        return Err(FhirPathError::TypeError {
                            message: "round() precision must be non-negative".to_string(),
                        });
                    }
                    let factor = rust_decimal::Decimal::from(10_i64.pow(p as u32));
                    let rounded = (n * factor).round() / factor;
                    Ok(FhirPathValue::Decimal(rounded))
                }
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = context.with_input(c.first().unwrap().clone());
                    self.evaluate(args, &item_context).await
                } else {
                    Err(FhirPathError::TypeError {
                        message: "round() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "round() can only be applied to numeric values, got {}",
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
        if args.len() > 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        let precision = if args.is_empty() {
            None
        } else {
            match &args[0] {
                FhirPathValue::Integer(p) => Some(*p),
                FhirPathValue::Collection(c) if c.len() == 1 => match c.first().unwrap() {
                    FhirPathValue::Integer(p) => Some(*p),
                    _ => {
                        return Some(Err(FhirPathError::TypeError {
                            message: "round() precision argument must be an integer".to_string(),
                        }));
                    }
                },
                _ => {
                    return Some(Err(FhirPathError::TypeError {
                        message: "round() precision argument must be an integer".to_string(),
                    }));
                }
            }
        };

        match &context.input {
            FhirPathValue::Integer(n) => match precision {
                None => Some(Ok(FhirPathValue::Integer(*n))),
                Some(_) => Some(Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*n)))),
            },
            FhirPathValue::Decimal(n) => match precision {
                None => Some(Ok(FhirPathValue::Integer(n.round().to_i64().unwrap_or(0)))),
                Some(p) => {
                    if p < 0 {
                        return Some(Err(FhirPathError::TypeError {
                            message: "round() precision must be non-negative".to_string(),
                        }));
                    }
                    let factor = rust_decimal::Decimal::from(10_i64.pow(p as u32));
                    let rounded = (n * factor).round() / factor;
                    Some(Ok(FhirPathValue::Decimal(rounded)))
                }
            },
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if c.len() == 1 {
                    let item_context = context.with_input(c.first().unwrap().clone());
                    self.try_evaluate_sync(args, &item_context)
                } else {
                    Some(Err(FhirPathError::TypeError {
                        message: "round() can only be applied to single numeric values".to_string(),
                    }))
                }
            }
            _ => Some(Err(FhirPathError::TypeError {
                message: format!(
                    "round() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            })),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
