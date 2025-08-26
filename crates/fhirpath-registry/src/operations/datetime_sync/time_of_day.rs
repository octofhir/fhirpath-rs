//! TimeOfDay function implementation - sync version

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, PrecisionTime, TemporalPrecision};

/// TimeOfDay function - extracts time portion from DateTime
#[derive(Debug, Clone)]
pub struct TimeOfDayFunction;

impl TimeOfDayFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for TimeOfDayFunction {
    fn name(&self) -> &'static str {
        "timeOfDay"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "timeOfDay",
                parameters: vec![],
                return_type: ValueType::Time,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "timeOfDay")?;

        let time = match &context.input {
            FhirPathValue::DateTime(datetime) => {
                // Extract time portion from datetime
                let time = datetime.datetime.time();
                let precision_time = PrecisionTime::new(time, TemporalPrecision::Millisecond);
                FhirPathValue::Time(precision_time)
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            let time = datetime.datetime.time();
                            let precision_time =
                                PrecisionTime::new(time, TemporalPrecision::Millisecond);
                            results.push(FhirPathValue::Time(precision_time));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "timeOfDay() can only be called on DateTime values"
                                    .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "timeOfDay() can only be called on DateTime values".to_string(),
                });
            }
        };

        Ok(time)
    }
}

impl Default for TimeOfDayFunction {
    fn default() -> Self {
        Self::new()
    }
}
