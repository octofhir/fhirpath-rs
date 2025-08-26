//! MinuteOf function implementation - sync version

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// MinuteOf function - extracts minute component from DateTime or Time (0-59)
#[derive(Debug, Clone)]
pub struct MinuteOfFunction;

impl MinuteOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for MinuteOfFunction {
    fn name(&self) -> &'static str {
        "minuteOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "minuteOf",
                parameters: vec![],
                return_type: ValueType::Integer,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "minuteOf")?;

        let minute = match &context.input {
            FhirPathValue::DateTime(datetime) => datetime.datetime.minute() as i64,
            FhirPathValue::Time(time) => time.time.minute() as i64,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::Integer(datetime.datetime.minute() as i64));
                        }
                        FhirPathValue::Time(time) => {
                            results.push(FhirPathValue::Integer(time.time.minute() as i64));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "minuteOf() can only be called on DateTime or Time values"
                                    .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "minuteOf() can only be called on DateTime or Time values".to_string(),
                });
            }
        };

        Ok(FhirPathValue::Integer(minute))
    }
}

impl Default for MinuteOfFunction {
    fn default() -> Self {
        Self::new()
    }
}
