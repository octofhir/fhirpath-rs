//! Simplified first function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified first function: returns the first item in a collection
pub struct SimpleFirstFunction;

impl SimpleFirstFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleFirstFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleFirstFunction {
    fn name(&self) -> &'static str {
        "first"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "first",
                parameters: vec![],
                return_type: ValueType::Any,
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
                function_name: "first".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(collection.first().unwrap().clone())
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(context.input.clone()),
        }
    }
}
