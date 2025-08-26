//! Simplified starts_with function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified starts_with function: returns true if the input string starts with the given prefix
pub struct SimpleStartsWithFunction;

impl SimpleStartsWithFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleStartsWithFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleStartsWithFunction {
    fn name(&self) -> &'static str {
        "startsWith"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "startsWith",
                parameters: vec![ParameterType::String],
                return_type: ValueType::Boolean,
                variadic: false,
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
                function_name: "startsWith".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get prefix parameter - try to convert to string if not already one
        let prefix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Integer(i) => {
                // Convert integer to string for comparison
                let temp_string = i.to_string();
                return match &context.input {
                    FhirPathValue::String(s) => {
                        let result = s.starts_with(&temp_string);
                        Ok(FhirPathValue::Boolean(result))
                    }
                    FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                    _ => Ok(FhirPathValue::Empty), // Return empty for invalid input type per FHIRPath spec
                };
            }
            _ => {
                // Return empty collection for invalid prefix type per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let result = s.starts_with(prefix);
                Ok(FhirPathValue::Boolean(result))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty), // Return empty for invalid input type per FHIRPath spec
        }
    }
}
