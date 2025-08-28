//! HasValue function implementation - sync version

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// HasValue function - returns true if the input collection contains exactly one item that has a value
#[derive(Debug, Clone)]
pub struct HasValueFunction;

impl HasValueFunction {
    pub fn new() -> Self {
        Self
    }

    fn item_has_value(&self, item: &FhirPathValue) -> bool {
        match item {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => !items.is_empty(),
            FhirPathValue::String(s) => !s.is_empty(),
            FhirPathValue::JsonValue(json) => {
                let inner = json.as_inner();
                if inner.is_object() {
                    // For objects, we can only check if it's an object (non-empty by definition if it parses)
                    true
                } else if inner.is_array() {
                    // For arrays, we can only check if it's an array (non-empty by definition if it parses)
                    true
                } else if let Some(s) = inner.as_str() {
                    !s.is_empty()
                } else {
                    !inner.is_null()
                }
            }
            // All other value types are considered to have value if they exist
            _ => true,
        }
    }
}

impl SyncOperation for HasValueFunction {
    fn name(&self) -> &'static str {
        "hasValue"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "hasValue",
                parameters: vec![],
                return_type: ValueType::Boolean,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "hasValue")?;

        let input = &context.input;

        let has_value = match input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item that is not empty/null
                items.len() == 1 && self.item_has_value(items.get(0).unwrap())
            }
            _ => {
                // Single item - check if it has a value
                self.item_has_value(input)
            }
        };

        Ok(FhirPathValue::Boolean(has_value))
    }
}

impl Default for HasValueFunction {
    fn default() -> Self {
        Self::new()
    }
}
