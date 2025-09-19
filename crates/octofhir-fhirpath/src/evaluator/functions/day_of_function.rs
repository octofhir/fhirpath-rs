//! DayOf function implementation
//!
//! The dayOf function extracts the day component from a date or datetime.
//! Syntax: date.dayOf() or datetime.dayOf()

use chrono::Datelike;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// DayOf function evaluator
pub struct DayOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DayOfFunctionEvaluator {
    /// Create a new dayOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "dayOf".to_string(),
                description: "Extracts the day component from a date or datetime".to_string(),
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
impl PureFunctionEvaluator for DayOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "dayOf function takes no arguments".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // dayOf function should only work on a single value, not collections
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "dayOf function can only be called on a single date or datetime value".to_string(),
            ));
        }

        let value = &input[0];
        let day = match value {
            FhirPathValue::Date(date, _, _) => date.date.day() as i64,
            FhirPathValue::DateTime(datetime, _, _) => datetime.datetime.day() as i64,
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as date or datetime
                use crate::core::temporal::{PrecisionDate, PrecisionDateTime};

                if let Some(precision_date) = PrecisionDate::parse(s) {
                    precision_date.date.day() as i64
                } else if let Some(precision_datetime) = PrecisionDateTime::parse(s) {
                    precision_datetime.datetime.day() as i64
                } else {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        format!(
                            "Cannot parse '{}' as Date or DateTime for dayOf function",
                            s
                        ),
                    ));
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "dayOf function can only be applied to Date or DateTime values, got {}",
                        value.type_name()
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::integer(day)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
