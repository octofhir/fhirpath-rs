//! Millisecond function implementation
//!
//! The millisecond function extracts the millisecond component from a DateTime or Time value.
//! Returns empty if the value doesn't have millisecond precision.
//!
//! # Examples
//!
//! ```fhirpath
//! @T10:30:15.250.millisecond()  // Returns 250
//! @2024-01-15T10:30:15.001.millisecond()  // Returns 1
//! @T10:30:15.millisecond()  // Returns {} (no millisecond precision)
//! ```

use chrono::Timelike;
use std::sync::Arc;

use crate::core::temporal::TemporalPrecision;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Millisecond function evaluator
pub struct MillisecondFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MillisecondFunctionEvaluator {
    /// Create a new millisecond function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "millisecond".to_string(),
                description: "Extracts the millisecond component (0-999) from a DateTime or Time value with millisecond precision"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "DateTime|Time".to_string(),
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
impl PureFunctionEvaluator for MillisecondFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "millisecond function takes no arguments".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // Require singleton input
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "millisecond function can only be called on a single DateTime or Time value"
                    .to_string(),
            ));
        }

        let value = &input[0];
        match value {
            FhirPathValue::DateTime(dt, _, _) => {
                // Only return millisecond if precision is Millisecond
                if dt.precision == TemporalPrecision::Millisecond {
                    // Extract milliseconds from nanoseconds (chrono stores nanoseconds)
                    let nanos = dt.datetime.nanosecond();
                    let millis = (nanos / 1_000_000) as i64;
                    Ok(EvaluationResult {
                        value: crate::core::Collection::from(vec![FhirPathValue::integer(millis)]),
                    })
                } else {
                    // Return empty if precision doesn't include milliseconds
                    Ok(EvaluationResult {
                        value: crate::core::Collection::empty(),
                    })
                }
            }
            FhirPathValue::Time(time, _, _) => {
                // Only return millisecond if precision is Millisecond
                if time.precision == TemporalPrecision::Millisecond {
                    // Extract milliseconds from nanoseconds (chrono stores nanoseconds)
                    let nanos = time.time.nanosecond();
                    let millis = (nanos / 1_000_000) as i64;
                    Ok(EvaluationResult {
                        value: crate::core::Collection::from(vec![FhirPathValue::integer(millis)]),
                    })
                } else {
                    // Return empty if precision doesn't include milliseconds
                    Ok(EvaluationResult {
                        value: crate::core::Collection::empty(),
                    })
                }
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                format!(
                    "millisecond function can only be applied to DateTime or Time values, got {}",
                    value.type_name()
                ),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::temporal::{PrecisionDateTime, PrecisionTime, TemporalPrecision};
    use chrono::{NaiveDate, NaiveTime};

    #[tokio::test]
    async fn test_millisecond_time_with_milliseconds() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @T10:30:15.250.millisecond() → 250
        let time = NaiveTime::from_hms_milli_opt(10, 30, 15, 250).unwrap();
        let precision_time = PrecisionTime::new(time, TemporalPrecision::Millisecond);
        let input = vec![FhirPathValue::time(precision_time)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(250)));
    }

    #[tokio::test]
    async fn test_millisecond_time_zero_milliseconds() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @T10:30:15.000.millisecond() → 0
        let time = NaiveTime::from_hms_milli_opt(10, 30, 15, 0).unwrap();
        let precision_time = PrecisionTime::new(time, TemporalPrecision::Millisecond);
        let input = vec![FhirPathValue::time(precision_time)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(0)));
    }

    #[tokio::test]
    async fn test_millisecond_time_max_milliseconds() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @T10:30:15.999.millisecond() → 999
        let time = NaiveTime::from_hms_milli_opt(10, 30, 15, 999).unwrap();
        let precision_time = PrecisionTime::new(time, TemporalPrecision::Millisecond);
        let input = vec![FhirPathValue::time(precision_time)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(999)));
    }

    #[tokio::test]
    async fn test_millisecond_datetime_with_milliseconds() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @2024-01-15T10:30:15.250.millisecond() → 250
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_milli_opt(10, 30, 15, 250).unwrap();
        let datetime = date.and_time(time);
        let precision_datetime =
            PrecisionDateTime::new(datetime.and_utc().into(), TemporalPrecision::Millisecond);
        let input = vec![FhirPathValue::datetime(precision_datetime)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(250)));
    }

    #[tokio::test]
    async fn test_millisecond_datetime_one_millisecond() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @2024-01-15T10:30:15.001.millisecond() → 1
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_milli_opt(10, 30, 15, 1).unwrap();
        let datetime = date.and_time(time);
        let precision_datetime =
            PrecisionDateTime::new(datetime.and_utc().into(), TemporalPrecision::Millisecond);
        let input = vec![FhirPathValue::datetime(precision_datetime)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(1)));
    }

    #[tokio::test]
    async fn test_millisecond_time_without_millisecond_precision() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @T10:30:15.millisecond() → {} (second precision, not millisecond)
        let time = NaiveTime::from_hms_opt(10, 30, 15).unwrap();
        let precision_time = PrecisionTime::new(time, TemporalPrecision::Second);
        let input = vec![FhirPathValue::time(precision_time)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 0); // Empty collection
    }

    #[tokio::test]
    async fn test_millisecond_time_minute_precision() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @T10:30.millisecond() → {} (minute precision)
        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let precision_time = PrecisionTime::new(time, TemporalPrecision::Minute);
        let input = vec![FhirPathValue::time(precision_time)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 0); // Empty collection
    }

    #[tokio::test]
    async fn test_millisecond_datetime_second_precision() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @2024-01-15T10:30:00.millisecond() → {} (second precision)
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let datetime = date.and_time(time);
        let precision_datetime =
            PrecisionDateTime::new(datetime.and_utc().into(), TemporalPrecision::Second);
        let input = vec![FhirPathValue::datetime(precision_datetime)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 0); // Empty collection
    }

    #[tokio::test]
    async fn test_millisecond_datetime_day_precision() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // @2024-01-15.millisecond() → {} (day precision)
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let datetime = date.and_time(time);
        let precision_datetime =
            PrecisionDateTime::new(datetime.and_utc().into(), TemporalPrecision::Day);
        let input = vec![FhirPathValue::datetime(precision_datetime)];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 0); // Empty collection
    }

    #[tokio::test]
    async fn test_millisecond_empty_input() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // {}.millisecond() → {}
        let input = vec![];

        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 0); // Empty collection
    }

    #[tokio::test]
    async fn test_millisecond_invalid_type_integer() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // 123.millisecond() → error
        let input = vec![FhirPathValue::integer(123)];

        let result = evaluator.evaluate(input, vec![]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only be applied to DateTime or Time")
        );
    }

    #[tokio::test]
    async fn test_millisecond_invalid_type_string() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // 'string'.millisecond() → error
        let input = vec![FhirPathValue::string("string".to_string())];

        let result = evaluator.evaluate(input, vec![]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only be applied to DateTime or Time")
        );
    }

    #[tokio::test]
    async fn test_millisecond_multiple_values_error() {
        let evaluator = MillisecondFunctionEvaluator::create();

        // Multiple values should error (not singleton)
        let time1 = NaiveTime::from_hms_milli_opt(10, 30, 15, 250).unwrap();
        let precision_time1 = PrecisionTime::new(time1, TemporalPrecision::Millisecond);
        let time2 = NaiveTime::from_hms_milli_opt(11, 45, 30, 500).unwrap();
        let precision_time2 = PrecisionTime::new(time2, TemporalPrecision::Millisecond);
        let input = vec![
            FhirPathValue::time(precision_time1),
            FhirPathValue::time(precision_time2),
        ];

        let result = evaluator.evaluate(input, vec![]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only be called on a single")
        );
    }

    #[tokio::test]
    async fn test_millisecond_with_arguments_error() {
        let evaluator = MillisecondFunctionEvaluator::create();

        let time = NaiveTime::from_hms_milli_opt(10, 30, 15, 250).unwrap();
        let precision_time = PrecisionTime::new(time, TemporalPrecision::Millisecond);
        let input = vec![FhirPathValue::time(precision_time)];

        // millisecond() takes no arguments
        let args = vec![vec![FhirPathValue::integer(1)]];

        let result = evaluator.evaluate(input, args).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("takes no arguments")
        );
    }
}
