//! Date and time functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use chrono::{Utc, Local, Date, DateTime};

/// now() function - returns current date/time
pub struct NowFunction;

impl FhirPathFunction for NowFunction {
    fn name(&self) -> &str { "now" }
    fn human_friendly_name(&self) -> &str { "Now" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "now",
                vec![],
                TypeInfo::DateTime,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let now = Utc::now();
        Ok(FhirPathValue::DateTime(now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())))
    }
}

/// today() function - returns current date
pub struct TodayFunction;

impl FhirPathFunction for TodayFunction {
    fn name(&self) -> &str { "today" }
    fn human_friendly_name(&self) -> &str { "Today" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "today",
                vec![],
                TypeInfo::Date,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let today = Local::now().date_naive();
        Ok(FhirPathValue::Date(today))
    }
}