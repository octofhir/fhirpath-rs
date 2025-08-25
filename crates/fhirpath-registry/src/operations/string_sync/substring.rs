//! Simplified substring function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified substring function: extracts a substring from a string
pub struct SimpleSubstringFunction;

impl SimpleSubstringFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSubstringFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSubstringFunction {
    fn name(&self) -> &'static str {
        "substring"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "substring",
            parameters: vec![ParameterType::Integer, ParameterType::Integer],
            return_type: ValueType::String,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments - requires 1 or 2 parameters (start, optional length)
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "substring".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get start index parameter
        let start = match &args[0] {
            FhirPathValue::Integer(n) => *n as usize,
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "substring() start parameter must be an integer".to_string()
                });
            }
        };

        // Get optional length parameter
        let length = if args.len() == 2 {
            match &args[1] {
                FhirPathValue::Integer(n) => Some(*n as usize),
                _ => {
                    return Err(FhirPathError::TypeError {
                        message: "substring() length parameter must be an integer".to_string()
                    });
                }
            }
        } else {
            None
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                if start >= chars.len() {
                    return Ok(FhirPathValue::String("".into()));
                }
                
                let end = if let Some(len) = length {
                    std::cmp::min(start + len, chars.len())
                } else {
                    chars.len()
                };
                
                let result: String = chars[start..end].iter().collect();
                Ok(FhirPathValue::String(result.into()))
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::String(s) => {
                            let chars: Vec<char> = s.chars().collect();
                            if start >= chars.len() {
                                results.push(FhirPathValue::String("".into()));
                            } else {
                                let end = if let Some(len) = length {
                                    std::cmp::min(start + len, chars.len())
                                } else {
                                    chars.len()
                                };
                                
                                let result: String = chars[start..end].iter().collect();
                                results.push(FhirPathValue::String(result.into()));
                            }
                        }
                        _ => {
                            // For non-string values, skip them (per FHIRPath spec for string functions)
                            continue;
                        }
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // For non-string values, return empty (per FHIRPath spec for string functions)
                Ok(FhirPathValue::Empty)
            }
        }
    }
}