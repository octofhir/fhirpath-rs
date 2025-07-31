//! hasValue() function - checks if a value exists (not empty)

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::FunctionSignature;

/// hasValue() function - returns true if the input is not empty
pub struct HasValueFunction;

impl FhirPathFunction for HasValueFunction {
    fn name(&self) -> &str {
        "hasValue"
    }

    fn human_friendly_name(&self) -> &str {
        "Has Value"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "hasValue",
                vec![], // No parameters
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // hasValue() returns true if the input is not empty
        let has_value = match &context.input {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(coll) => !coll.is_empty(),
            _ => true, // Any other value means it exists
        };

        Ok(FhirPathValue::Boolean(has_value))
    }
}
