//! now() function - returns current date/time

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::FunctionSignature;
use chrono::Utc;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// now() function - returns current date/time
pub struct NowFunction;

impl FhirPathFunction for NowFunction {
    fn name(&self) -> &str {
        "now"
    }
    fn human_friendly_name(&self) -> &str {
        "Now"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("now", vec![], TypeInfo::DateTime));
        &SIG
    }
    
    fn documentation(&self) -> &str {
        "Returns the current date and time, including timezone information."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let now = Utc::now();
        Ok(FhirPathValue::DateTime(
            now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        ))
    }
}