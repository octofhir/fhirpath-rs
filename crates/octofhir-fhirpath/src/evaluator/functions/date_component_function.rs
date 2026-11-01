//! Date/time component extraction (FHIRPath §1.5.10.8): `yearOf()`, `monthOf()`, `dayOf()`,
//! `hourOf()`, `minuteOf()`, `secondOf()`, `millisecondOf()` and `timezoneOffsetOf()`.
//!
//! Each returns the component when the value has it: a component beyond the value's
//! precision (`@2012.monthOf()`), a time component of a Date, a date component of a Time,
//! or a value that is not a Date, DateTime or Time gives empty. `timezoneOffsetOf()` is the
//! offset in hours as a Decimal (`-07:00` is `-7.0`, `+08:45` is `8.75`), empty without an
//! explicit timezone.

use std::sync::Arc;

use chrono::{Datelike, Timelike};
use rust_decimal::Decimal;

use crate::core::temporal::TemporalPrecision;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// The component a function extracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateComponent {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    TimezoneOffset,
}

impl DateComponent {
    /// The precision a value needs to have this component.
    fn precision(self) -> TemporalPrecision {
        match self {
            Self::Year => TemporalPrecision::Year,
            Self::Month => TemporalPrecision::Month,
            Self::Day => TemporalPrecision::Day,
            Self::Hour | Self::TimezoneOffset => TemporalPrecision::Hour,
            Self::Minute => TemporalPrecision::Minute,
            Self::Second => TemporalPrecision::Second,
            Self::Millisecond => TemporalPrecision::Millisecond,
        }
    }

    fn is_date_part(self) -> bool {
        matches!(self, Self::Year | Self::Month | Self::Day)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Year => "yearOf",
            Self::Month => "monthOf",
            Self::Day => "dayOf",
            Self::Hour => "hourOf",
            Self::Minute => "minuteOf",
            Self::Second => "secondOf",
            Self::Millisecond => "millisecondOf",
            Self::TimezoneOffset => "timezoneOffsetOf",
        }
    }

    /// The component of one value, if it has it.
    fn of(self, value: &FhirPathValue) -> Option<FhirPathValue> {
        let integer = |v: u32| Some(FhirPathValue::integer(i64::from(v)));
        match value {
            FhirPathValue::Date(date, _, _) => {
                if !self.is_date_part() || date.precision < self.precision() {
                    return None;
                }
                match self {
                    Self::Year => Some(FhirPathValue::integer(i64::from(date.date.year()))),
                    Self::Month => integer(date.date.month()),
                    _ => integer(date.date.day()),
                }
            }
            FhirPathValue::DateTime(dt, _, _) => {
                if self == Self::TimezoneOffset {
                    if !dt.tz_specified {
                        return None;
                    }
                    let seconds = dt.datetime.offset().local_minus_utc();
                    let mut hours = (Decimal::from(seconds) / Decimal::from(3600)).normalize();
                    if hours.scale() == 0 {
                        hours.rescale(1);
                    }
                    return Some(FhirPathValue::decimal(hours));
                }
                if dt.precision < self.precision() {
                    return None;
                }
                let at = dt.datetime;
                match self {
                    Self::Year => Some(FhirPathValue::integer(i64::from(at.year()))),
                    Self::Month => integer(at.month()),
                    Self::Day => integer(at.day()),
                    Self::Hour => integer(at.hour()),
                    Self::Minute => integer(at.minute()),
                    Self::Second => integer(at.second()),
                    _ => integer(at.nanosecond() / 1_000_000),
                }
            }
            FhirPathValue::Time(time, _, _) => {
                if self.is_date_part()
                    || self == Self::TimezoneOffset
                    || time.precision < self.precision()
                {
                    return None;
                }
                let at = time.time;
                match self {
                    Self::Hour => integer(at.hour()),
                    Self::Minute => integer(at.minute()),
                    Self::Second => integer(at.second()),
                    _ => integer(at.nanosecond() / 1_000_000),
                }
            }
            _ => None,
        }
    }
}

/// One of the component extraction functions.
pub struct DateComponentFunctionEvaluator {
    component: DateComponent,
    metadata: FunctionMetadata,
}

impl DateComponentFunctionEvaluator {
    /// The function extracting `component`, registered under its FHIRPath name.
    pub fn create(component: DateComponent) -> Arc<dyn PureFunctionEvaluator> {
        Self::create_named(component, component.name())
    }

    /// The function extracting `component` under another name (e.g. the legacy
    /// `millisecond()`).
    pub fn create_named(component: DateComponent, name: &str) -> Arc<dyn PureFunctionEvaluator> {
        let (input_type, return_type) = match component {
            DateComponent::Year | DateComponent::Month | DateComponent::Day => {
                ("Date | DateTime", "Integer")
            }
            DateComponent::TimezoneOffset => ("DateTime", "Decimal"),
            _ => ("DateTime | Time", "Integer"),
        };
        Arc::new(Self {
            component,
            metadata: FunctionMetadata {
                name: name.to_string(),
                description: format!("The {component:?} component of a date/time value"),
                signature: FunctionSignature {
                    input_type: input_type.to_string(),
                    parameters: vec![],
                    return_type: return_type.to_string(),
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
impl PureFunctionEvaluator for DateComponentFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("{} takes no arguments", self.metadata.name),
            ));
        }
        let value = match input.len() {
            0 => None,
            1 => self.component.of(&input[0]),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    format!("{} can only be called on a single item", self.metadata.name),
                ));
            }
        };
        Ok(EvaluationResult {
            value: value
                .map(Collection::single)
                .unwrap_or_else(Collection::empty),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
