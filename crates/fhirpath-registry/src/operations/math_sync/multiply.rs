//! Simplified multiply function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

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
            (FhirPathValue::Quantity(l), FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                let result = l.value * right_decimal;
                Ok(FhirPathValue::quantity(result, l.unit.clone()))
            }
            (FhirPathValue::Quantity(l), FhirPathValue::Decimal(r)) => {
                let result = l.value * r;
                Ok(FhirPathValue::quantity(result, l.unit.clone()))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Quantity(r)) => {
                let left_decimal = rust_decimal::Decimal::from(*l);
                let result = left_decimal * r.value;
                Ok(FhirPathValue::quantity(result, r.unit.clone()))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Quantity(r)) => {
                let result = l * r.value;
                Ok(FhirPathValue::quantity(result, r.unit.clone()))
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Multiplication can only be performed on numeric values".to_string(),
            }),
        }
    }
}
