//! Simplified truncate function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Simplified truncate function: truncates a numeric value to an integer by removing the fractional part
pub struct SimpleTruncateFunction;

impl SimpleTruncateFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleTruncateFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleTruncateFunction {
    fn name(&self) -> &'static str {
        "truncate"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "truncate",
                parameters: vec![],
                return_type: ValueType::Integer,
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
                function_name: "truncate".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => Ok(FhirPathValue::Integer(*n)),
            FhirPathValue::Decimal(n) => {
                let result = n.to_f64().unwrap_or(0.0).trunc() as i64;
                Ok(FhirPathValue::Integer(result))
            }
            FhirPathValue::Quantity(q) => {
                let result = q.value.to_f64().unwrap_or(0.0).trunc();
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
                        message: "truncate() can only be applied to single numeric values"
                            .to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "truncate() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}
