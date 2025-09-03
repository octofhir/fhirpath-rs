//! Simplified lower function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified lower function: converts string to lowercase
pub struct SimpleLowerFunction;

impl SimpleLowerFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLowerFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleLowerFunction {
    fn name(&self) -> &'static str {
        "lower"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "lower",
            parameters: vec![],
            return_type: ValueType::String,
            variadic: false,
            category: FunctionCategory::Scalar,
            cardinality_requirement: CardinalityRequirement::RequiresScalar,
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
                function_name: "lower".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::String(s) => {
                let lower_str = s.to_lowercase();
                Ok(FhirPathValue::String(lower_str.into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "lower() can only be called on string values".to_string(),
            }),
        }
    }
}
