//! contains() function - checks if string contains substring

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// contains() function - checks if string contains substring
pub struct ContainsFunction;

impl FhirPathFunction for ContainsFunction {
    fn name(&self) -> &str {
        "contains"
    }
    fn human_friendly_name(&self) -> &str {
        "Contains"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "contains",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // contains() is a pure string function
    }
    
    fn documentation(&self) -> &str {
        "Returns `true` when the given `substring` is a substring of the input string. If `substring` is the empty string (''), the result is `true`."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                Ok(FhirPathValue::Boolean(s.contains(substring)))
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