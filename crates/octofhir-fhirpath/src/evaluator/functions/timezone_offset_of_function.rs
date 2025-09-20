//! TimezoneOffsetOf function implementation
//!
//! The timezoneOffsetOf function returns the timezone offset in minutes from UTC for a datetime value.
//! Syntax: datetime.timezoneOffsetOf()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// TimezoneOffsetOf function evaluator
pub struct TimezoneOffsetOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TimezoneOffsetOfFunctionEvaluator {
    /// Create a new timezoneOffsetOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "timezoneOffsetOf".to_string(),
                description:
                    "Returns the timezone offset in minutes from UTC for a datetime value."
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "DateTime".to_string(),
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
impl PureFunctionEvaluator for TimezoneOffsetOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "timezoneOffsetOf function takes no arguments".to_string(),
            ));
        }

        // Must be exactly one datetime value
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "timezoneOffsetOf function can only be called on a single datetime value"
                    .to_string(),
            ));
        }

        let result = match &input[0] {
            FhirPathValue::DateTime(precision_datetime, _, _) => {
                // Get the timezone offset from the DateTime<FixedOffset>
                let offset_seconds = precision_datetime.datetime.offset().local_minus_utc();
                // Convert seconds to minutes
                let offset_minutes = offset_seconds / 60;

                Some(FhirPathValue::integer(offset_minutes as i64))
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "timezoneOffsetOf function can only be called on datetime values".to_string(),
                ));
            }
        };

        Ok(EvaluationResult {
            value: match result {
                Some(offset_value) => crate::core::Collection::from(vec![offset_value]),
                None => crate::core::Collection::empty(),
            },
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
