//! Simplified allTrue function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Simplified allTrue function: returns true if all items in collection are true
pub struct SimpleAllTrueFunction;

impl SimpleAllTrueFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleAllTrueFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleAllTrueFunction {
    fn name(&self) -> &'static str {
        "allTrue"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "allTrue",
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
                function_name: "allTrue".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    return Ok(FhirPathValue::Boolean(true));
                }

                let mut has_boolean_values = false;
                for item in collection.iter() {
                    match item {
                        FhirPathValue::Boolean(false) => return Ok(FhirPathValue::Boolean(false)),
                        FhirPathValue::Boolean(true) => {
                            has_boolean_values = true;
                            continue;
                        }
                        FhirPathValue::Empty => continue, // Empty values are ignored
                        _ => {
                            // Non-boolean values are ignored per FHIRPath specification
                            continue;
                        }
                    }
                }

                // If no boolean values were found, return empty
                if !has_boolean_values {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Boolean(true))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            _ => {
                // Non-boolean single values return empty per FHIRPath specification
                Ok(FhirPathValue::Empty)
            }
        }
    }
}
