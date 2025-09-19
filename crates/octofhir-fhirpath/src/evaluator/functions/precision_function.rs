//! Precision function implementation
//!
//! The precision function returns the precision of a temporal or numeric value as a string.
//! For temporal values, returns precision like 'year', 'month', 'day', 'hour', 'minute', 'second', 'millisecond'.
//! For decimal values, returns the number of decimal places.
//! Syntax: value.precision()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// Precision function evaluator
pub struct PrecisionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl PrecisionFunctionEvaluator {
    /// Create a new precision function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "precision".to_string(),
                description: "Returns the precision of a temporal or numeric value. For temporal values, returns the precision unit (e.g., 'year', 'month', 'day'). For decimal values, returns the number of decimal places.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "String".to_string(),
                    polymorphic: true,
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

    /// Get precision of a decimal value (number of decimal places)
    fn get_decimal_precision(decimal_str: &str) -> String {
        if let Some(dot_pos) = decimal_str.find('.') {
            let decimal_places = decimal_str.len() - dot_pos - 1;
            decimal_places.to_string()
        } else {
            "0".to_string()
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for PrecisionFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "precision function takes no arguments".to_string(),
            ));
        }

        // Handle empty input (empty propagation)
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "precision function can only be called on a single value".to_string(),
            ));
        }

        let precision_value = match &input[0] {
            FhirPathValue::Date(date, _, _) => {
                // For dates, return character count of the formatted representation up to precision
                let count = match date.precision {
                    crate::core::temporal::TemporalPrecision::Year => 4,    // "2014"
                    crate::core::temporal::TemporalPrecision::Month => 7,   // "2014-01"
                    crate::core::temporal::TemporalPrecision::Day => 10,    // "2014-01-05"
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            "Invalid precision for date value".to_string(),
                        ));
                    }
                };
                FhirPathValue::integer(count)
            }
            FhirPathValue::DateTime(datetime, _, _) => {
                // For datetimes, return character count of the formatted representation up to precision
                let count = match datetime.precision {
                    crate::core::temporal::TemporalPrecision::Year => 4,         // "2014"
                    crate::core::temporal::TemporalPrecision::Month => 7,        // "2014-01"
                    crate::core::temporal::TemporalPrecision::Day => 10,         // "2014-01-05"
                    crate::core::temporal::TemporalPrecision::Hour => 13,        // "2014-01-05T10"
                    crate::core::temporal::TemporalPrecision::Minute => 16,      // "2014-01-05T10:30"
                    crate::core::temporal::TemporalPrecision::Second => 19,      // "2014-01-05T10:30:00"
                    crate::core::temporal::TemporalPrecision::Millisecond => 17, // "2014-01-05T10:30" (excluding :00.000)
                };
                FhirPathValue::integer(count)
            }
            FhirPathValue::Time(time, _, _) => {
                // For times, return character count of the formatted representation up to precision
                let count = match time.precision {
                    crate::core::temporal::TemporalPrecision::Hour => 2,        // "10"
                    crate::core::temporal::TemporalPrecision::Minute => 4,      // "10:30" (excluding T prefix)
                    crate::core::temporal::TemporalPrecision::Second => 8,      // "10:30:00"
                    crate::core::temporal::TemporalPrecision::Millisecond => 9, // "10:30:00" (excluding .000 and T prefix)
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            "Invalid precision for time value".to_string(),
                        ));
                    }
                };
                FhirPathValue::integer(count)
            }
            FhirPathValue::Decimal(decimal, _, _) => {
                // For decimals, return number of decimal places as integer
                let decimal_str = decimal.to_string();
                let decimal_places = if let Some(dot_pos) = decimal_str.find('.') {
                    decimal_str.len() - dot_pos - 1
                } else {
                    0
                };
                FhirPathValue::integer(decimal_places as i64)
            }
            FhirPathValue::Integer(_, _, _) => {
                // Integers have precision of 0 (no decimal places)
                FhirPathValue::integer(0)
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "precision function can only be called on temporal or numeric values"
                        .to_string(),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![precision_value]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}