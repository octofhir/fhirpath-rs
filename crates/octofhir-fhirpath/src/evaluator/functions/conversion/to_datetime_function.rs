//! ToDateTime function implementation
//!
//! The toDateTime function converts a string or date to a datetime.
//! Syntax: value.toDateTime()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::EvaluationResult;

/// ToDateTime function evaluator
pub struct ToDateTimeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToDateTimeFunctionEvaluator {
    /// Create a new toDateTime function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toDateTime".to_string(),
                description: "Converts a value to a datetime".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "DateTime".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Conversion,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ToDateTimeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toDateTime function takes no arguments".to_string(),
            ));
        }

        if input.len() != 1 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let result = match &input[0] {
            FhirPathValue::DateTime(precision_datetime, _, _) => {
                // DateTime is already a datetime, return as-is
                Some(input[0].clone())
            }
            FhirPathValue::Date(precision_date, _, _) => {
                // Convert Date to DateTime by adding midnight time (assume no timezone, so use UTC)
                use crate::core::temporal::PrecisionDateTime;
                use chrono::{DateTime, FixedOffset, NaiveTime};

                // Create midnight time
                let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                let naive_datetime = precision_date.date.and_time(midnight);

                // Convert to DateTime<FixedOffset> assuming UTC (no timezone specified)
                let utc_offset = FixedOffset::east_opt(0).unwrap(); // UTC
                let datetime = DateTime::from_naive_utc_and_offset(naive_datetime, utc_offset);

                // Create PrecisionDateTime with the same precision as the date, but mark timezone as not specified
                let precision_datetime =
                    PrecisionDateTime::new_with_tz(datetime, precision_date.precision, false);
                Some(FhirPathValue::datetime(precision_datetime))
            }
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as datetime or date
                use crate::core::temporal::{PrecisionDate, PrecisionDateTime};

                // First try parsing as a DateTime
                if let Some(precision_datetime) = PrecisionDateTime::parse(s) {
                    Some(FhirPathValue::datetime(precision_datetime))
                } else {
                    // Try parsing as a Date and convert to DateTime
                    if let Some(precision_date) = PrecisionDate::parse(s) {
                        use chrono::{DateTime, FixedOffset, NaiveTime};

                        // Create midnight time
                        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                        let naive_datetime = precision_date.date.and_time(midnight);

                        // Convert to DateTime<FixedOffset> assuming UTC (no timezone specified)
                        let utc_offset = FixedOffset::east_opt(0).unwrap(); // UTC
                        let datetime =
                            DateTime::from_naive_utc_and_offset(naive_datetime, utc_offset);

                        // Create PrecisionDateTime with the same precision as the date, but mark timezone as not specified
                        let precision_datetime = PrecisionDateTime::new_with_tz(
                            datetime,
                            precision_date.precision,
                            false,
                        );
                        Some(FhirPathValue::datetime(precision_datetime))
                    } else {
                        // String is not a valid date or datetime
                        None
                    }
                }
            }
            _ => {
                // Other types cannot be converted to datetime
                None
            }
        };

        Ok(EvaluationResult {
            value: match result {
                Some(datetime_value) => crate::core::Collection::from(vec![datetime_value]),
                None => crate::core::Collection::empty(),
            },
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
