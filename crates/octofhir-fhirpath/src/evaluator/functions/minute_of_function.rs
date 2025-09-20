//! MinuteOf function implementation
//!
//! The minuteOf function extracts the minute component from a datetime or time.
//! Syntax: datetime.minuteOf() or time.minuteOf()

use chrono::Timelike;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// MinuteOf function evaluator
pub struct MinuteOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MinuteOfFunctionEvaluator {
    /// Create a new minuteOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "minuteOf".to_string(),
                description: "Extracts the minute component from a datetime or time".to_string(),
                signature: FunctionSignature {
                    input_type: "DateTime | Time".to_string(),
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
impl PureFunctionEvaluator for MinuteOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "minuteOf function takes no arguments".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // minuteOf function should only work on a single value, not collections
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "minuteOf function can only be called on a single datetime or time value"
                    .to_string(),
            ));
        }

        let value = &input[0];
        let minute = match value {
            FhirPathValue::DateTime(datetime, _, _) => datetime.datetime.minute() as i64,
            FhirPathValue::Time(time, _, _) => time.time.minute() as i64,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "minuteOf function can only be applied to DateTime or Time values, got {}",
                        value.type_name()
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::integer(minute)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
