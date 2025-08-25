//! Today function implementation - async version (system calls)

use crate::traits::{AsyncOperation, EvaluationContext, validation};
use crate::signature::{FunctionSignature, ValueType};
use async_trait::async_trait;
use chrono::Utc;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::{FhirPathValue, PrecisionDate, TemporalPrecision};

/// Today function - returns current date (requires system call)
#[derive(Debug, Clone)]
pub struct TodayFunction;

impl TodayFunction {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AsyncOperation for TodayFunction {
    fn name(&self) -> &'static str {
        "today"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "today",
                parameters: vec![],
                return_type: ValueType::Date,
                variadic: false,
            }
        });
        &SIGNATURE
    }

    async fn execute(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "today")?;

        // System call to get current date
        let today = Utc::now().date_naive();
        Ok(FhirPathValue::Date(PrecisionDate::new(
            today,
            TemporalPrecision::Day, // Day precision for today()
        )))
    }
}

impl Default for TodayFunction {
    fn default() -> Self {
        Self::new()
    }
}