//! startsWith() function - checks if string starts with prefix

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
/// startsWith() function - checks if string starts with prefix
pub struct StartsWithFunction;

#[async_trait]
impl AsyncFhirPathFunction for StartsWithFunction {
    fn name(&self) -> &str {
        "startsWith"
    }
    fn human_friendly_name(&self) -> &str {
        "Starts With"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "startsWith",
                vec![ParameterInfo::required("prefix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // startsWith() is a pure string function
    }

    fn documentation(&self) -> &str {
        "Returns `true` when the input string starts with the given `prefix`. If `prefix` is the empty string (''), the result is `true`."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(prefix)) => {
                Ok(FhirPathValue::Boolean(s.starts_with(prefix)))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
