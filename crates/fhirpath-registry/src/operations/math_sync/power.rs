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
                // Return empty collection for invalid exponent type per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        match &context.input {
            FhirPathValue::Integer(n) => {
                let base = *n as f64;
                
                // Check for invalid power operations per FHIRPath spec
                if base < 0.0 && exponent.fract() != 0.0 {
                    // Negative base with fractional exponent is undefined - return empty
                    return Ok(FhirPathValue::Empty);
                }
                
                let result = base.powf(exponent);
                
                // Check for NaN or infinite results
                if result.is_nan() || result.is_infinite() {
                    return Ok(FhirPathValue::Empty);
                }
                
                if result.fract() == 0.0 && result <= i64::MAX as f64 && result >= i64::MIN as f64 {
                    Ok(FhirPathValue::Integer(result as i64))
                } else {
                    match rust_decimal::Decimal::try_from(result) {
                        Ok(decimal) => Ok(FhirPathValue::Decimal(decimal)),
                        Err(_) => Ok(FhirPathValue::Empty) // Return empty for conversion errors
                    }
                }
            }
            FhirPathValue::Decimal(n) => {
                let base = n.to_f64().unwrap_or(0.0);
                
                // Check for invalid power operations per FHIRPath spec
                if base < 0.0 && exponent.fract() != 0.0 {
                    // Negative base with fractional exponent is undefined - return empty
                    return Ok(FhirPathValue::Empty);
                }
                
                let result = base.powf(exponent);
                
                // Check for NaN or infinite results
                if result.is_nan() || result.is_infinite() {
                    return Ok(FhirPathValue::Empty);
                }
                
                match rust_decimal::Decimal::try_from(result) {
                    Ok(decimal) => Ok(FhirPathValue::Decimal(decimal)),
                    Err(_) => Ok(FhirPathValue::Empty) // Return empty for conversion errors
                }
            }
            FhirPathValue::Quantity(q) => {
                let base = q.value.to_f64().unwrap_or(0.0);
                
                // Check for invalid power operations per FHIRPath spec
                if base < 0.0 && exponent.fract() != 0.0 {
                    // Negative base with fractional exponent is undefined - return empty
                    return Ok(FhirPathValue::Empty);
                }
                
                let result = base.powf(exponent);
                
                // Check for NaN or infinite results
                if result.is_nan() || result.is_infinite() {
                    return Ok(FhirPathValue::Empty);
                }
                
                match rust_decimal::Decimal::try_from(result) {
                    Ok(decimal) => Ok(FhirPathValue::quantity(decimal, q.unit.clone())),
                    Err(_) => Ok(FhirPathValue::Empty) // Return empty for conversion errors
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
                    // Multiple items - return empty collection per FHIRPath spec
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => Ok(FhirPathValue::Empty), // Return empty for invalid input type per FHIRPath spec
        }
    }
}