//! count() function - returns the number of elements in the collection

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::FunctionSignature;

/// count() function - returns the number of elements in the collection
pub struct CountFunction;

impl FhirPathFunction for CountFunction {
    fn name(&self) -> &str {
        "count"
    }
    fn human_friendly_name(&self) -> &str {
        "Count"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("count", vec![], TypeInfo::Integer));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // count() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns a collection with a single value which is the integer count of the number of items in the input collection. Returns 0 when the input collection is empty."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(
            count as i64,
        )]))
    }
}
