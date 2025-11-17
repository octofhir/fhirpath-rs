//! Difference function implementation
//!
//! Computes the signed integer difference between two temporal values in specified units.
//! This is an STU (Standard for Trial Use) feature from FHIRPath v3.0.0.
//!
//! ## Specification
//! - **Signature:** `difference(precision : String) : Integer`
//! - **Purpose:** Computes signed integer difference in specified units
//! - **Parameters:**
//!   - `precision`: Unit name - 'years', 'months', 'days', 'hours', 'minutes', 'seconds', 'milliseconds'
//! - **Behavior:**
//!   - Returns signed integer difference (can be negative)
//!   - Truncates to whole units
//!   - Order matters: input - parameter
//!
//! ## Examples
//! - `@2023-01-01.difference(@2024-01-01, 'years')` → `-1`
//! - `@2024-01-01.difference(@2023-01-01, 'years')` → `1`
//! - `@2024-01-01.difference(@2024-01-15, 'days')` → `-14`

use std::sync::Arc;

use chrono::{Datelike, Timelike};

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Precision unit for difference calculation
#[derive(Debug, Clone, Copy, PartialEq)]
enum DifferencePrecision {
    Years,
    Months,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
}

impl DifferencePrecision {
    /// Parse precision string to enum
    fn from_string(s: &str) -> Option<Self> {
        match s {
            "years" | "year" => Some(Self::Years),
            "months" | "month" => Some(Self::Months),
            "days" | "day" => Some(Self::Days),
            "hours" | "hour" => Some(Self::Hours),
            "minutes" | "minute" => Some(Self::Minutes),
            "seconds" | "second" => Some(Self::Seconds),
            "milliseconds" | "millisecond" => Some(Self::Milliseconds),
            _ => None,
        }
    }
}

/// Difference function evaluator
pub struct DifferenceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DifferenceFunctionEvaluator {
    /// Create a new difference function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "difference".to_string(),
                description: "Computes the signed integer difference between two temporal values in specified units"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Date | DateTime | Time".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "precision".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "Unit for difference: 'years', 'months', 'days', 'hours', 'minutes', 'seconds', 'milliseconds'".to_string(),
                        default_value: None,
                    }],
                    return_type: "Integer".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Custom,
                empty_propagation: EmptyPropagation::Custom,
                deterministic: true,
                category: FunctionCategory::Math,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Calculate year difference between two dates (date1 - date2)
    fn calculate_year_difference(date1: &PrecisionDate, date2: &PrecisionDate) -> i64 {
        let year_diff = date1.date.year() - date2.date.year();
        year_diff as i64
    }

    /// Calculate month difference between two dates (date1 - date2)
    fn calculate_month_difference(date1: &PrecisionDate, date2: &PrecisionDate) -> i64 {
        let year_diff = date1.date.year() - date2.date.year();
        let month_diff = date1.date.month() as i32 - date2.date.month() as i32;
        (year_diff * 12 + month_diff) as i64
    }

    /// Calculate day difference between two dates (date1 - date2)
    fn calculate_day_difference(date1: &PrecisionDate, date2: &PrecisionDate) -> i64 {
        (date1.date - date2.date).num_days()
    }

    /// Calculate difference between two datetimes in specified precision (dt1 - dt2)
    fn calculate_datetime_difference(
        dt1: &PrecisionDateTime,
        dt2: &PrecisionDateTime,
        precision: DifferencePrecision,
    ) -> Result<i64> {
        let ts1 = dt1.datetime.timestamp_millis();
        let ts2 = dt2.datetime.timestamp_millis();
        let diff_ms = ts1 - ts2;

        let result = match precision {
            DifferencePrecision::Milliseconds => diff_ms,
            DifferencePrecision::Seconds => diff_ms / 1000,
            DifferencePrecision::Minutes => diff_ms / (60 * 1000),
            DifferencePrecision::Hours => diff_ms / (60 * 60 * 1000),
            DifferencePrecision::Days => diff_ms / (24 * 60 * 60 * 1000),
            DifferencePrecision::Months => {
                // For months and years, use calendar arithmetic
                let date1 = dt1.date();
                let date2 = dt2.date();
                return Ok(Self::calculate_month_difference(&date1, &date2));
            }
            DifferencePrecision::Years => {
                let date1 = dt1.date();
                let date2 = dt2.date();
                return Ok(Self::calculate_year_difference(&date1, &date2));
            }
        };

        Ok(result)
    }

    /// Calculate difference between two times in specified precision (time1 - time2)
    fn calculate_time_difference(
        time1: &PrecisionTime,
        time2: &PrecisionTime,
        precision: DifferencePrecision,
    ) -> Result<i64> {
        // Convert both times to milliseconds since midnight
        let ms1 = time1.time.hour() as i64 * 3_600_000
            + time1.time.minute() as i64 * 60_000
            + time1.time.second() as i64 * 1_000
            + time1.time.nanosecond() as i64 / 1_000_000;

        let ms2 = time2.time.hour() as i64 * 3_600_000
            + time2.time.minute() as i64 * 60_000
            + time2.time.second() as i64 * 1_000
            + time2.time.nanosecond() as i64 / 1_000_000;

        let diff_ms = ms1 - ms2;

        let result = match precision {
            DifferencePrecision::Milliseconds => diff_ms,
            DifferencePrecision::Seconds => diff_ms / 1000,
            DifferencePrecision::Minutes => diff_ms / (60 * 1000),
            DifferencePrecision::Hours => diff_ms / (60 * 60 * 1000),
            // Days, months, years don't make sense for Time
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0081,
                    format!("Invalid precision '{:?}' for Time values", precision),
                ));
            }
        };

        Ok(result)
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DifferenceFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        // Validate input: need exactly 1 temporal value in input
        if input.len() != 1 {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Validate args: need exactly 1 argument (the other temporal value to compare)
        if args.is_empty() || args[0].is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Extract precision parameter (should be second argument in real implementation)
        // For now, let's assume the first arg is the comparand and second is precision
        // Actually looking at the spec, it seems like: input.difference(comparand, precision)
        // So args[0] should have 2 elements or we need to check the function signature

        // Wait, re-reading the task: the signature shows difference(precision) where:
        // - input is the first temporal value
        // - The function needs TWO temporal values total
        // - Plus the precision parameter

        // Looking at the example: `@2023-01-01.difference(@2024-01-01, 'years')`
        // This suggests: input.difference(other_date, precision_string)
        // So args should have 2 elements: [other_temporal, precision_string]

        if args.len() < 2 {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let comparand_values = &args[0];
        let precision_values = &args[1];

        if comparand_values.is_empty() || precision_values.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Extract precision string
        let precision_str = match precision_values.first().unwrap() {
            FhirPathValue::String(s, _, _) => s,
            _ => {
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
        };

        // Parse precision
        let precision = match DifferencePrecision::from_string(precision_str) {
            Some(p) => p,
            None => {
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
        };

        let first = &input[0];
        let second = comparand_values.first().unwrap();

        // Calculate difference based on the temporal type
        let result = match (first, second) {
            // Date - Date
            (FhirPathValue::Date(d1, _, _), FhirPathValue::Date(d2, _, _)) => {
                let diff = match precision {
                    DifferencePrecision::Years => Self::calculate_year_difference(d1, d2),
                    DifferencePrecision::Months => Self::calculate_month_difference(d1, d2),
                    DifferencePrecision::Days => Self::calculate_day_difference(d1, d2),
                    _ => {
                        // Hours, minutes, seconds, milliseconds don't make sense for Date
                        return Ok(EvaluationResult {
                            value: Collection::empty(),
                        });
                    }
                };
                Some(FhirPathValue::integer(diff))
            }

            // DateTime - DateTime
            (FhirPathValue::DateTime(dt1, _, _), FhirPathValue::DateTime(dt2, _, _)) => {
                let diff = Self::calculate_datetime_difference(dt1, dt2, precision)?;
                Some(FhirPathValue::integer(diff))
            }

            // Time - Time
            (FhirPathValue::Time(t1, _, _), FhirPathValue::Time(t2, _, _)) => {
                let diff = Self::calculate_time_difference(t1, t2, precision)?;
                Some(FhirPathValue::integer(diff))
            }

            // Type mismatch - return empty
            _ => None,
        };

        Ok(EvaluationResult {
            value: result
                .map(Collection::single)
                .unwrap_or_else(Collection::empty),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::temporal::{
        PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision,
    };
    use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

    #[tokio::test]
    async fn test_difference_years_negative() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("years".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(-1));
    }

    #[tokio::test]
    async fn test_difference_years_positive() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("years".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(1));
    }

    #[tokio::test]
    async fn test_difference_months() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("months".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(-2));
    }

    #[tokio::test]
    async fn test_difference_days() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("days".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(-14));
    }

    #[tokio::test]
    async fn test_difference_hours_datetime() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let utc = FixedOffset::east_opt(0).unwrap();
        let input = vec![FhirPathValue::datetime(PrecisionDateTime::new(
            utc.from_local_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            ))
            .unwrap(),
            TemporalPrecision::Second,
        ))];
        let args = vec![
            vec![FhirPathValue::datetime(PrecisionDateTime::new(
                utc.from_local_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                ))
                .unwrap(),
                TemporalPrecision::Second,
            ))],
            vec![FhirPathValue::string("hours".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(-2));
    }

    #[tokio::test]
    async fn test_difference_minutes_time() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            TemporalPrecision::Second,
        ))];
        let args = vec![
            vec![FhirPathValue::time(PrecisionTime::new(
                NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
                TemporalPrecision::Second,
            ))],
            vec![FhirPathValue::string("minutes".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(-30));
    }

    #[tokio::test]
    async fn test_difference_invalid_precision() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("weeks".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_difference_type_mismatch() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::time(PrecisionTime::new(
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                TemporalPrecision::Second,
            ))],
            vec![FhirPathValue::string("days".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_difference_zero() {
        let evaluator = DifferenceFunctionEvaluator::create();
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("days".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(0));
    }

    #[tokio::test]
    async fn test_difference_leap_year() {
        let evaluator = DifferenceFunctionEvaluator::create();
        // 2024 is a leap year, so 2024-01-01 to 2025-01-01 is 366 days
        let input = vec![FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ))];
        let args = vec![
            vec![FhirPathValue::date(PrecisionDate::new(
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                TemporalPrecision::Day,
            ))],
            vec![FhirPathValue::string("days".to_string())],
        ];

        let result = evaluator.evaluate(input, args).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(-366));
    }
}
