//! dateOf() and timeOf() (FHIRPath §1.5.10.8.9, §1.5.10.8.10)
//!
//! `dateOf()` returns the date component of a Date or DateTime, up to the precision present;
//! `timeOf()` returns the time component of a DateTime that has one. Any other value, or a
//! DateTime without a time, gives empty.

use std::sync::Arc;

use crate::core::temporal::{PrecisionDate, PrecisionTime, TemporalPrecision};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

fn metadata(name: &str, description: &str, return_type: &str) -> FunctionMetadata {
    FunctionMetadata {
        name: name.to_string(),
        description: description.to_string(),
        signature: FunctionSignature {
            input_type: "Date | DateTime".to_string(),
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
    }
}

fn component(
    name: &str,
    input: &Collection,
    extract: impl Fn(&FhirPathValue) -> Option<FhirPathValue>,
) -> Result<EvaluationResult> {
    let value = match input.len() {
        0 => None,
        1 => input.first().and_then(extract),
        _ => {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("{name} can only be called on a single item"),
            ));
        }
    };
    Ok(EvaluationResult {
        value: value
            .map(Collection::single)
            .unwrap_or_else(Collection::empty),
    })
}

/// The date of a Date or DateTime, at the input's precision (at most day).
pub(crate) fn date_of(value: &FhirPathValue) -> Option<FhirPathValue> {
    match value {
        FhirPathValue::Date(date, _, _) => Some(FhirPathValue::date(date.clone())),
        FhirPathValue::DateTime(dt, _, _) => {
            let precision = match dt.precision {
                TemporalPrecision::Year => TemporalPrecision::Year,
                TemporalPrecision::Month => TemporalPrecision::Month,
                _ => TemporalPrecision::Day,
            };
            Some(FhirPathValue::date(PrecisionDate::new(
                dt.datetime.date_naive(),
                precision,
            )))
        }
        _ => None,
    }
}

/// The time of a DateTime that has one, at the input's precision.
pub(crate) fn time_of(value: &FhirPathValue) -> Option<FhirPathValue> {
    match value {
        FhirPathValue::DateTime(dt, _, _)
            if !matches!(
                dt.precision,
                TemporalPrecision::Year | TemporalPrecision::Month | TemporalPrecision::Day
            ) =>
        {
            Some(FhirPathValue::time(PrecisionTime::new(
                dt.datetime.time(),
                dt.precision,
            )))
        }
        _ => None,
    }
}

/// dateOf() function evaluator
pub struct DateOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DateOfFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: metadata("dateOf", "The date component of a Date or DateTime", "Date"),
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for DateOfFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, _args: Vec<Collection>) -> Result<EvaluationResult> {
        component("dateOf", &input, date_of)
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// timeOf() function evaluator
pub struct TimeOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TimeOfFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: metadata("timeOf", "The time component of a DateTime", "Time"),
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for TimeOfFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, _args: Vec<Collection>) -> Result<EvaluationResult> {
        component("timeOf", &input, time_of)
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
