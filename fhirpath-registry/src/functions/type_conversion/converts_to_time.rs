//! convertsToTime() function - checks if value can be converted to time

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// convertsToTime() function - checks if value can be converted to time
pub struct ConvertsToTimeFunction;

impl FhirPathFunction for ConvertsToTimeFunction {
    fn name(&self) -> &str {
        "convertsToTime"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Time"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToTime", vec![], TypeInfo::Boolean)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // convertsToTime() is a pure type conversion function
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
            FhirPathValue::Time(_) => true,
            FhirPathValue::String(s) => {
                // Check if string matches valid time formats: HH, HH:MM, HH:MM:SS, HH:MM:SS.mmm
                let time_regex = regex::Regex::new(r"^\d{2}(:\d{2}(:\d{2}(\.\d{3})?)?)?$").unwrap();
                time_regex.is_match(s)
            }
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}