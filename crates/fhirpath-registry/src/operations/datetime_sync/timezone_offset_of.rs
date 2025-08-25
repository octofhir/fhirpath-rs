//! TimezoneOffsetOf function implementation - sync version

use crate::traits::{SyncOperation, EvaluationContext, validation};
use crate::signature::{FunctionSignature, ValueType};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;

/// TimezoneOffsetOf function - extracts timezone offset from DateTime (in minutes per FHIRPath spec)
#[derive(Debug, Clone)]
pub struct TimezoneOffsetOfFunction;

impl TimezoneOffsetOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for TimezoneOffsetOfFunction {
    fn name(&self) -> &'static str {
        "timezoneOffsetOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "timezoneOffsetOf",
                parameters: vec![],
                return_type: ValueType::Decimal,
                variadic: false,
            }
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "timezoneOffsetOf")?;

        let offset = match &context.input {
            FhirPathValue::DateTime(datetime) => {
                let offset_seconds = datetime.datetime.offset().local_minus_utc();
                let offset_minutes = Decimal::new(offset_seconds as i64, 0) / Decimal::new(60, 0);
                FhirPathValue::Decimal(offset_minutes)
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            let offset_seconds = datetime.datetime.offset().local_minus_utc();
                            let offset_minutes = Decimal::new(offset_seconds as i64, 0) / Decimal::new(60, 0);
                            results.push(FhirPathValue::Decimal(offset_minutes));
                        }
                        _ => return Err(FhirPathError::TypeError {
                            message: "timezoneOffsetOf() can only be called on DateTime values".to_string()
                        }),
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => return Err(FhirPathError::TypeError {
                message: "timezoneOffsetOf() can only be called on DateTime values".to_string()
            }),
        };

        Ok(offset)
    }
}

impl Default for TimezoneOffsetOfFunction {
    fn default() -> Self {
        Self::new()
    }
}