//! indexOf() function - finds index of substring

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// indexOf() function - finds index of substring
pub struct IndexOfFunction;

impl FhirPathFunction for IndexOfFunction {
    fn name(&self) -> &str {
        "indexOf"
    }
    fn human_friendly_name(&self) -> &str {
        "Index Of"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "indexOf",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Integer,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // indexOf() is a pure string function
    }
    
    fn documentation(&self) -> &str {
        "Returns the 0-based index of the first position `substring` is found in the input string, or -1 if it is not found."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                match s.find(substring) {
                    Some(index) => Ok(FhirPathValue::Integer(index as i64)),
                    None => Ok(FhirPathValue::Integer(-1)),
                }
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