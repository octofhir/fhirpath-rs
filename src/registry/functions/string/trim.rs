//! trim() function - removes whitespace from both ends

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;

/// trim() function - removes whitespace from both ends
pub struct TrimFunction;

impl FhirPathFunction for TrimFunction {
    fn name(&self) -> &str {
        "trim"
    }
    fn human_friendly_name(&self) -> &str {
        "Trim"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("trim", vec![], TypeInfo::String));
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // trim() is a pure string function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.trim().to_string())),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
