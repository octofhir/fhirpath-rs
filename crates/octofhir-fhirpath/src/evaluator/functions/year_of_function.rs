//! YearOf function implementation
//!
//! The yearOf function extracts the year component from a date or datetime.
//! Syntax: date.yearOf() or datetime.yearOf()

use chrono::Datelike;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// YearOf function evaluator
pub struct YearOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl YearOfFunctionEvaluator {
    /// Create a new yearOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "yearOf".to_string(),
                description: "Extracts the year component from a date or datetime".to_string(),
                signature: FunctionSignature {
                    input_type: "Date | DateTime | String".to_string(),
                    parameters: vec![],
                    return_type: "Integer".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for YearOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "yearOf function takes no arguments".to_string(),
            ));
        }

        // yearOf() is a scalar function that requires exactly one input value
        if input.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                format!(
                    "yearOf function can only be applied to a single value, got {} values",
                    input.len()
                ),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let value = &input[0];
        let year = match &value {
            FhirPathValue::Date(date, _, _) => date.date.year() as i64,
            FhirPathValue::DateTime(datetime, _, _) => datetime.datetime.year() as i64,
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as date or datetime
                use crate::core::temporal::{PrecisionDate, PrecisionDateTime};

                if let Some(precision_date) = PrecisionDate::parse(s) {
                    precision_date.date.year() as i64
                } else if let Some(precision_datetime) = PrecisionDateTime::parse(s) {
                    precision_datetime.datetime.year() as i64
                } else {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        format!("Cannot parse '{s}' as Date or DateTime for yearOf function"),
                    ));
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "yearOf function can only be applied to Date or DateTime values, got {}",
                        value.type_name()
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::integer(year)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
