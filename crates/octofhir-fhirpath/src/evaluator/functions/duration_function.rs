//! Duration function implementation
//!
//! Calculates the absolute duration between two temporal values.
//! This is an STU (Standard for Trial Use) feature from FHIRPath v3.0.0.
//!
//! ## Specification
//! - **Signature:** `duration() : Quantity`
//! - **Purpose:** Calculates duration/span between two temporal values
//! - **Behavior:**
//!   - Operates on collection with exactly two temporal values
//!   - Returns absolute duration (always positive) in appropriate time units
//!   - Handles partial dates with precision boundaries
//!   - Result unit depends on precision (days, milliseconds, etc.)
//!
//! ## Examples
//! - `{ @2024-01-01, @2024-01-15 }.duration()` → `14 'day'`
//! - `{ @2024-01-15, @2024-01-01 }.duration()` → `14 'day'` (absolute value)
//! - `{ @2024-01-01T00:00:00, @2024-01-01T01:00:00 }.duration()` → `3600000 'ms'`
//! - `{ @T10:00:00, @T10:30:00 }.duration()` → `1800000 'ms'`

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Duration function evaluator
pub struct DurationFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DurationFunctionEvaluator {
    /// Create a new duration function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "duration".to_string(),
                description: "Calculates the absolute duration between two temporal values"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Quantity".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
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

    /// Calculate duration between two dates in days
    fn calculate_date_duration(date1: &PrecisionDate, date2: &PrecisionDate) -> Result<Decimal> {
        // Calculate the number of days between the two dates
        let days = (date2.date - date1.date).num_days();
        Ok(Decimal::from(days.abs()))
    }

    /// Calculate duration between two datetimes in milliseconds
    fn calculate_datetime_duration(
        dt1: &PrecisionDateTime,
        dt2: &PrecisionDateTime,
    ) -> Result<Decimal> {
        // Both datetimes are stored as DateTime<FixedOffset>
        // Get timestamp in milliseconds
        let ts1 = dt1.datetime.timestamp_millis();
        let ts2 = dt2.datetime.timestamp_millis();

        let diff_ms = (ts2 - ts1).abs();
        Ok(Decimal::from(diff_ms))
    }

    /// Calculate duration between two times in milliseconds
    fn calculate_time_duration(time1: &PrecisionTime, time2: &PrecisionTime) -> Result<Decimal> {
        use chrono::Timelike;

        // Convert both times to milliseconds since midnight
        let ms1 = time1.time.hour() as i64 * 3_600_000
            + time1.time.minute() as i64 * 60_000
            + time1.time.second() as i64 * 1_000
            + time1.time.nanosecond() as i64 / 1_000_000;

        let ms2 = time2.time.hour() as i64 * 3_600_000
            + time2.time.minute() as i64 * 60_000
            + time2.time.second() as i64 * 1_000
            + time2.time.nanosecond() as i64 / 1_000_000;

        let diff_ms = (ms2 - ms1).abs();
        Ok(Decimal::from(diff_ms))
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DurationFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        // Empty collection or single item returns empty
        if input.len() != 2 {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let first = &input[0];
        let second = &input[1];

        // Calculate duration based on the temporal type
        let result = match (first, second) {
            // Date - Date → Duration in days
            (FhirPathValue::Date(d1, _, _), FhirPathValue::Date(d2, _, _)) => {
                let days = Self::calculate_date_duration(d1, d2)?;
                Some(FhirPathValue::quantity(days, Some("day".to_string())))
            }

            // DateTime - DateTime → Duration in milliseconds
            (FhirPathValue::DateTime(dt1, _, _), FhirPathValue::DateTime(dt2, _, _)) => {
                let ms = Self::calculate_datetime_duration(dt1, dt2)?;
                Some(FhirPathValue::quantity(ms, Some("ms".to_string())))
            }

            // Time - Time → Duration in milliseconds
            (FhirPathValue::Time(t1, _, _), FhirPathValue::Time(t2, _, _)) => {
                let ms = Self::calculate_time_duration(t1, t2)?;
                Some(FhirPathValue::quantity(ms, Some("ms".to_string())))
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
    async fn test_duration_empty_collection() {
        let evaluator = DurationFunctionEvaluator::create();
        let result = evaluator.evaluate(vec![], vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_duration_single_item() {
        let evaluator = DurationFunctionEvaluator::create();
        let date = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let result = evaluator.evaluate(vec![date], vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_duration_date_same() {
        let evaluator = DurationFunctionEvaluator::create();
        let date1 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let date2 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let result = evaluator
            .evaluate(vec![date1, date2], vec![])
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(0));
            assert_eq!(unit.as_deref(), Some("day"));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_duration_date_forward() {
        let evaluator = DurationFunctionEvaluator::create();
        let date1 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let date2 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            TemporalPrecision::Day,
        ));
        let result = evaluator
            .evaluate(vec![date1, date2], vec![])
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(14));
            assert_eq!(unit.as_deref(), Some("day"));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_duration_date_backward() {
        let evaluator = DurationFunctionEvaluator::create();
        let date1 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            TemporalPrecision::Day,
        ));
        let date2 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let result = evaluator
            .evaluate(vec![date1, date2], vec![])
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(14)); // Absolute value
            assert_eq!(unit.as_deref(), Some("day"));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_duration_datetime_one_hour() {
        let evaluator = DurationFunctionEvaluator::create();
        let utc = FixedOffset::east_opt(0).unwrap();
        let dt1 = FhirPathValue::datetime(PrecisionDateTime::new(
            utc.from_local_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ))
            .unwrap(),
            TemporalPrecision::Second,
        ));
        let dt2 = FhirPathValue::datetime(PrecisionDateTime::new(
            utc.from_local_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
            ))
            .unwrap(),
            TemporalPrecision::Second,
        ));
        let result = evaluator.evaluate(vec![dt1, dt2], vec![]).await.unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(3_600_000)); // 1 hour in ms
            assert_eq!(unit.as_deref(), Some("ms"));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_duration_time_half_hour() {
        let evaluator = DurationFunctionEvaluator::create();
        let time1 = FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            TemporalPrecision::Second,
        ));
        let time2 = FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
            TemporalPrecision::Second,
        ));
        let result = evaluator
            .evaluate(vec![time1, time2], vec![])
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(1_800_000)); // 30 minutes in ms
            assert_eq!(unit.as_deref(), Some("ms"));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_duration_time_backward() {
        let evaluator = DurationFunctionEvaluator::create();
        let time1 = FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
            TemporalPrecision::Second,
        ));
        let time2 = FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            TemporalPrecision::Second,
        ));
        let result = evaluator
            .evaluate(vec![time1, time2], vec![])
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(1_800_000)); // Absolute value
            assert_eq!(unit.as_deref(), Some("ms"));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_duration_type_mismatch() {
        let evaluator = DurationFunctionEvaluator::create();
        let date = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let time = FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            TemporalPrecision::Second,
        ));
        let result = evaluator.evaluate(vec![date, time], vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_duration_large_date_span() {
        let evaluator = DurationFunctionEvaluator::create();
        let date1 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let date2 = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            TemporalPrecision::Day,
        ));
        let result = evaluator
            .evaluate(vec![date1, date2], vec![])
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            // 4 years (2020-2024) = 1461 days (including leap year 2020)
            assert_eq!(*value, Decimal::from(1461));
            assert_eq!(unit.as_deref(), Some("day"));
        } else {
            panic!("Expected Quantity result");
        }
    }
}
