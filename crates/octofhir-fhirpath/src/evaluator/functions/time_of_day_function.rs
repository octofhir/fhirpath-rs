//! TimeOfDay function implementation
//!
//! The timeOfDay function returns the current time.
//! Syntax: timeOfDay()

use std::sync::Arc;

use crate::core::temporal::PrecisionTime;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// TimeOfDay function evaluator
pub struct TimeOfDayFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TimeOfDayFunctionEvaluator {
    /// Create a new timeOfDay function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "timeOfDay".to_string(),
                description: "Returns the current time".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Time".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: false, // Current time is non-deterministic
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for TimeOfDayFunctionEvaluator {
    async fn evaluate(
        &self,
        _input: Collection,
        _args: Vec<Collection>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "timeOfDay function takes no arguments".to_string(),
            ));
        }

        // Get current system time
        let now = chrono::Utc::now();
        let current_time = now.time();

        // Create a PrecisionTime from the current time with second precision
        let precision_time = PrecisionTime::from_time(current_time);

        let result = FhirPathValue::time(precision_time);

        Ok(EvaluationResult {
            value: crate::core::Collection::single(result),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
