//! Difference function implementation
//!
//! Counts the calendar boundaries crossed between two temporal values.
//! This is an STU (Standard for Trial Use) feature of FHIRPath 3.1 CI.
//!
//! ## Specification
//! - **Signature:** `difference(value : Date | DateTime | Time, precision : String) : Integer`
//! - **Behavior:**
//!   - Returns the number of `precision` boundaries crossed going from the input to `value`;
//!     negative when the input is after `value`
//!   - Weeks start on Sunday
//!   - Date: `year`, `month`, `week`, `day`; Time: `hour` .. `millisecond`;
//!     DateTime: all of them. Plural forms are accepted.
//!   - Empty when either operand is empty or has less precision than requested
//!   - DateTime values are normalized to UTC for `hour` and finer precisions
//!
//! ## Examples
//! - `@2025-01-02.difference(@2025-01-07, 'week')` → `1`
//! - `@2025-01-01.difference(@2025-09-01, 'year')` → `0`
//! - `@2024-12-01.difference(@2025-09-01, 'year')` → `1`

use std::sync::Arc;

use super::duration_function::{Mode, evaluate_interval, interval_metadata};
use crate::core::{Collection, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{FunctionMetadata, PureFunctionEvaluator};

/// Difference function evaluator
pub struct DifferenceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DifferenceFunctionEvaluator {
    /// Create a new difference function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: interval_metadata(
                "difference",
                "Returns the number of calendar boundaries crossed between two temporal values",
            ),
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DifferenceFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        evaluate_interval(Mode::Difference, "difference", &input, &args)
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::FhirPathValue;
    use crate::core::temporal::{PrecisionDate, TemporalPrecision};
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> FhirPathValue {
        FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(y, m, d).unwrap(),
            TemporalPrecision::Day,
        ))
    }

    fn diff(a: FhirPathValue, b: FhirPathValue, unit: &str) -> Option<i64> {
        let r = DifferenceFunctionEvaluator::create()
            .evaluate_sync(
                Collection::single(a),
                vec![
                    Collection::single(b),
                    Collection::single(FhirPathValue::string(unit)),
                ],
            )
            .unwrap();
        match r.value.first() {
            Some(FhirPathValue::Integer(i, _, _)) => Some(*i),
            _ => None,
        }
    }

    #[test]
    fn spec_examples() {
        assert_eq!(diff(date(2025, 1, 2), date(2025, 1, 7), "week"), Some(1));
        assert_eq!(diff(date(2025, 1, 1), date(2025, 9, 1), "year"), Some(0));
        assert_eq!(diff(date(2024, 12, 1), date(2025, 9, 1), "year"), Some(1));
    }

    #[test]
    fn sign_and_boundaries() {
        assert_eq!(diff(date(2025, 1, 1), date(2025, 9, 1), "month"), Some(8));
        assert_eq!(diff(date(2025, 9, 1), date(2025, 1, 1), "months"), Some(-8));
        assert_eq!(diff(date(2025, 1, 31), date(2025, 2, 1), "month"), Some(1));
        assert_eq!(diff(date(2025, 1, 1), date(2025, 1, 15), "day"), Some(14));
        // Saturday to Sunday crosses a week boundary
        assert_eq!(diff(date(2025, 1, 4), date(2025, 1, 5), "week"), Some(1));
        assert_eq!(diff(date(2025, 1, 5), date(2025, 1, 11), "week"), Some(0));
    }
}
