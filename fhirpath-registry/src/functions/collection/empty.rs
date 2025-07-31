//! empty() function - returns true if the collection is empty

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// empty() function - returns true if the collection is empty
pub struct EmptyFunction;

impl FhirPathFunction for EmptyFunction {
    fn name(&self) -> &str {
        "empty"
    }
    fn human_friendly_name(&self) -> &str {
        "Empty"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("empty", vec![], TypeInfo::Boolean));
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // empty() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns `true` if the input collection is empty (`{ }`) and `false` otherwise."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let is_empty = match &context.input {
            FhirPathValue::Empty => true,
            FhirPathValue::Collection(items) => items.is_empty(),
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            is_empty,
        )]))
    }
}