//! Simplified split function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified split function: splits a string into a collection by separator
pub struct SimpleSplitFunction;

impl SimpleSplitFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSplitFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSplitFunction {
    fn name(&self) -> &'static str {
        "split"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "split",
            parameters: vec![ParameterType::String],
            return_type: ValueType::Collection,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "split".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get separator parameter
        let separator = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "split() separator argument must be a string".to_string()
                });
            }
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let parts: Vec<FhirPathValue> = if separator.is_empty() {
                    // Empty separator means split into individual characters
                    s.chars()
                        .map(|c| FhirPathValue::String(c.to_string().into()))
                        .collect()
                } else {
                    // Split by the separator
                    s.split(separator)
                        .map(|s| FhirPathValue::String(s.to_string().into()))
                        .collect()
                };
                Ok(FhirPathValue::Collection(parts.into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
            _ => {
                // Try to convert to string first
                if let Some(string_val) = context.input.to_string_value() {
                    let parts: Vec<FhirPathValue> = if separator.is_empty() {
                        string_val.chars()
                            .map(|c| FhirPathValue::String(c.to_string().into()))
                            .collect()
                    } else {
                        string_val.split(separator)
                            .map(|s| FhirPathValue::String(s.to_string().into()))
                            .collect()
                    };
                    Ok(FhirPathValue::Collection(parts.into()))
                } else {
                    Err(FhirPathError::TypeError {
                        message: "split() can only be called on string values".to_string()
                    })
                }
            }
        }
    }
}