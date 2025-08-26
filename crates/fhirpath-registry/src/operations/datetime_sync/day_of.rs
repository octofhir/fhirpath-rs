//! DayOf function implementation - sync version

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use chrono::Datelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// DayOf function - extracts day component from Date or DateTime (1-31)
#[derive(Debug, Clone)]
pub struct DayOfFunction;

impl DayOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for DayOfFunction {
    fn name(&self) -> &'static str {
        "dayOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "dayOf",
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
        validation::validate_no_args(args, "dayOf")?;

        let day = match &context.input {
            FhirPathValue::Date(date) => date.date.day() as i64,
            FhirPathValue::DateTime(datetime) => datetime.datetime.day() as i64,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Date(date) => {
                            results.push(FhirPathValue::Integer(date.date.day() as i64));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::Integer(datetime.datetime.day() as i64));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "dayOf() can only be called on Date or DateTime values"
                                    .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "dayOf() can only be called on Date or DateTime values".to_string(),
                });
            }
        };

        Ok(FhirPathValue::Integer(day))
    }
}

impl Default for DayOfFunction {
    fn default() -> Self {
        Self::new()
    }
}
