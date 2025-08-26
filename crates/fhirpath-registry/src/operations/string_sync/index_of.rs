//! Simplified index_of function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified index_of function: finds the index of a substring
pub struct SimpleIndexOfFunction;

impl SimpleIndexOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleIndexOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleIndexOfFunction {
    fn name(&self) -> &'static str {
        "indexOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "indexOf",
                parameters: vec![ParameterType::String],
                return_type: ValueType::Integer,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "indexOf".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let substring = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {
                // Return empty for invalid argument types per FHIRPath specification
                return Ok(FhirPathValue::Empty);
            }
        };

        match &context.input {
            FhirPathValue::String(s) => {
                if let Some(byte_index) = s.find(substring) {
                    // Convert byte index to character index for Unicode support
                    let char_index = s[..byte_index].chars().count();
                    Ok(FhirPathValue::Integer(char_index as i64))
                } else {
                    Ok(FhirPathValue::Integer(-1))
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::String(s) => {
                            if let Some(byte_index) = s.find(substring) {
                                // Convert byte index to character index for Unicode support
                                let char_index = s[..byte_index].chars().count();
                                results.push(FhirPathValue::Integer(char_index as i64));
                            } else {
                                results.push(FhirPathValue::Integer(-1));
                            }
                        }
                        _ => {
                            // Non-string items in collection return empty per FHIRPath spec
                            // But since we're in a collection context, we skip them
                        }
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // Non-string input returns empty per FHIRPath specification
                Ok(FhirPathValue::Empty)
            }
        }
    }
}
