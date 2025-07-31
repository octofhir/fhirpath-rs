//! sqrt() function - square root

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use rust_decimal::prelude::*;

/// sqrt() function - square root
pub struct SqrtFunction;

impl FhirPathFunction for SqrtFunction {
    fn name(&self) -> &str {
        "sqrt"
    }
    fn human_friendly_name(&self) -> &str {
        "Square Root"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("sqrt", vec![], TypeInfo::Decimal));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // sqrt() is a pure mathematical function
    }

    fn documentation(&self) -> &str {
        "Returns the square root of the input number as a Decimal. If the square root cannot be represented (such as the square root of -1), the result is empty."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Integer(i) => {
                if *i < 0 {
                    return Ok(FhirPathValue::Empty);
                }
                let result = (*i as f64).sqrt();
                Ok(FhirPathValue::Decimal(
                    Decimal::from_f64(result).unwrap_or_default(),
                ))
            }
            FhirPathValue::Decimal(d) => {
                if d.is_sign_negative() {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot take square root of negative number".to_string(),
                    });
                }
                let result = d.to_f64().unwrap_or(0.0).sqrt();
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
