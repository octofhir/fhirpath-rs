//! matchesFull() function - full regex match

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use regex::Regex;

/// matchesFull() function - full regex match
pub struct MatchesFullFunction;

impl FhirPathFunction for MatchesFullFunction {
    fn name(&self) -> &str {
        "matchesFull"
    }
    fn human_friendly_name(&self) -> &str {
        "Matches Full"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "matchesFull",
                vec![ParameterInfo::required("pattern", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // matchesFull() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern)) => {
                // Add anchors if not present
                let full_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
                    pattern.clone()
                } else if pattern.starts_with('^') {
                    format!("{}$", pattern)
                } else if pattern.ends_with('$') {
                    format!("^{}", pattern)
                } else {
                    format!("^{}$", pattern)
                };

                match Regex::new(&full_pattern) {
                    Ok(re) => Ok(FhirPathValue::Boolean(re.is_match(s))),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
                    }),
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}