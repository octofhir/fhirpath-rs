//! upper() function - converts to uppercase

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// upper() function - converts to uppercase
pub struct UpperFunction;

impl FhirPathFunction for UpperFunction {
    fn name(&self) -> &str {
        "upper"
    }
    fn human_friendly_name(&self) -> &str {
        "Upper"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("upper", vec![], TypeInfo::String));
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // upper() is a pure string function
    }
    
    fn documentation(&self) -> &str {
        "Returns the input string with all characters converted to upper case."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase())),
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