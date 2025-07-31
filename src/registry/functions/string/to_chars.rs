//! toChars() function - converts string to array of single characters

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;

/// toChars() function - converts string to array of single characters
pub struct ToCharsFunction;

impl FhirPathFunction for ToCharsFunction {
    fn name(&self) -> &str {
        "toChars"
    }
    fn human_friendly_name(&self) -> &str {
        "To Chars"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toChars", vec![], TypeInfo::collection(TypeInfo::String))
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // toChars() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<FhirPathValue> = s
                    .chars()
                    .map(|c| FhirPathValue::String(c.to_string()))
                    .collect();
                Ok(FhirPathValue::collection(chars))
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
