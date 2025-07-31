//! matches() function - regex match

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use regex::RegexBuilder;

/// matches() function - regex match
pub struct MatchesFunction;

impl FhirPathFunction for MatchesFunction {
    fn name(&self) -> &str {
        "matches"
    }
    fn human_friendly_name(&self) -> &str {
        "Matches"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "matches",
                vec![ParameterInfo::required("pattern", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // matches() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern)) => {
                // Use RegexBuilder with single-line mode (dot matches newlines)
                match RegexBuilder::new(pattern)
                    .dot_matches_new_line(true)
                    .build()
                {
                    Ok(re) => Ok(FhirPathValue::Boolean(re.is_match(s))),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
                    }),
                }
            }
            (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}