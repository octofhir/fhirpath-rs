//! first() function - returns the first item in the collection

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// first() function - returns the first item in the collection
pub struct FirstFunction;

impl FhirPathFunction for FirstFunction {
    fn name(&self) -> &str {
        "first"
    }
    fn human_friendly_name(&self) -> &str {
        "First"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("first", vec![], TypeInfo::Any));
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // first() is a pure collection function
    }
    
    fn documentation(&self) -> &str {
        "Returns a collection containing only the first item in the input collection. This function is equivalent to `item(0)`. Returns empty (`{ }`) if the input collection has no items."
    }
    
    fn evaluate(
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
                    Ok(items.iter().next().unwrap().clone())
                }
            }
            other => Ok(other.clone()), // Single value is its own first
        }
    }
}