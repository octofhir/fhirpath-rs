//! lower() function - converts to lowercase

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// lower() function - converts to lowercase
pub struct LowerFunction;

#[async_trait]
impl AsyncFhirPathFunction for LowerFunction {
    fn name(&self) -> &str {
        "lower"
    }
    fn human_friendly_name(&self) -> &str {
        "Lower"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("lower", vec![], TypeInfo::String));
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // lower() is a pure string function
    }

    fn documentation(&self) -> &str {
        "Returns the input string with all characters converted to lower case."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.as_ref().to_lowercase().into())),
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
