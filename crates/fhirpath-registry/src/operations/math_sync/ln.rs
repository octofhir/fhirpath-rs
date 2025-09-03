//! Simplified ln function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// Simplified ln function: returns the natural logarithm of the value
pub struct SimpleLnFunction;

impl SimpleLnFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLnFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleLnFunction {
    fn name(&self) -> &'static str {
        "ln"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "ln",
                parameters: vec![],
                return_type: ValueType::Any,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
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
                function_name: "ln".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => {
                if *n <= 0 {
                    Err(FhirPathError::evaluation_error(
                        "ln() can only be applied to positive numbers",
                    ))
                } else {
                    let result = (*n as f64).ln();
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Decimal(n) => {
                if *n <= rust_decimal::Decimal::ZERO {
                    Err(FhirPathError::evaluation_error(
                        "ln() can only be applied to positive numbers",
                    ))
                } else {
                    let result = n.to_f64().unwrap_or(0.0).ln();
                    Ok(FhirPathValue::Decimal(
                        rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Quantity { value, unit, ucum_expr } => {
                if *value <= rust_decimal::Decimal::ZERO {
                    Err(FhirPathError::evaluation_error(
                        "ln() can only be applied to positive numbers",
                    ))
                } else {
                    let result = value.to_f64().unwrap_or(0.0).ln();
                    Ok(FhirPathValue::Quantity { 
                        value: rust_decimal::Decimal::try_from(result).unwrap_or_default(),
                        unit: unit.clone(), 
                        ucum_expr: ucum_expr.clone(),
                    })
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
                        message: "ln() can only be applied to single numeric values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "ln() can only be applied to numeric values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}
