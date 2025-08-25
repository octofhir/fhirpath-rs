//! Simplified allFalse function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified allFalse function: returns true if all items in collection are false
pub struct SimpleAllFalseFunction;

impl SimpleAllFalseFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleAllFalseFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleAllFalseFunction {
    fn name(&self) -> &'static str {
        "allFalse"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "allFalse",
            parameters: vec![],
            return_type: ValueType::Boolean,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "allFalse".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    return Ok(FhirPathValue::Boolean(true));
                }

                let mut has_boolean_values = false;
                for item in collection.iter() {
                    match item {
                        FhirPathValue::Boolean(true) => return Ok(FhirPathValue::Boolean(false)),
                        FhirPathValue::Boolean(false) => {
                            has_boolean_values = true;
                            continue;
                        },
                        FhirPathValue::Empty => continue, // Empty values are ignored
                        _ => {
                            // Non-boolean values are ignored per FHIRPath specification
                            continue;
                        }
                    }
                }
                
                // If no boolean values were found, return empty
                if !has_boolean_values {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Boolean(true))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            _ => {
                // Non-boolean single values return empty per FHIRPath specification
                Ok(FhirPathValue::Empty)
            }
        }
    }
}