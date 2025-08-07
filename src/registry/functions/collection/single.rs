//! single() function - returns the single item if collection has exactly one item

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// single() function - returns the single item if collection has exactly one item
pub struct SingleFunction;

#[async_trait]
impl AsyncFhirPathFunction for SingleFunction {
    fn name(&self) -> &str {
        "single"
    }
    fn human_friendly_name(&self) -> &str {
        "Single"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("single", vec![], TypeInfo::Any));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // single() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns the single item in the input collection. If the input collection does not contain exactly one item, an empty collection is returned."
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
                if items.len() == 1 {
                    Ok(items.iter().next().unwrap().clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            other => Ok(other.clone()), // Single value returns itself
        }
    }
}
