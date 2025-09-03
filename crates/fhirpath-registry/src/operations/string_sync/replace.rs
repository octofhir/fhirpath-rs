//! Simplified replace function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified replace function: replaces all instances of a pattern with a substitution
pub struct SimpleReplaceFunction;

impl SimpleReplaceFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleReplaceFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleReplaceFunction {
    fn name(&self) -> &'static str {
        "replace"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "replace",
                parameters: vec![ParameterType::String, ParameterType::String],
                return_type: ValueType::String,
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
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "replace".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // Get pattern parameter
        let pattern: &str = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                // Return empty collection for invalid pattern type per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        // Get substitution parameter
        let substitution: &str = match &args[1] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                // Return empty collection for invalid substitution type per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let result = if pattern.is_empty() {
                    // Special case: empty pattern means insert substitution between every character
                    let chars: Vec<char> = s.chars().collect();
                    if chars.is_empty() {
                        substitution.to_string()
                    } else {
                        let mut result =
                            String::with_capacity(s.len() + substitution.len() * (chars.len() + 1));
                        result.push_str(substitution);
                        for ch in chars {
                            result.push(ch);
                            result.push_str(substitution);
                        }
                        result
                    }
                } else {
                    s.replace(pattern, substitution)
                };
                Ok(FhirPathValue::String(result.into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // Try to convert to string first
                let string_val = context.input.to_string_value();
                let result = string_val.replace(pattern, substitution);
                Ok(FhirPathValue::String(result.into()))
            }
        }
    }
}
