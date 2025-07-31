//! single() function - returns the single item if collection has exactly one item

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// single() function - returns the single item if collection has exactly one item
pub struct SingleFunction;

impl FhirPathFunction for SingleFunction {
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
    
    fn evaluate(
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