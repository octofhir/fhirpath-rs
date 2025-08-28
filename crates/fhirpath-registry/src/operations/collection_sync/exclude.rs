//! Simplified exclude function implementation for FHIRPath

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Simplified exclude function: excludes items from the first collection that are in the second
pub struct SimpleExcludeFunction;

impl SimpleExcludeFunction {
    pub fn new() -> Self {
        Self
    }

    /// Check if two FhirPathValues are equivalent for exclude operation
    fn values_equivalent(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use FhirPathValue::*;
        match (left, right) {
            (String(a), String(b)) => a == b,
            (Integer(a), Integer(b)) => a == b,
            (Decimal(a), Decimal(b)) => a == b,
            (Boolean(a), Boolean(b)) => a == b,
            (Date(a), Date(b)) => a == b,
            (DateTime(a), DateTime(b)) => a == b,
            (Time(a), Time(b)) => a == b,
            (JsonValue(a), JsonValue(b)) => a.as_inner() == b.as_inner(),
            // Handle JsonValue vs String comparison (common case)
            (JsonValue(a), String(b)) | (String(b), JsonValue(a)) => {
                if let Some(a_str) = a.as_inner().as_str() {
                    a_str == b.as_ref()
                } else {
                    false
                }
            }
            // For different types or complex types, use debug comparison as fallback
            _ => format!("{left:?}") == format!("{right:?}"),
        }
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
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "exclude",
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

        // Exclude items that exist in right collection using proper equivalence
        let result: Vec<FhirPathValue> = left_items
            .into_iter()
            .filter(|left_item| {
                !right_items
                    .iter()
                    .any(|right_item| self.values_equivalent(left_item, right_item))
            })
            .collect();

        Ok(FhirPathValue::Collection(
            octofhir_fhirpath_model::Collection::from(result),
        ))
    }
}
