//! power() function - exponentiation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use rust_decimal::prelude::*;

/// power() function - exponentiation
pub struct PowerFunction;

#[async_trait]
impl AsyncFhirPathFunction for PowerFunction {
    fn name(&self) -> &str {
        "power"
    }
    fn human_friendly_name(&self) -> &str {
        "Power"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "power",
                vec![ParameterInfo::required("exponent", TypeInfo::Any)],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // power() is a pure mathematical function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let exponent = match &args[0] {
            FhirPathValue::Integer(i) => *i as f64,
            FhirPathValue::Decimal(d) => d.to_f64().unwrap_or(0.0),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Number".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        match &context.input {
            FhirPathValue::Integer(i) => {
                let result = (*i as f64).powf(exponent);
                if exponent.fract() == 0.0 && exponent >= 0.0 {
                    // Integer result for integer exponents
                    Ok(FhirPathValue::Integer(result as i64))
                } else {
                    Ok(FhirPathValue::Decimal(
                        Decimal::from_f64(result).unwrap_or_default(),
                    ))
                }
            }
            FhirPathValue::Decimal(d) => {
                let result = d.to_f64().unwrap_or(0.0).powf(exponent);
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
