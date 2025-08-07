//! replaceMatches() function - regex replacement

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use regex::Regex;

/// replaceMatches() function - regex replacement
pub struct ReplaceMatchesFunction;

#[async_trait]
impl AsyncFhirPathFunction for ReplaceMatchesFunction {
    fn name(&self) -> &str {
        "replaceMatches"
    }
    fn human_friendly_name(&self) -> &str {
        "Replace Matches"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replaceMatches",
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
        true // replaceMatches() is a pure string function
    }
    async fn evaluate(
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
                // Handle empty pattern - return original string unchanged
                if pattern.is_empty() {
                    return Ok(FhirPathValue::String(s.clone()));
                }

                match Regex::new(pattern) {
                    Ok(re) => Ok(FhirPathValue::String(
                        re.replace_all(s, substitution).to_string(),
                    )),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {e}"),
                    }),
                }
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _, _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, _, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
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
