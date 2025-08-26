//! Simplified trim function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified trim function: removes whitespace from both ends of a string
pub struct SimpleTrimFunction;

impl SimpleTrimFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleTrimFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleTrimFunction {
    fn name(&self) -> &'static str {
        "trim"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "trim",
            parameters: vec![],
            return_type: ValueType::String,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "trim".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::String(s) => {
                let trimmed_str = s.trim();
                Ok(FhirPathValue::String(trimmed_str.to_string().into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "trim() can only be called on string values".to_string(),
            }),
        }
    }
}
