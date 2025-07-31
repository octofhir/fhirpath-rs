//! endsWith() function - checks if string ends with suffix

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// endsWith() function - checks if string ends with suffix
pub struct EndsWithFunction;

impl FhirPathFunction for EndsWithFunction {
    fn name(&self) -> &str {
        "endsWith"
    }
    fn human_friendly_name(&self) -> &str {
        "Ends With"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "endsWith",
                vec![ParameterInfo::required("suffix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // endsWith() is a pure string function
    }

    fn documentation(&self) -> &str {
        "Returns `true` when the input string ends with the given `suffix`. If `suffix` is the empty string (''), the result is `true`."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(suffix)) => {
                Ok(FhirPathValue::Boolean(s.ends_with(suffix)))
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
