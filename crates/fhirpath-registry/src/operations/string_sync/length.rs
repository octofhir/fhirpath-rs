//! Simplified length function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified length function: returns the length of a string
pub struct SimpleLengthFunction;

impl SimpleLengthFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLengthFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleLengthFunction {
    fn name(&self) -> &'static str {
        "length"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "length",
            parameters: vec![],
            return_type: ValueType::Integer,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "length".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::String(s) => {
                let length = s.chars().count() as i64;
                Ok(FhirPathValue::Integer(length))
            }
            FhirPathValue::Collection(items) => {
                // For collections, return collection length
                Ok(FhirPathValue::Integer(items.len() as i64))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // Try to convert to string first
                if let Some(string_val) = context.input.to_string_value() {
                    let length = string_val.chars().count() as i64;
                    Ok(FhirPathValue::Integer(length))
                } else {
                    Err(FhirPathError::TypeError {
                        message: "length() can only be called on string values or collections".to_string()
                    })
                }
            }
        }
    }
}