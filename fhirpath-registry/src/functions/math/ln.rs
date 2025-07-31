//! ln() function - natural logarithm

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// ln() function - natural logarithm
pub struct LnFunction;

impl FhirPathFunction for LnFunction {
    fn name(&self) -> &str {
        "ln"
    }
    fn human_friendly_name(&self) -> &str {
        "Natural Logarithm"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("ln", vec![], TypeInfo::Decimal));
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // ln() is a pure mathematical function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Integer(i) => {
                if *i <= 0 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot take logarithm of non-positive number".to_string(),
                    });
                }
                let result = (*i as f64).ln();
                Ok(FhirPathValue::Decimal(
                    Decimal::from_f64(result).unwrap_or_default(),
                ))
            }
            FhirPathValue::Decimal(d) => {
                if d.is_sign_negative() || d.is_zero() {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot take logarithm of non-positive number".to_string(),
                    });
                }
                let result = d.to_f64().unwrap_or(0.0).ln();
                Ok(FhirPathValue::Decimal(
                    Decimal::from_f64(result).unwrap_or_default(),
                ))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Number".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}