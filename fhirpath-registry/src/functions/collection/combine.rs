//! combine() function - concatenates two collections

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// combine() function - concatenates two collections
pub struct CombineFunction;

impl FhirPathFunction for CombineFunction {
    fn name(&self) -> &str {
        "combine"
    }
    fn human_friendly_name(&self) -> &str {
        "Combine"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "combine",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // combine() is a pure collection function
    }
    
    fn documentation(&self) -> &str {
        "Returns a collection that contains all items in the input collection, followed by all items in the other collection. Duplicates are not removed."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let other = &args[0];
        let mut result = context
            .input
            .clone()
            .to_collection()
            .into_iter()
            .collect::<Vec<_>>();
        result.extend(other.clone().to_collection().into_iter());
        Ok(FhirPathValue::collection(result))
    }
}