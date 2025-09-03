//! Simplified matchesFull function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use regex::Regex;

/// Simplified matchesFull function: returns true if the entire input string matches the given regular expression
pub struct SimpleMatchesFullFunction;

impl SimpleMatchesFullFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleMatchesFullFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleMatchesFullFunction {
    fn name(&self) -> &'static str {
        "matchesFull"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "matchesFull",
                parameters: vec![ParameterType::String],
                return_type: ValueType::Boolean,
                variadic: false,
                category: FunctionCategory::Scalar,
                cardinality_requirement: CardinalityRequirement::RequiresScalar,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "matchesFull".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get regex pattern parameter
        let pattern: &str = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "matchesFull() pattern argument must be a string".to_string(),
                });
            }
        };

        // If pattern is empty string, return empty per spec
        if pattern.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::String(s) => {
                // Ensure the pattern matches the entire string by anchoring it
                let anchored_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
                    pattern.to_string()
                } else if pattern.starts_with('^') {
                    format!("{pattern}$")
                } else if pattern.ends_with('$') {
                    format!("^{pattern}")
                } else {
                    format!("^{pattern}$")
                };

                let regex = Regex::new(&anchored_pattern).map_err(|e| {
                    FhirPathError::evaluation_error(format!(
                        "Invalid regex pattern '{pattern}': {e}"
                    ))
                })?;

                let matches = regex.is_match(s.as_ref());
                Ok(FhirPathValue::Boolean(matches))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "matchesFull() can only be called on string values".to_string(),
            }),
        }
    }
}
