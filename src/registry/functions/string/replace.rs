//! replace() function - string replacement

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// replace() function - string replacement
pub struct ReplaceFunction;

impl FhirPathFunction for ReplaceFunction {
    fn name(&self) -> &str {
        "replace"
    }
    fn human_friendly_name(&self) -> &str {
        "Replace"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replace",
                vec![
                    ParameterInfo::required("pattern", TypeInfo::String),
                    ParameterInfo::required("substitution", TypeInfo::String),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // replace() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match (&context.input, &args[0], &args[1]) {
            (
                FhirPathValue::String(s),
                FhirPathValue::String(pattern),
                FhirPathValue::String(substitution),
            ) => {
                // Handle empty pattern case: 'abc'.replace('', 'x') should return 'xaxbxcx'
                if pattern.is_empty() {
                    let mut result = String::new();
                    result.push_str(substitution);
                    for ch in s.chars() {
                        result.push(ch);
                        result.push_str(substitution);
                    }
                    Ok(FhirPathValue::String(result))
                } else {
                    Ok(FhirPathValue::String(s.replace(pattern, substitution)))
                }
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, _, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _, _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, FhirPathValue::Collection(items), _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, _, FhirPathValue::Collection(items)) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
