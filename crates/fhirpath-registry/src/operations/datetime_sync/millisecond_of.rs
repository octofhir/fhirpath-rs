//! MillisecondOf function implementation - sync version

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// MillisecondOf function - extracts millisecond component from DateTime or Time (0-999)
#[derive(Debug, Clone)]
pub struct MillisecondOfFunction;

impl MillisecondOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for MillisecondOfFunction {
    fn name(&self) -> &'static str {
        "millisecondOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "millisecondOf",
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
        validation::validate_no_args(args, "millisecondOf")?;

        let millisecond = match &context.input {
            FhirPathValue::DateTime(datetime) => {
                let nanoseconds = datetime.datetime.nanosecond();
                (nanoseconds / 1_000_000) as i64
            }
            FhirPathValue::Time(time) => {
                let nanoseconds = time.time.nanosecond();
                (nanoseconds / 1_000_000) as i64
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            let nanoseconds = datetime.datetime.nanosecond();
                            results.push(FhirPathValue::Integer((nanoseconds / 1_000_000) as i64));
                        }
                        FhirPathValue::Time(time) => {
                            let nanoseconds = time.time.nanosecond();
                            results.push(FhirPathValue::Integer((nanoseconds / 1_000_000) as i64));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message:
                                    "millisecondOf() can only be called on DateTime or Time values"
                                        .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "millisecondOf() can only be called on DateTime or Time values"
                        .to_string(),
                });
            }
        };

        Ok(FhirPathValue::Integer(millisecond))
    }
}

impl Default for MillisecondOfFunction {
    fn default() -> Self {
        Self::new()
    }
}
