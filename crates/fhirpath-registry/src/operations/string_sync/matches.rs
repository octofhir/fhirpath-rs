//! Simplified matches function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use regex::Regex;

/// Simplified matches function: returns true if the input string matches the given regular expression
pub struct SimpleMatchesFunction;

impl SimpleMatchesFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleMatchesFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleMatchesFunction {
    fn name(&self) -> &'static str {
        "matches"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "matches",
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
                function_name: "matches".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get regex pattern parameter
        let pattern: &str = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                // Return empty collection for invalid pattern type per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        // If pattern is empty string, return empty per spec
        if pattern.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Compile regex with single-line mode (per FHIRPath spec - dot matches newlines)
        let pattern_with_flags = if pattern.contains("(?") && pattern.contains('s') {
            // Pattern already has single-line flag set
            pattern.to_string()
        } else {
            // Add single-line flag to enable . to match newlines
            format!("(?s){pattern}")
        };

        let regex = Regex::new(&pattern_with_flags).map_err(|e| {
            FhirPathError::evaluation_error(format!("Invalid regex pattern '{pattern}': {e}"))
        })?;

        match &context.input {
            FhirPathValue::String(s) => {
                let matches = regex.is_match(s.as_ref());
                Ok(FhirPathValue::Boolean(matches))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "matches() can only be called on string values".to_string(),
            }),
        }
    }
}
