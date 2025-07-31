//! split() function - splits string by separator

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// split() function - splits string by separator
pub struct SplitFunction;

impl FhirPathFunction for SplitFunction {
    fn name(&self) -> &str {
        "split"
    }
    fn human_friendly_name(&self) -> &str {
        "Split"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "split",
                vec![ParameterInfo::required("separator", TypeInfo::String)],
                TypeInfo::collection(TypeInfo::String),
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // split() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(separator)) => {
                let parts: Vec<FhirPathValue> = s
                    .split(separator)
                    .map(|part| FhirPathValue::String(part.to_string()))
                    .collect();
                Ok(FhirPathValue::collection(parts))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}