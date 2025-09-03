//! Simplified upper function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified upper function: converts string to uppercase
pub struct SimpleUpperFunction;

impl SimpleUpperFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleUpperFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleUpperFunction {
    fn name(&self) -> &'static str {
        "upper"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "upper",
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
                function_name: "upper".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::String(s) => {
                let upper_str = s.to_uppercase();
                Ok(FhirPathValue::String(upper_str.into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::TypeError {
                message: "upper() can only be called on string values".to_string(),
            }),
        }
    }
}
