//! YearOf function implementation - sync version

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use chrono::Datelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// YearOf function - extracts year component from Date or DateTime
#[derive(Debug, Clone)]
pub struct YearOfFunction;

impl YearOfFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for YearOfFunction {
    fn name(&self) -> &'static str {
        "yearOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "yearOf",
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
        validation::validate_no_args(args, "yearOf")?;

        let year = match &context.input {
            FhirPathValue::Date(date) => date.date.year() as i64,
            FhirPathValue::DateTime(datetime) => datetime.datetime.year() as i64,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Date(date) => {
                            results.push(FhirPathValue::Integer(date.date.year() as i64));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::Integer(datetime.datetime.year() as i64));
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: "yearOf() can only be called on Date or DateTime values"
                                    .to_string(),
                            });
                        }
                    }
                }
                return Ok(FhirPathValue::collection(results));
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "yearOf() can only be called on Date or DateTime values".to_string(),
                });
            }
        };

        Ok(FhirPathValue::Integer(year))
    }
}

impl Default for YearOfFunction {
    fn default() -> Self {
        Self::new()
    }
}
