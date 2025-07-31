//! convertsToInteger() function - checks if value can be converted to integer

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// convertsToInteger() function - checks if value can be converted to integer
pub struct ConvertsToIntegerFunction;

impl FhirPathFunction for ConvertsToIntegerFunction {
    fn name(&self) -> &str {
        "convertsToInteger"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Integer"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToInteger", vec![], TypeInfo::Boolean)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // convertsToInteger() is a pure type conversion function
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
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => !s.contains('.') && s.trim().parse::<i64>().is_ok(),
            FhirPathValue::Boolean(_) => true,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}