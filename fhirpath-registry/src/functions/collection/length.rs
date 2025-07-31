//! length() function implementation

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// length() function - returns the length of a string
pub struct LengthFunction;

impl FhirPathFunction for LengthFunction {
    fn name(&self) -> &str {
        "length"
    }
    fn human_friendly_name(&self) -> &str {
        "Length"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("length", vec![], TypeInfo::Integer)
        });
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // length() is a pure function - same input always produces same output
    }
    
    fn documentation(&self) -> &str {
        "Returns the length of the input string. If the input collection is empty (`{ }`), the result is empty."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
            FhirPathValue::Resource(r) => {
                // Try to extract string value from FhirResource
                match r.as_json() {
                    serde_json::Value::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
                    _ => Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 0,
                        expected: "String".to_string(),
                        actual: format!("{:?}", context.input),
                    }),
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}