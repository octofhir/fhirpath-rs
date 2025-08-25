//! Simplified power function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Simplified power function: raises the value to the specified power
pub struct SimplePowerFunction;

impl SimplePowerFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimplePowerFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimplePowerFunction {
    fn name(&self) -> &'static str {
        "power"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "power",
            parameters: vec![ParameterType::Numeric], // Exponent parameter
            return_type: ValueType::Any,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "power".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let exponent = match &args[0] {
            FhirPathValue::Integer(e) => *e as f64,
            FhirPathValue::Decimal(e) => e.to_f64().unwrap_or(0.0),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "power() exponent argument must be numeric".to_string(),
                });
            }
        };

        match &context.input {
            FhirPathValue::Integer(n) => {
                let base = *n as f64;
                let result = base.powf(exponent);
                if result.fract() == 0.0 && result <= i64::MAX as f64 && result >= i64::MIN as f64 {
                    Ok(FhirPathValue::Integer(result as i64))
                } else {
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::try_from(result).unwrap_or_default()))
                }
            }
            FhirPathValue::Decimal(n) => {
                let base = n.to_f64().unwrap_or(0.0);
                let result = base.powf(exponent);
                Ok(FhirPathValue::Decimal(rust_decimal::Decimal::try_from(result).unwrap_or_default()))
            }
            FhirPathValue::Quantity(q) => {
                let base = q.value.to_f64().unwrap_or(0.0);
                let result = base.powf(exponent);
                Ok(FhirPathValue::quantity(rust_decimal::Decimal::try_from(result).unwrap_or_default(), q.unit.clone()))
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
                        message: "power() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "power() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}