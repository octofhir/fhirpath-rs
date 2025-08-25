//! Simplified single function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified single function: returns single item or error if not exactly one
pub struct SimpleSingleFunction;

impl SimpleSingleFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSingleFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSingleFunction {
    fn name(&self) -> &'static str {
        "single"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "single",
            parameters: vec![],
            return_type: ValueType::Any,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "single".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                match collection.len() {
                    0 => Ok(FhirPathValue::Empty),
                    1 => Ok(collection.first().unwrap().clone()),
                    _ => {
                        // Per FHIRPath spec: single() on multiple items should return empty
                        // But the result needs to propagate as truly empty, not false
                        Ok(FhirPathValue::Empty)
                    }
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(context.input.clone()),
        }
    }
}