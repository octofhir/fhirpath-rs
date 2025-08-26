//! Simplified round function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Simplified round function: rounds a numeric value to the specified number of decimal places
pub struct SimpleRoundFunction;

impl SimpleRoundFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleRoundFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleRoundFunction {
    fn name(&self) -> &'static str {
        "round"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "round",
                parameters: vec![ParameterType::Integer], // Optional precision parameter
                return_type: ValueType::Any,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments - round can take 0 or 1 argument
        if args.len() > 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "round".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let precision = if args.is_empty() {
            0
        } else {
            match &args[0] {
                FhirPathValue::Integer(p) => *p,
                _ => {
                    return Err(FhirPathError::TypeError {
                        message: "round() precision argument must be an integer".to_string(),
                    });
                }
            }
        };

        match &context.input {
            FhirPathValue::Integer(n) => {
                if precision <= 0 {
                    Ok(FhirPathValue::Integer(*n))
                } else {
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*n)))
                }
            }
            FhirPathValue::Decimal(n) => {
                let value = n.to_f64().unwrap_or(0.0);
                if precision <= 0 {
                    let result = value.round() as i64;
                    Ok(FhirPathValue::Integer(result))
                } else {
                    let multiplier = 10_f64.powi(precision as i32);
                    let result = (value * multiplier).round() / multiplier;
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Quantity(q) => {
                let value = q.value.to_f64().unwrap_or(0.0);
                let result = if precision <= 0 {
                    value.round()
                } else {
                    let multiplier = 10_f64.powi(precision as i32);
                    (value * multiplier).round() / multiplier
                };
                Ok(FhirPathValue::quantity(
                    rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                    q.unit.clone(),
                ))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item = c.first().unwrap();
                    let item_context = context.with_input(item.clone());
                    self.execute(args, &item_context)
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
}
