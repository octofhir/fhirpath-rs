//! last() function - returns the last item in the collection

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// last() function - returns the last item in the collection
pub struct LastFunction;

#[async_trait]
impl AsyncFhirPathFunction for LastFunction {
    fn name(&self) -> &str {
        "last"
    }
    fn human_friendly_name(&self) -> &str {
        "Last"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("last", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // last() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns a collection containing only the last item in the input collection. Returns empty (`{ }`) if the input collection has no items."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(items.iter().last().unwrap().clone())
                }
            }
            other => Ok(other.clone()), // Single value is its own last
        }
    }
}
