//! tail() function - returns all items except the first

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// tail() function - returns all items except the first
pub struct TailFunction;

#[async_trait]
impl AsyncFhirPathFunction for TailFunction {
    fn name(&self) -> &str {
        "tail"
    }
    fn human_friendly_name(&self) -> &str {
        "Tail"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "tail",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // tail() is a pure collection function
    }

    fn documentation(&self) -> &str {
        "Returns a collection containing all but the first item in the input collection. If the input collection is empty, an empty collection is returned."
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
                if items.len() <= 1 {
                    Ok(FhirPathValue::Empty)
                } else {
                    let tail_items: Vec<FhirPathValue> = items.iter().skip(1).cloned().collect();
                    Ok(FhirPathValue::collection(tail_items))
                }
            }
            _ => Ok(FhirPathValue::Empty), // Single value's tail is empty
        }
    }
}
