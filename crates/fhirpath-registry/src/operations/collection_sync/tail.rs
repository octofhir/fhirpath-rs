//! Simplified tail function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified tail function: returns all items except the first
pub struct SimpleTailFunction;

impl SimpleTailFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleTailFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleTailFunction {
    fn name(&self) -> &'static str {
        "tail"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "tail",
                parameters: vec![],
                return_type: ValueType::Collection,
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
                function_name: "tail".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.len() <= 1 {
                    Ok(FhirPathValue::Collection(vec![]))
                } else {
                    let tail: Vec<FhirPathValue> = collection.iter().skip(1).cloned().collect();
                    Ok(FhirPathValue::Collection(tail))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![])),
            _ => {
                // Single item - tail is empty
                Ok(FhirPathValue::Collection(vec![]))
            }
        }
    }
}
