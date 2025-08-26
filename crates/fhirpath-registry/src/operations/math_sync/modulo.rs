//! Simplified modulo function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified modulo function: returns the remainder of division
pub struct SimpleModuloFunction;

impl SimpleModuloFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleModuloFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleModuloFunction {
    fn name(&self) -> &'static str {
        "mod"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "mod",
                parameters: vec![ParameterType::Numeric],
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
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "mod".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let right = &args[0];
        let left = &context.input;

        // Check for modulo by zero
        let is_zero = match right {
            FhirPathValue::Integer(r) => *r == 0,
            FhirPathValue::Decimal(r) => *r == rust_decimal::Decimal::ZERO,
            _ => false,
        };

        if is_zero {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "Modulo by zero is not allowed".to_string(),
            });
        }

        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                Ok(FhirPathValue::Integer(l % r))
            }
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                let left_decimal = rust_decimal::Decimal::from(*l);
                Ok(FhirPathValue::Decimal(left_decimal % r))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(*r);
                Ok(FhirPathValue::Decimal(l % right_decimal))
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                Ok(FhirPathValue::Decimal(l % r))
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "Modulo can only be performed on numeric values".to_string(),
            }),
        }
    }
}
