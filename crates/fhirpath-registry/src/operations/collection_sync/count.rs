//! Simplified count function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified count function: returns the number of items in a collection
pub struct SimpleCountFunction;

impl SimpleCountFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleCountFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleCountFunction {
    fn name(&self) -> &'static str {
        "count"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "count",
                parameters: vec![],
                return_type: ValueType::Integer,
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
                function_name: "count".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                Ok(FhirPathValue::Integer(collection.len() as i64))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        }
    }
}
