//! Simplified exists function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified exists function: returns true if collection is not empty
pub struct SimpleExistsFunction;

impl SimpleExistsFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleExistsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleExistsFunction {
    fn name(&self) -> &'static str {
        "exists"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "exists",
                parameters: vec![],
                return_type: ValueType::Boolean,
                variadic: false,
                category: FunctionCategory::Collection,
                cardinality_requirement: CardinalityRequirement::RequiresCollection,
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
                function_name: "exists".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                Ok(FhirPathValue::Boolean(!collection.is_empty()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Boolean(true)),
        }
    }
}
