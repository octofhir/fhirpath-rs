//! Simplified last function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified last function: returns the last item in a collection
pub struct SimpleLastFunction;

impl SimpleLastFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLastFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleLastFunction {
    fn name(&self) -> &'static str {
        "last"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "last",
                parameters: vec![],
                return_type: ValueType::Any,
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
                function_name: "last".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(collection.last().unwrap().clone())
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(context.input.clone()),
        }
    }
}
