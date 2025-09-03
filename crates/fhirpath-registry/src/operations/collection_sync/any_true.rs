//! Simplified anyTrue function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified anyTrue function: returns true if any item in collection is true
pub struct SimpleAnyTrueFunction;

impl SimpleAnyTrueFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleAnyTrueFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleAnyTrueFunction {
    fn name(&self) -> &'static str {
        "anyTrue"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "anyTrue",
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
                function_name: "anyTrue".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    return Ok(FhirPathValue::Boolean(false));
                }

                for item in collection.iter() {
                    match item {
                        FhirPathValue::Boolean(true) => return Ok(FhirPathValue::Boolean(true)),
                        FhirPathValue::Boolean(false) => continue,
                        FhirPathValue::Empty => continue, // Empty values are ignored
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "anyTrue() can only be applied to boolean collections"
                                    .to_string(),
                            });
                        }
                    }
                }

                Ok(FhirPathValue::Boolean(false))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            _ => Err(FhirPathError::TypeError {
                message: "anyTrue() can only be applied to boolean values".to_string(),
            }),
        }
    }
}
