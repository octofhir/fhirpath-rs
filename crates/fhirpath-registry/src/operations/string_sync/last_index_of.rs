//! Simplified lastIndexOf function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified lastIndexOf function: finds the last index of a substring
pub struct SimpleLastIndexOfFunction;

impl SimpleLastIndexOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLastIndexOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleLastIndexOfFunction {
    fn name(&self) -> &'static str {
        "lastIndexOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "lastIndexOf",
            parameters: vec![ParameterType::String],
            return_type: ValueType::Integer,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "lastIndexOf".to_string(),
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
                if let Some(byte_index) = s.rfind(substring) {
                    // Convert byte index to character index for Unicode support
                    let char_index = s[..byte_index].chars().count();
                    Ok(FhirPathValue::Integer(char_index as i64))
                } else {
                    Ok(FhirPathValue::Integer(-1))
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    // FHIRPath spec: signal error for multiple items in collection
                    return Err(FhirPathError::EvaluationError {
                        message: "lastIndexOf() can only be applied to single values, not collections with multiple items".into(),
                        expression: None,
                        location: None,
                    });
                } else if items.len() == 1 {
                    // Single item collection - unwrap and process
                    match items.iter().next().unwrap() {
                        FhirPathValue::String(s) => {
                            if let Some(byte_index) = s.rfind(substring) {
                                // Convert byte index to character index for Unicode support
                                let char_index = s[..byte_index].chars().count();
                                Ok(FhirPathValue::Integer(char_index as i64))
                            } else {
                                Ok(FhirPathValue::Integer(-1))
                            }
                        }
                        _ => {
                            // Non-string item returns empty
                            Ok(FhirPathValue::Empty)
                        }
                    }
                } else {
                    // Empty collection
                    Ok(FhirPathValue::Empty)
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