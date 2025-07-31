//! distinct() function - returns unique items in the collection

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// distinct() function - returns unique items in the collection
pub struct DistinctFunction;

impl FhirPathFunction for DistinctFunction {
    fn name(&self) -> &str {
        "distinct"
    }
    fn human_friendly_name(&self) -> &str {
        "Distinct"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "distinct",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // distinct() is a pure collection function
    }
    
    fn documentation(&self) -> &str {
        "Returns a collection containing only the unique items in the input collection. To determine whether two items are the same, the equals (`=`) operator is used."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = context.input.clone().to_collection();
        let mut unique = Vec::new();
        for item in items.into_iter() {
            if !unique.iter().any(|u| u == &item) {
                unique.push(item);
            }
        }
        Ok(FhirPathValue::collection(unique))
    }
}