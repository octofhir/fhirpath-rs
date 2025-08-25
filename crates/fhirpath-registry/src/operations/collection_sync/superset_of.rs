//! Simplified supersetOf function implementation for FHIRPath

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

/// Simplified supersetOf function: returns true if left collection is superset of right
pub struct SimpleSupersetOfFunction;

impl SimpleSupersetOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleSupersetOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimpleSupersetOfFunction {
    fn name(&self) -> &'static str {
        "supersetOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| FunctionSignature {
            name: "supersetOf",
            parameters: vec![ParameterType::Collection],
            return_type: ValueType::Boolean,
            variadic: false,
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "supersetOf".to_string(),
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

        // Create set of left items for fast lookup
        let left_set: HashSet<String> = left_items.iter()
            .map(|item| format!("{:?}", item))
            .collect();

        // Check if all right items exist in left set
        for item in right_items {
            let key = format!("{:?}", item);
            if !left_set.contains(&key) {
                return Ok(FhirPathValue::Boolean(false));
            }
        }

        Ok(FhirPathValue::Boolean(true))
    }
}