//! convertsToBoolean() function - checks if value can be converted to boolean

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// convertsToBoolean() function - checks if value can be converted to boolean
pub struct ConvertsToBooleanFunction;

impl FhirPathFunction for ConvertsToBooleanFunction {
    fn name(&self) -> &str {
        "convertsToBoolean"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Boolean"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToBoolean", vec![], TypeInfo::Boolean)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // convertsToBoolean() is a pure type conversion function
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

        let can_convert = match input_item {
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                matches!(
                    lower.as_str(),
                    "true"
                        | "t"
                        | "yes"
                        | "y"
                        | "1"
                        | "1.0"
                        | "false"
                        | "f"
                        | "no"
                        | "n"
                        | "0"
                        | "0.0"
                )
            }
            FhirPathValue::Integer(i) => *i == 0 || *i == 1,
            FhirPathValue::Decimal(d) => *d == Decimal::ZERO || *d == Decimal::ONE,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}