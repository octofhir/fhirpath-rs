//! today() function - returns current date

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::FunctionSignature;
use chrono::Local;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// today() function - returns current date
pub struct TodayFunction;

impl FhirPathFunction for TodayFunction {
    fn name(&self) -> &str {
        "today"
    }
    fn human_friendly_name(&self) -> &str {
        "Today"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("today", vec![], TypeInfo::Date));
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let today = Local::now().date_naive();
        Ok(FhirPathValue::Date(today))
    }
}