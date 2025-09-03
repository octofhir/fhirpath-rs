//! SecondOf function implementation - sync version

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// SecondOf function - extracts second component from DateTime or Time (0-59)
#[derive(Debug, Clone)]
pub struct SecondOfFunction;

impl SecondOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for SecondOfFunction {
    fn name(&self) -> &'static str {
        "secondOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "secondOf",
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
        validation::validate_no_args(args, "secondOf")?;

        let second = match &context.input {
            FhirPathValue::DateTime(datetime) => datetime.datetime.second() as i64,
            FhirPathValue::Time(time) => time.time.second() as i64,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::Integer(datetime.datetime.second() as i64));
                        }
                        FhirPathValue::Time(time) => {
                            results.push(FhirPathValue::Integer(time.time.second() as i64));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "secondOf() can only be called on DateTime or Time values"
                                    .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "secondOf() can only be called on DateTime or Time values".to_string(),
                });
            }
        };

        Ok(FhirPathValue::Integer(second))
    }
}

impl Default for SecondOfFunction {
    fn default() -> Self {
        Self::new()
    }
}
