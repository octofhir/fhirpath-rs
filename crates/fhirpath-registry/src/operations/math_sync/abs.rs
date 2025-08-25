//! Simplified abs function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified abs function: returns the absolute value of a numeric value
pub struct SimpleAbsFunction;

impl SimpleAbsFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleAbsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleAbsFunction {
    fn name(&self) -> &'static str {
        "abs"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "abs",
            parameters: vec![],
            return_type: ValueType::Any,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "abs".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => Ok(FhirPathValue::Integer(n.abs())),
            FhirPathValue::Decimal(n) => Ok(FhirPathValue::Decimal(n.abs())),
            FhirPathValue::Quantity(q) => {
                let abs_value = q.value.abs();
                Ok(FhirPathValue::quantity(abs_value, q.unit.clone()))
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
                        message: "abs() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "abs() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}