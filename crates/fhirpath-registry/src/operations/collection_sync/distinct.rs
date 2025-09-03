//! Simplified distinct function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use std::collections::HashSet;

/// Simplified distinct function: returns unique items from collection
pub struct SimpleDistinctFunction;

impl SimpleDistinctFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleDistinctFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleDistinctFunction {
    fn name(&self) -> &'static str {
        "distinct"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "distinct",
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
                function_name: "distinct".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(collection) => {
                let mut seen = HashSet::new();
                let mut unique_items = Vec::new();

                for item in collection.iter() {
                    // Use string representation for hash comparison
                    let key = format!("{item:?}");
                    if !seen.contains(&key) {
                        seen.insert(key);
                        unique_items.push(item.clone());
                    }
                }

                Ok(FhirPathValue::Collection(unique_items))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![])),
            _ => {
                // Single item is always distinct
                Ok(FhirPathValue::Collection(vec![context.input.clone()]))
            }
        }
    }
}
