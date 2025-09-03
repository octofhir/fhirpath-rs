//! Simplified divide function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified divide function: divides two numeric values
pub struct SimpleDivideFunction;

impl SimpleDivideFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleDivideFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleDivideFunction {
    fn name(&self) -> &'static str {
        "/"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "/",
                parameters: vec![ParameterType::Numeric],
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
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "/".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let right = &args[0];
        let left = &context.input;

        // Check for division by zero
        let is_zero = match right {
            FhirPathValue::Integer(r) => *r == 0,
            FhirPathValue::Decimal(r) => *r == rust_decimal::Decimal::ZERO,
            FhirPathValue::Quantity { value: r_value, unit: r_unit, ucum_expr: r_ucum } => *r_value == rust_decimal::Decimal::ZERO,
            _ => false,
        };

        if is_zero {
            return Err(FhirPathError::evaluation_error(
                "Division by zero is not allowed",
            ));
        }

        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                // Integer division in FHIRPath typically returns decimal for precision
                let left_decimal = rust_decimal::Decimal::from(*l);
                let right_decimal = rust_decimal::Decimal::from(*r);
                Ok(FhirPathValue::Decimal(left_decimal / right_decimal))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                let left_decimal = rust_decimal::Decimal::from(*l);
                Ok(FhirPathValue::Decimal(left_decimal / r))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                Ok(FhirPathValue::Decimal(l / right_decimal))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l / r))
            }
            (FhirPathValue::Quantity { value: l_value, unit: l_unit, ucum_expr: l_ucum }, FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                let result = l_value / right_decimal;
                Ok(FhirPathValue::Quantity { value: result, unit: l_unit.clone(), ucum_expr: l_ucum.clone() })
            }
            (FhirPathValue::Quantity { value: l_value, unit: l_unit, ucum_expr: l_ucum }, FhirPathValue::Decimal(r)) => {
                let result = l_value / r;
                Ok(FhirPathValue::Quantity { value: result, unit: l_unit.clone(), ucum_expr: l_ucum.clone() })
            }
            (FhirPathValue::Quantity { value: l_value, unit: l_unit, ucum_expr: l_ucum }, FhirPathValue::Quantity { value: r_value, unit: r_unit, ucum_expr: r_ucum }) => {
                // For now, assume same units - full UCUM conversion would be needed for proper implementation
                if l_unit == r_unit {
                    let result = l_value / r_value;
                    // Division of same units typically results in dimensionless quantity
                    Ok(FhirPathValue::Decimal(result))
                } else {
                    Err(FhirPathError::TypeError {
                        message: format!(
                            "Cannot divide quantities with different units: {:?} and {:?}",
                            l_unit, r_unit
                        ),
                    })
                }
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Division can only be performed on numeric values".to_string(),
            }),
        }
    }
}
