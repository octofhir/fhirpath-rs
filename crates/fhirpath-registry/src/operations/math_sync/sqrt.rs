//! Simplified sqrt function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Simplified sqrt function: returns the square root of a numeric value
pub struct SimpleSqrtFunction;

impl SimpleSqrtFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSqrtFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSqrtFunction {
    fn name(&self) -> &'static str {
        "sqrt"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "sqrt",
                parameters: vec![],
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
        // Validate arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "sqrt".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => {
                if *n < 0 {
                    Err(FhirPathError::evaluation_error("sqrt() cannot be applied to negative numbers"))
                } else {
                    let result = (*n as f64).sqrt();
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Decimal(n) => {
                if *n < rust_decimal::Decimal::ZERO {
                    Err(FhirPathError::evaluation_error("sqrt() cannot be applied to negative numbers"))
                } else {
                    let result = n.to_f64().unwrap_or(0.0).sqrt();
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Quantity(q) => {
                if q.value < rust_decimal::Decimal::ZERO {
                    Err(FhirPathError::evaluation_error("sqrt() cannot be applied to negative numbers"))
                } else {
                    let result = q.value.to_f64().unwrap_or(0.0).sqrt();
                    Ok(FhirPathValue::quantity(
                        rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                        q.unit.clone(),
                    ))
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
                        message: "sqrt() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "sqrt() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}
