//! Simplified empty function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified empty function: returns true if collection is empty
pub struct SimpleEmptyFunction;

impl SimpleEmptyFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleEmptyFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleEmptyFunction {
    fn name(&self) -> &'static str {
        "empty"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "empty",
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
                function_name: "empty".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                Ok(FhirPathValue::Boolean(collection.is_empty()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => Ok(FhirPathValue::Boolean(false)),
        }
    }
}