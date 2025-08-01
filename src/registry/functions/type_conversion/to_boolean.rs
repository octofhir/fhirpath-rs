//! toBoolean() function - converts value to boolean

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use rust_decimal::prelude::*;

/// toBoolean() function - converts value to boolean
pub struct ToBooleanFunction;

impl FhirPathFunction for ToBooleanFunction {
    fn name(&self) -> &str {
        "toBoolean"
    }
    fn human_friendly_name(&self) -> &str {
        "To Boolean"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toBoolean", vec![], TypeInfo::Boolean)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toBoolean() is a pure type conversion function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)]))
            }
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => Ok(FhirPathValue::collection(
                        vec![FhirPathValue::Boolean(true)],
                    )),
                    "false" | "f" | "no" | "n" | "0" | "0.0" => Ok(FhirPathValue::collection(
                        vec![FhirPathValue::Boolean(false)],
                    )),
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Integer(i) => match *i {
                1 => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )])),
                0 => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    false,
                )])),
                _ => Ok(FhirPathValue::Empty),
            },
            FhirPathValue::Decimal(d) => {
                if *d == Decimal::ONE {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        true,
                    )]))
                } else if *d == Decimal::ZERO {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        false,
                    )]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
