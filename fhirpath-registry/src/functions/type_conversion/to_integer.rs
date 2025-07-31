//! toInteger() function - converts value to integer

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// toInteger() function - converts value to integer
pub struct ToIntegerFunction;

impl FhirPathFunction for ToIntegerFunction {
    fn name(&self) -> &str {
        "toInteger"
    }
    fn human_friendly_name(&self) -> &str {
        "To Integer"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toInteger", vec![], TypeInfo::Integer)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toInteger() is a pure type conversion function
    }
    
    fn documentation(&self) -> &str {
        "Returns the value as an Integer if it is a valid representation of an integer. If the input is not convertible to an Integer, the result is empty."
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
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)]))
            }
            FhirPathValue::String(s) => {
                // According to FHIRPath spec, strings with decimal points cannot be converted to integers
                if s.contains('.') {
                    Ok(FhirPathValue::Empty)
                } else {
                    match s.trim().parse::<i64>() {
                        Ok(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)])),
                        Err(_) => Ok(FhirPathValue::Empty),
                    }
                }
            }
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(
                    if *b { 1 } else { 0 },
                )]))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}