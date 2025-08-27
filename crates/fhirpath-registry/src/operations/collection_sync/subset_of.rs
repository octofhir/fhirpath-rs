//! Simplified subsetOf function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// Simplified subsetOf function: returns true if left collection is subset of right
pub struct SimpleSubsetOfFunction;

impl SimpleSubsetOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSubsetOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSubsetOfFunction {
    fn name(&self) -> &'static str {
        "subsetOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "subsetOf",
                parameters: vec![ParameterType::Collection],
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
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "subsetOf".to_string(),
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

        // Check if all left items exist in right set
        for item in left_items {
            let key = format!("{item:?}");
            if !right_set.contains(&key) {
                return Ok(FhirPathValue::Boolean(false));
            }
        }

        Ok(FhirPathValue::Boolean(true))
    }
}
