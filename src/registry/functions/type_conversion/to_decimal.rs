//! toDecimal() function - converts value to decimal

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use rust_decimal::prelude::*;

/// toDecimal() function - converts value to decimal
pub struct ToDecimalFunction;

impl FhirPathFunction for ToDecimalFunction {
    fn name(&self) -> &str {
        "toDecimal"
    }
    fn human_friendly_name(&self) -> &str {
        "To Decimal"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toDecimal", vec![], TypeInfo::Decimal)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toDecimal() is a pure type conversion function
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
            FhirPathValue::Decimal(d) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(*d)]))
            }
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                    Decimal::from(*i),
                )]))
            }
            FhirPathValue::String(s) => match Decimal::from_str(s.trim()) {
                Ok(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(d)])),
                Err(_) => Ok(FhirPathValue::Empty),
            },
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                    if *b { Decimal::ONE } else { Decimal::ZERO },
                )]))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
