//! skip() function - skips first n elements

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// skip() function - skips first n elements
pub struct SkipFunction;

impl FhirPathFunction for SkipFunction {
    fn name(&self) -> &str {
        "skip"
    }
    fn human_friendly_name(&self) -> &str {
        "Skip"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "skip",
                vec![ParameterInfo::required("num", TypeInfo::Integer)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let num = match &args[0] {
            FhirPathValue::Integer(n) => *n as usize,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        let items = context.input.clone().to_collection();
        let result: Vec<FhirPathValue> = items.into_iter().skip(num).collect();
        Ok(FhirPathValue::collection(result))
    }
}
