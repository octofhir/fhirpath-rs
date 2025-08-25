//! Simplified toChars function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified toChars function: converts a string into a collection of individual characters
pub struct SimpleToCharsFunction;

impl SimpleToCharsFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleToCharsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleToCharsFunction {
    fn name(&self) -> &'static str {
        "toChars"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "toChars",
            parameters: vec![],
            return_type: ValueType::Collection,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "toChars".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<FhirPathValue> = s.chars()
                    .map(|c| FhirPathValue::String(c.to_string().into()))
                    .collect();
                Ok(FhirPathValue::Collection(chars.into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
            _ => {
                // Try to convert to string first
                if let Some(string_val) = context.input.to_string_value() {
                    let chars: Vec<FhirPathValue> = string_val.chars()
                        .map(|c| FhirPathValue::String(c.to_string().into()))
                        .collect();
                    Ok(FhirPathValue::Collection(chars.into()))
                } else {
                    Err(FhirPathError::TypeError {
                        message: "toChars() can only be called on string values".to_string()
                    })
                }
            }
        }
    }
}