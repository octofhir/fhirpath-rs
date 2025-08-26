//! Simplified isDistinct function implementation for FHIRPath

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// Simplified isDistinct function: returns true if all items in collection are unique
pub struct SimpleIsDistinctFunction;

impl SimpleIsDistinctFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleIsDistinctFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleIsDistinctFunction {
    fn name(&self) -> &'static str {
        "isDistinct"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "isDistinct",
                parameters: vec![],
                return_type: ValueType::Boolean,
                variadic: false,
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
                function_name: "isDistinct".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                let mut seen = HashSet::new();

                for item in collection.iter() {
                    let key = format!("{item:?}");
                    if seen.contains(&key) {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                    seen.insert(key);
                }

                Ok(FhirPathValue::Boolean(true))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => {
                // Single item is always distinct
                Ok(FhirPathValue::Boolean(true))
            }
        }
    }
}
