//! Now function implementation - async version (system calls)
use octofhir_fhirpath_core::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{AsyncOperation, EvaluationContext, validation};
use async_trait::async_trait;
use chrono::Utc;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_core::FhirPathValue;

/// Now function - returns current date and time (requires system call)
#[derive(Debug, Clone)]
pub struct NowFunction;

impl NowFunction {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AsyncOperation for NowFunction {
    fn name(&self) -> &'static str {
        "now"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "now",
                parameters: vec![],
                return_type: ValueType::DateTime,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    async fn execute(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "now")?;

        // System call to get current time
        let now = Utc::now().fixed_offset();
        Ok(FhirPathValue::DateTime(PrecisionDateTime::new(
            now,
            TemporalPrecision::Millisecond, // Full precision for now()
        )))
    }
}

impl Default for NowFunction {
    fn default() -> Self {
        Self::new()
    }
}
