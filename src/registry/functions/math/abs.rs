//! abs() function - absolute value

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use rust_decimal::prelude::*;

/// abs() function - absolute value
pub struct AbsFunction;

impl FhirPathFunction for AbsFunction {
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

    fn evaluate(
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
                        q.multiply_scalar(rust_decimal::Decimal::from(-1)),
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
                                    q.multiply_scalar(rust_decimal::Decimal::from(-1)),
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
                    crate::model::Collection::from_vec(results),
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
