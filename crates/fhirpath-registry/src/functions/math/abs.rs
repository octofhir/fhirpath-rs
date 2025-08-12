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

//! abs() function - absolute value

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// abs() function - absolute value
pub struct AbsFunction;

#[async_trait]
impl AsyncFhirPathFunction for AbsFunction {
    fn name(&self) -> &str {
        "abs"
    }
    fn human_friendly_name(&self) -> &str {
        "Absolute Value"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("abs", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // abs() is a pure mathematical function
    }

    fn documentation(&self) -> &str {
        "Returns the absolute value of the input. When taking the absolute value of a quantity, the unit is unchanged."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle single-element collections (common in method calls like (-5).abs())
        let input_value = match &context.input {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
            other => other,
        };

        match input_value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(i.abs())),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(d.abs())),
            FhirPathValue::Quantity(q) => {
                if q.value < rust_decimal::Decimal::ZERO {
                    Ok(FhirPathValue::Quantity(
                        q.multiply_scalar(rust_decimal::Decimal::from(-1)).into(),
                    ))
                } else {
                    Ok(FhirPathValue::Quantity(q.clone()))
                }
            }
            FhirPathValue::Collection(collection) => {
                let mut results = Vec::new();
                for item in collection.iter() {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i.abs())),
                        FhirPathValue::Decimal(d) => results.push(FhirPathValue::Decimal(d.abs())),
                        FhirPathValue::Quantity(q) => {
                            if q.value < rust_decimal::Decimal::ZERO {
                                results.push(FhirPathValue::Quantity(
                                    q.multiply_scalar(rust_decimal::Decimal::from(-1)).into(),
                                ));
                            } else {
                                results.push(FhirPathValue::Quantity(q.clone()));
                            }
                        }
                        _ => {
                            return Err(FunctionError::InvalidArgumentType {
                                name: self.name().to_string(),
                                index: 0,
                                expected: "Number or Quantity".to_string(),
                                actual: format!("{item:?}"),
                            });
                        }
                    }
                }
                Ok(FhirPathValue::Collection(
                    fhirpath_model::Collection::from_vec(results),
                ))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Number or Quantity".to_string(),
                actual: format!("{input_value:?}"),
            }),
        }
    }
}
