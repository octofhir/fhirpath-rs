//! Simplified log function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Simplified log function: returns the logarithm of the value with the specified base
pub struct SimpleLogFunction;

impl SimpleLogFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLogFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleLogFunction {
    fn name(&self) -> &'static str {
        "log"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "log",
            parameters: vec![ParameterType::Numeric], // Base parameter
            return_type: ValueType::Any,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "log".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let base = match &args[0] {
            FhirPathValue::Integer(b) => *b as f64,
            FhirPathValue::Decimal(b) => b.to_f64().unwrap_or(0.0),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {
                // Return empty for invalid argument types per FHIRPath specification
                return Ok(FhirPathValue::Empty);
            }
        };

        if base <= 0.0 || base == 1.0 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "log() base must be positive and not equal to 1".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => {
                if *n <= 0 {
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "log() can only be applied to positive numbers".to_string(),
                    })
                } else {
                    let result = (*n as f64).log(base);
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::try_from(result).unwrap_or_default()))
                }
            }
            FhirPathValue::Decimal(n) => {
                if *n <= rust_decimal::Decimal::ZERO {
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "log() can only be applied to positive numbers".to_string(),
                    })
                } else {
                    let result = n.to_f64().unwrap_or(0.0).log(base);
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::try_from(result).unwrap_or_default()))
                }
            }
            FhirPathValue::Quantity(q) => {
                if q.value <= rust_decimal::Decimal::ZERO {
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "log() can only be applied to positive numbers".to_string(),
                    })
                } else {
                    let result = q.value.to_f64().unwrap_or(0.0).log(base);
                    Ok(FhirPathValue::quantity(rust_decimal::Decimal::try_from(result).unwrap_or_default(), q.unit.clone()))
                }
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
                        message: "log() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "log() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}