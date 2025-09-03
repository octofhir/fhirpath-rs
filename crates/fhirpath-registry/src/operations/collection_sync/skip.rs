//! Simplified skip function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified skip function: skips the first n items
pub struct SimpleSkipFunction;

impl SimpleSkipFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSkipFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSkipFunction {
    fn name(&self) -> &'static str {
        "skip"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "skip",
                parameters: vec![ParameterType::Integer],
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
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "skip".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let count = match &args[0] {
            FhirPathValue::Integer(n) => *n,
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "skip() count argument must be an integer".to_string(),
                });
            }
        };

        if count < 0 {
            return Err(FhirPathError::evaluation_error(
                "skip() count cannot be negative",
            ));
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                let skip_count = count as usize;
                if skip_count >= collection.len() {
                    Ok(FhirPathValue::Collection(vec![]))
                } else {
                    let remaining: Vec<FhirPathValue> =
                        collection.iter().skip(skip_count).cloned().collect();
                    Ok(FhirPathValue::Collection(remaining))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![])),
            _ => {
                // Single item
                if count == 0 {
                    Ok(FhirPathValue::Collection(vec![context.input.clone()]))
                } else {
                    Ok(FhirPathValue::Collection(vec![]))
                }
            }
        }
    }
}
