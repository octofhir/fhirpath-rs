//! Simplified subtract function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified subtract function: subtracts two numeric values
pub struct SimpleSubtractFunction;

impl SimpleSubtractFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSubtractFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSubtractFunction {
    fn name(&self) -> &'static str {
        "-"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "-",
            parameters: vec![ParameterType::Numeric],
            return_type: ValueType::Any,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "-".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let right = &args[0];
        let left = &context.input;

        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                Ok(FhirPathValue::Integer(l - r))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                let left_decimal = rust_decimal::Decimal::from(*l);
                Ok(FhirPathValue::Decimal(left_decimal - r))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                Ok(FhirPathValue::Decimal(l - right_decimal))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l - r))
            }
            (FhirPathValue::Quantity(l), FhirPathValue::Quantity(r)) => {
                // For now, assume same units - full UCUM conversion would be needed for proper implementation
                if l.unit == r.unit {
                    let result = l.value - r.value;
                    Ok(FhirPathValue::quantity(result, l.unit.clone()))
                } else {
                    Err(FhirPathError::TypeError {
                        message: format!("Cannot subtract quantities with different units: {:?} and {:?}", l.unit, r.unit),
                    })
                }
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Subtraction can only be performed on numeric values".to_string(),
            }),
        }
    }
}