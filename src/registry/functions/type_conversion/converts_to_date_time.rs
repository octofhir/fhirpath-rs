//! convertsToDateTime() function - checks if value can be converted to datetime

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;

/// convertsToDateTime() function - checks if value can be converted to datetime
pub struct ConvertsToDateTimeFunction;

impl FhirPathFunction for ConvertsToDateTimeFunction {
    fn name(&self) -> &str {
        "convertsToDateTime"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To DateTime"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToDateTime", vec![], TypeInfo::Boolean)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // convertsToDateTime() is a pure type conversion function
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
            FhirPathValue::DateTime(_) => true,
            FhirPathValue::Date(_) => true, // Date can be converted to DateTime
            FhirPathValue::String(s) => {
                // Check if string matches valid datetime formats
                let datetime_regex = regex::Regex::new(r"^\d{4}(-\d{2}(-\d{2}(T\d{2}(:\d{2}(:\d{2}(\.\d{3})?)?)?(Z|[+-]\d{2}:\d{2})?)?)?)?$").unwrap();
                datetime_regex.is_match(s)
            }
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}
