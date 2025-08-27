//! Simplified intersect function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// Simplified intersect function: returns the intersection of two collections
pub struct SimpleIntersectFunction;

impl SimpleIntersectFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleIntersectFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleIntersectFunction {
    fn name(&self) -> &'static str {
        "intersect"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "intersect",
                parameters: vec![ParameterType::Collection],
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
                function_name: "intersect".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Convert input to collection
        let left_items = match &context.input {
            FhirPathValue::Collection(collection) => collection.iter().cloned().collect::<Vec<_>>(),
            FhirPathValue::Empty => vec![],
            _ => vec![context.input.clone()],
        };

        // Convert argument to collection
        let right_items = match &args[0] {
            FhirPathValue::Collection(collection) => collection.iter().cloned().collect::<Vec<_>>(),
            FhirPathValue::Empty => vec![],
            _ => vec![args[0].clone()],
        };

        // Create set of right items for fast lookup
        let right_set: HashSet<String> =
            right_items.iter().map(|item| format!("{item:?}")).collect();

        // Find intersection
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for item in left_items {
            let key = format!("{item:?}");
            if right_set.contains(&key) && !seen.contains(&key) {
                seen.insert(key);
                result.push(item);
            }
        }

        Ok(FhirPathValue::Collection(
            octofhir_fhirpath_model::Collection::from(result),
        ))
    }
}
