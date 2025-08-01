//! exclude() function - returns items in first collection but not in second

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// exclude() function - returns items in first collection but not in second
pub struct ExcludeFunction;

impl FhirPathFunction for ExcludeFunction {
    fn name(&self) -> &str {
        "exclude"
    }
    fn human_friendly_name(&self) -> &str {
        "Exclude"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "exclude",
                vec![ParameterInfo::required("other", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // exclude() is a pure collection function
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
            if !right.iter().any(|r| r == &item) {
                result.push(item);
            }
        }
        Ok(FhirPathValue::collection(result))
    }
}
