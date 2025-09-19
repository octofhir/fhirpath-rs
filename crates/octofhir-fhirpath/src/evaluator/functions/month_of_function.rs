//! MonthOf function implementation
//!
//! The monthOf function extracts the month component from a date or datetime.
//! Syntax: date.monthOf() or datetime.monthOf()

use chrono::Datelike;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// MonthOf function evaluator
pub struct MonthOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MonthOfFunctionEvaluator {
    /// Create a new monthOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "monthOf".to_string(),
                description: "Extracts the month component from a date or datetime".to_string(),
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
impl PureFunctionEvaluator for MonthOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "monthOf function takes no arguments".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // monthOf function should only work on a single value, not collections
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "monthOf function can only be called on a single date or datetime value".to_string(),
            ));
        }

        let value = &input[0];
        let month = match value {
            FhirPathValue::Date(date, _, _) => date.date.month() as i64,
            FhirPathValue::DateTime(datetime, _, _) => datetime.datetime.month() as i64,
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as date or datetime
                use crate::core::temporal::{PrecisionDate, PrecisionDateTime};

                if let Some(precision_date) = PrecisionDate::parse(s) {
                    precision_date.date.month() as i64
                } else if let Some(precision_datetime) = PrecisionDateTime::parse(s) {
                    precision_datetime.datetime.month() as i64
                } else {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        format!("Cannot parse '{}' as Date or DateTime for monthOf function", s),
                    ));
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "monthOf function can only be applied to Date or DateTime values, got {}",
                        value.type_name()
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::integer(month)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}