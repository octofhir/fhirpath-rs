//! Simplified multiply function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified multiply function: multiplies two numeric values
pub struct SimpleMultiplyFunction;

impl SimpleMultiplyFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleMultiplyFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleMultiplyFunction {
    fn name(&self) -> &'static str {
        "*"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "*",
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
                function_name: "*".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let right = &args[0];
        let left = &context.input;

        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                Ok(FhirPathValue::Integer(l * r))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                let left_decimal = rust_decimal::Decimal::from(*l);
                Ok(FhirPathValue::Decimal(left_decimal * r))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                Ok(FhirPathValue::Decimal(l * right_decimal))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l * r))
            }
            (FhirPathValue::Quantity { value: l_value, unit: l_unit, ucum_expr: l_ucum }, FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                let result = l_value * right_decimal;
                Ok(FhirPathValue::Quantity { value: result, unit: l_unit.clone(), ucum_expr: l_ucum.clone() })
            }
            (FhirPathValue::Quantity { value: l_value, unit: l_unit, ucum_expr: l_ucum }, FhirPathValue::Decimal(r)) => {
                let result = l_value * r;
                Ok(FhirPathValue::Quantity { value: result, unit: l_unit.clone(), ucum_expr: l_ucum.clone() })
            }
            (FhirPathValue::Integer(l), FhirPathValue::Quantity { value: r_value, unit: r_unit, ucum_expr: r_ucum }) => {
                let left_decimal = rust_decimal::Decimal::from(*l);
                let result = left_decimal * r_value;
                Ok(FhirPathValue::Quantity { value: result, unit: r_unit.clone(), ucum_expr: r_ucum.clone() })
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Quantity { value: r_value, unit: r_unit, ucum_expr: r_ucum }) => {
                let result = l * r_value;
                Ok(FhirPathValue::Quantity { value: result, unit: r_unit.clone(), ucum_expr: r_ucum.clone() })
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Multiplication can only be performed on numeric values".to_string(),
            }),
        }
    }
}
