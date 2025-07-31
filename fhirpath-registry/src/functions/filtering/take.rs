//! take() function - takes first n elements

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// take() function - takes first n elements
pub struct TakeFunction;

impl FhirPathFunction for TakeFunction {
    fn name(&self) -> &str {
        "take"
    }
    fn human_friendly_name(&self) -> &str {
        "Take"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "take",
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
        let result: Vec<FhirPathValue> = items.into_iter().take(num).collect();
        Ok(FhirPathValue::collection(result))
    }
}