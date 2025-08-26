//! Simplified anyFalse function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified anyFalse function: returns true if any item in collection is false
pub struct SimpleAnyFalseFunction;

impl SimpleAnyFalseFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleAnyFalseFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleAnyFalseFunction {
    fn name(&self) -> &'static str {
        "anyFalse"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "anyFalse",
                parameters: vec![],
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
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "anyFalse".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    return Ok(FhirPathValue::Boolean(false));
                }

                for item in collection.iter() {
                    match item {
                        FhirPathValue::Boolean(false) => return Ok(FhirPathValue::Boolean(true)),
                        FhirPathValue::Boolean(true) => continue,
                        FhirPathValue::Empty => continue, // Empty values are ignored
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "anyFalse() can only be applied to boolean collections"
                                    .to_string(),
                            });
                        }
                    }
                }

                Ok(FhirPathValue::Boolean(false))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            _ => Err(FhirPathError::TypeError {
                message: "anyFalse() can only be applied to boolean values".to_string(),
            }),
        }
    }
}
