//! Duration function implementation
//!
//! Counts the whole calendar periods between two temporal values.
//! This is an STU (Standard for Trial Use) feature of FHIRPath 3.1 CI.
//!
//! ## Specification
//! - **Signature:** `duration(value : Date | DateTime | Time, precision : String) : Integer`
//! - **Behavior:**
//!   - Returns the number of whole periods of `precision` from the input to `value`;
//!     fractions are dropped and the result is negative when the input is after `value`
//!   - Date: `year`, `month`, `week`, `day`; Time: `hour` .. `millisecond`;
//!     DateTime: all of them. Plural forms are accepted.
//!   - Empty when either operand is empty or has less precision than requested
//!   - DateTime values are normalized to UTC for `hour` and finer precisions
//!
//! `difference` (boundaries crossed) shares the calculation in this module.
//!
//! ## Examples
//! - `@2025-01-02.duration(@2025-01-07, 'week')` → `0`
//! - `@2025-01-01.duration(@2025-09-01, 'year')` → `0`
//! - `@2024-12-01.duration(@2025-09-01, 'month')` → `9`

use std::sync::Arc;

use chrono::{Datelike, NaiveDateTime, Timelike};

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Which flavour of interval calculation to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    /// Whole elapsed calendar periods.
    Duration,
    /// Number of calendar boundaries crossed.
    Difference,
}

/// Calendar precision accepted by `duration` / `difference`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
}

impl Unit {
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "year" | "years" => Self::Year,
            "month" | "months" => Self::Month,
            "week" | "weeks" => Self::Week,
            "day" | "days" => Self::Day,
            "hour" | "hours" => Self::Hour,
            "minute" | "minutes" => Self::Minute,
            "second" | "seconds" => Self::Second,
            "millisecond" | "milliseconds" => Self::Millisecond,
            _ => return None,
        })
    }

    /// Minimum precision an operand must carry for this unit to be computable.
    fn required_precision(self) -> TemporalPrecision {
        match self {
            Self::Year => TemporalPrecision::Year,
            Self::Month => TemporalPrecision::Month,
            Self::Week | Self::Day => TemporalPrecision::Day,
            Self::Hour => TemporalPrecision::Hour,
            Self::Minute => TemporalPrecision::Minute,
            Self::Second => TemporalPrecision::Second,
            Self::Millisecond => TemporalPrecision::Millisecond,
        }
    }

    /// Fixed length in milliseconds (everything except year and month).
    fn millis(self) -> i64 {
        match self {
            Self::Year | Self::Month => 1,
            Self::Week => 7 * 86_400_000,
            Self::Day => 86_400_000,
            Self::Hour => 3_600_000,
            Self::Minute => 60_000,
            Self::Second => 1_000,
            Self::Millisecond => 1,
        }
    }

    fn is_time_unit(self) -> bool {
        matches!(
            self,
            Self::Hour | Self::Minute | Self::Second | Self::Millisecond
        )
    }
}

/// One operand reduced to the values the calculation needs.
struct Operand {
    /// Local (wall clock) date and time.
    local: NaiveDateTime,
    /// Milliseconds since the epoch (UTC) or since midnight for times.
    instant_ms: i64,
}

fn time_ms(t: chrono::NaiveTime) -> i64 {
    t.hour() as i64 * 3_600_000
        + t.minute() as i64 * 60_000
        + t.second() as i64 * 1_000
        + t.nanosecond() as i64 / 1_000_000
}

fn invalid(msg: String) -> FhirPathError {
    FhirPathError::evaluation_error(crate::core::error_code::FP0081, msg)
}

/// Calculate the result for two operands.
fn compute(mode: Mode, unit: Unit, a: &Operand, b: &Operand) -> i64 {
    match unit {
        Unit::Year | Unit::Month => {
            let months = |o: &Operand| o.local.year() as i64 * 12 + o.local.month0() as i64;
            match (mode, unit) {
                (Mode::Difference, Unit::Year) => b.local.year() as i64 - a.local.year() as i64,
                (Mode::Difference, _) => months(b) - months(a),
                (Mode::Duration, _) => {
                    let rest = |o: &Operand| (o.local.day(), o.local.time());
                    let mut m = months(b) - months(a);
                    if m > 0 && rest(b) < rest(a) {
                        m -= 1;
                    } else if m < 0 && rest(b) > rest(a) {
                        m += 1;
                    }
                    if unit == Unit::Year { m / 12 } else { m }
                }
            }
        }
        Unit::Week | Unit::Day => match mode {
            Mode::Difference => {
                // num_days_from_ce is 1 for Monday 0001-01-01, so Sundays are multiples of 7.
                let idx = |o: &Operand| {
                    let days = o.local.date().num_days_from_ce() as i64;
                    if unit == Unit::Week {
                        days.div_euclid(7)
                    } else {
                        days
                    }
                };
                idx(b) - idx(a)
            }
            Mode::Duration => (b.local - a.local).num_milliseconds() / unit.millis(),
        },
        _ => {
            let len = unit.millis();
            match mode {
                Mode::Difference => b.instant_ms.div_euclid(len) - a.instant_ms.div_euclid(len),
                Mode::Duration => (b.instant_ms - a.instant_ms) / len,
            }
        }
    }
}

/// Shared implementation of `duration` and `difference`.
pub(super) fn evaluate_interval(
    mode: Mode,
    name: &str,
    input: &Collection,
    args: &[Collection],
) -> Result<EvaluationResult> {
    let empty = || {
        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    };

    if input.len() > 1 {
        return Err(invalid(format!(
            "{name}() requires a single input item, found {}",
            input.len()
        )));
    }
    let (Some(first), Some(other), Some(precision)) = (
        input.first(),
        args.first().and_then(|c| c.first()),
        args.get(1).and_then(|c| c.first()),
    ) else {
        return empty();
    };

    let FhirPathValue::String(precision, _, _) = precision else {
        return Err(invalid(format!("{name}() precision must be a String")));
    };
    let unit = Unit::parse(precision)
        .ok_or_else(|| invalid(format!("{name}(): unsupported precision '{precision}'")))?;
    let required = unit.required_precision();

    let (a, b) = match (first, other) {
        (FhirPathValue::Date(d1, _, _), FhirPathValue::Date(d2, _, _)) => {
            if unit.is_time_unit() {
                return Err(invalid(format!(
                    "{name}(): precision '{precision}' is not valid for Date values"
                )));
            }
            if d1.precision < required || d2.precision < required {
                return empty();
            }
            let op = |d: &PrecisionDate| Operand {
                local: d.date.and_time(chrono::NaiveTime::MIN),
                instant_ms: 0,
            };
            (op(d1), op(d2))
        }
        (FhirPathValue::DateTime(d1, _, _), FhirPathValue::DateTime(d2, _, _)) => {
            if d1.precision < required || d2.precision < required {
                return empty();
            }
            let op = |d: &PrecisionDateTime| Operand {
                local: d.datetime.naive_local(),
                instant_ms: d.datetime.timestamp_millis(),
            };
            (op(d1), op(d2))
        }
        (FhirPathValue::Time(t1, _, _), FhirPathValue::Time(t2, _, _)) => {
            if !unit.is_time_unit() {
                return Err(invalid(format!(
                    "{name}(): precision '{precision}' is not valid for Time values"
                )));
            }
            if t1.precision < required || t2.precision < required {
                return empty();
            }
            let op = |t: &PrecisionTime| Operand {
                local: chrono::NaiveDate::MIN.and_time(t.time),
                instant_ms: time_ms(t.time),
            };
            (op(t1), op(t2))
        }
        _ => {
            return Err(invalid(format!(
                "{name}() requires Date, DateTime or Time values of the same type"
            )));
        }
    };

    Ok(EvaluationResult {
        value: Collection::single(FhirPathValue::integer(compute(mode, unit, &a, &b))),
    })
}

/// Build metadata shared by `duration` and `difference`.
pub(super) fn interval_metadata(name: &str, description: &str) -> FunctionMetadata {
    let param = |name: &str, ty: &str, description: &str| FunctionParameter {
        name: name.to_string(),
        parameter_type: vec![ty.to_string()],
        optional: false,
        is_expression: false,
        description: description.to_string(),
        default_value: None,
    };
    FunctionMetadata {
        name: name.to_string(),
        description: description.to_string(),
        signature: FunctionSignature {
            input_type: "Date | DateTime | Time".to_string(),
            parameters: vec![
                param(
                    "value",
                    "Date | DateTime | Time",
                    "The other temporal value",
                ),
                param(
                    "precision",
                    "String",
                    "year, month, week, day, hour, minute, second or millisecond",
                ),
            ],
            return_type: "Integer".to_string(),
            polymorphic: true,
            min_params: 2,
            max_params: Some(2),
        },
        argument_evaluation: ArgumentEvaluationStrategy::Current,
        null_propagation: NullPropagationStrategy::Custom,
        empty_propagation: EmptyPropagation::Custom,
        deterministic: true,
        category: FunctionCategory::Math,
        requires_terminology: false,
        requires_model: false,
    }
}

/// Duration function evaluator
pub struct DurationFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DurationFunctionEvaluator {
    /// Create a new duration function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: interval_metadata(
                "duration",
                "Returns the number of whole calendar periods between two temporal values",
            ),
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DurationFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        evaluate_interval(Mode::Duration, "duration", &input, &args)
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};

    fn date(y: i32, m: u32, d: u32) -> FhirPathValue {
        FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(y, m, d).unwrap(),
            TemporalPrecision::Day,
        ))
    }

    fn dt(s: &str, precision: TemporalPrecision) -> FhirPathValue {
        let parsed: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(s).unwrap();
        FhirPathValue::datetime(PrecisionDateTime::new(parsed, precision))
    }

    fn time(h: u32, m: u32, s: u32) -> FhirPathValue {
        FhirPathValue::time(PrecisionTime::new(
            NaiveTime::from_hms_opt(h, m, s).unwrap(),
            TemporalPrecision::Second,
        ))
    }

    fn call(input: Collection, other: FhirPathValue, unit: &str) -> Result<Collection> {
        evaluate_interval(
            Mode::Duration,
            "duration",
            &input,
            &[
                Collection::single(other),
                Collection::single(FhirPathValue::string(unit)),
            ],
        )
        .map(|r| r.value)
    }

    fn run(input: FhirPathValue, other: FhirPathValue, unit: &str) -> Result<Collection> {
        call(Collection::single(input), other, unit)
    }

    fn int(v: Result<Collection>) -> Option<i64> {
        match v.unwrap().first() {
            Some(FhirPathValue::Integer(i, _, _)) => Some(*i),
            _ => None,
        }
    }

    #[test]
    fn spec_examples() {
        assert_eq!(
            int(run(date(2025, 1, 2), date(2025, 1, 7), "week")),
            Some(0)
        );
        assert_eq!(
            int(run(date(2025, 1, 1), date(2025, 9, 1), "year")),
            Some(0)
        );
        assert_eq!(
            int(run(date(2024, 12, 1), date(2025, 9, 1), "year")),
            Some(0)
        );
    }

    #[test]
    fn whole_periods_and_sign() {
        assert_eq!(
            int(run(date(2024, 12, 1), date(2025, 9, 1), "months")),
            Some(9)
        );
        assert_eq!(
            int(run(date(2025, 9, 1), date(2024, 12, 1), "month")),
            Some(-9)
        );
        assert_eq!(
            int(run(date(2025, 1, 15), date(2025, 2, 14), "month")),
            Some(0)
        );
        assert_eq!(
            int(run(date(2020, 3, 1), date(2025, 3, 1), "year")),
            Some(5)
        );
        assert_eq!(
            int(run(date(2025, 1, 1), date(2025, 1, 15), "week")),
            Some(2)
        );
        assert_eq!(
            int(run(date(2025, 1, 15), date(2025, 1, 1), "day")),
            Some(-14)
        );
    }

    #[test]
    fn datetime_normalizes_offsets_for_hours() {
        let a = dt("2025-01-01T10:00:00+02:00", TemporalPrecision::Second);
        let b = dt("2025-01-01T10:00:00Z", TemporalPrecision::Second);
        assert_eq!(int(run(a, b, "hour")), Some(2));
    }

    #[test]
    fn time_values() {
        assert_eq!(int(run(time(10, 0, 0), time(11, 30, 0), "hour")), Some(1));
        assert_eq!(
            int(run(time(10, 0, 0), time(11, 30, 0), "minute")),
            Some(90)
        );
    }

    #[test]
    fn low_precision_and_empty_yield_empty() {
        let yearly = FhirPathValue::date(PrecisionDate::new(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            TemporalPrecision::Year,
        ));
        assert_eq!(int(run(yearly, date(2025, 9, 1), "month")), None);
        assert!(
            call(Collection::empty(), date(2025, 1, 1), "day")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn invalid_precision_errors() {
        assert!(run(date(2025, 1, 1), date(2025, 1, 2), "hour").is_err());
        assert!(run(time(1, 0, 0), time(2, 0, 0), "day").is_err());
        assert!(run(date(2025, 1, 1), date(2025, 1, 2), "fortnight").is_err());
    }
}
