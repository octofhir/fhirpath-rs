//! intersect() function - returns the intersection of two collections

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// intersect() function - returns the intersection of two collections
pub struct IntersectFunction;

impl FhirPathFunction for IntersectFunction {
    fn name(&self) -> &str {
        "intersect"
    }
    fn human_friendly_name(&self) -> &str {
        "Intersect"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "intersect",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // intersect() is a pure collection function
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let other = &args[0];
        let left = context.input.clone().to_collection();
        let right = other.clone().to_collection();

        let mut result = Vec::new();
        for item in left.into_iter() {
            if right.iter().any(|r| r == &item) && !result.iter().any(|res| res == &item) {
                result.push(item);
            }
        }
        Ok(FhirPathValue::collection(result))
    }
}
