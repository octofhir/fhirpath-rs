//! Simplified exclude function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// Simplified exclude function: excludes items from the first collection that are in the second
pub struct SimpleExcludeFunction;

impl SimpleExcludeFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleExcludeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleExcludeFunction {
    fn name(&self) -> &'static str {
        "exclude"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "exclude",
            parameters: vec![ParameterType::Collection],
            return_type: ValueType::Collection,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "exclude".to_string(),
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
        let right_set: HashSet<String> = right_items.iter()
            .map(|item| format!("{:?}", item))
            .collect();

        // Exclude items that exist in right collection
        let result: Vec<FhirPathValue> = left_items.into_iter()
            .filter(|item| !right_set.contains(&format!("{:?}", item)))
            .collect();

        Ok(FhirPathValue::Collection(
            octofhir_fhirpath_model::Collection::from(result)
        ))
    }
}