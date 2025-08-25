//! Simplified take function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified take function: takes the first n items
pub struct SimpleTakeFunction;

impl SimpleTakeFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleTakeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleTakeFunction {
    fn name(&self) -> &'static str {
        "take"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "take",
            parameters: vec![ParameterType::Integer],
            return_type: ValueType::Collection,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "take".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let count = match &args[0] {
            FhirPathValue::Integer(n) => *n,
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "take() count argument must be an integer".to_string(),
                });
            }
        };

        if count < 0 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "take() count cannot be negative".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                let take_count = count as usize;
                let taken: Vec<FhirPathValue> = collection.iter().take(take_count).cloned().collect();
                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(taken)
                ))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![])
            )),
            _ => {
                // Single item
                if count == 0 {
                    Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![])
                    ))
                } else {
                    Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![context.input.clone()])
                    ))
                }
            }
        }
    }
}