//! HourOf function implementation - sync version

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// HourOf function - extracts hour component from DateTime or Time (0-23)
#[derive(Debug, Clone)]
pub struct HourOfFunction;

impl HourOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for HourOfFunction {
    fn name(&self) -> &'static str {
        "hourOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "hourOf",
                parameters: vec![],
                return_type: ValueType::Integer,
                variadic: false,
                category: FunctionCategory::Scalar,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "hourOf")?;

        let hour = match &context.input {
            FhirPathValue::DateTime(datetime) => datetime.datetime.hour() as i64,
            FhirPathValue::Time(time) => time.time.hour() as i64,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::Integer(datetime.datetime.hour() as i64));
                        }
                        FhirPathValue::Time(time) => {
                            results.push(FhirPathValue::Integer(time.time.hour() as i64));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "hourOf() can only be called on DateTime or Time values"
                                    .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "hourOf() can only be called on DateTime or Time values".to_string(),
                });
            }
        };

        Ok(FhirPathValue::Integer(hour))
    }
}

impl Default for HourOfFunction {
    fn default() -> Self {
        Self::new()
    }
}
