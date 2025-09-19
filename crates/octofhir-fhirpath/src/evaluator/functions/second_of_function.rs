//! SecondOf function implementation
//!
//! The secondOf function extracts the second component from a Time or DateTime value.
//! Syntax: time.secondOf() or dateTime.secondOf()

use chrono::Timelike;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// SecondOf function evaluator
pub struct SecondOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SecondOfFunctionEvaluator {
    /// Create a new secondOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "secondOf".to_string(),
                description: "Extracts the second component from a Time or DateTime value"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "DateTime|Time".to_string(),
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
impl PureFunctionEvaluator for SecondOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "secondOf function takes no arguments".to_string(),
            ));
        }

        // Require singleton input
        if input.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "secondOf function requires singleton input".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let value = &input[0];
        match value {
            FhirPathValue::DateTime(dt, _, _) => {
                let second = dt.datetime.second() as i64;
                Ok(EvaluationResult {
                    value: crate::core::Collection::from(vec![FhirPathValue::integer(second)]),
                })
            }
            FhirPathValue::Time(time, _, _) => {
                let second = time.time.second() as i64;
                Ok(EvaluationResult {
                    value: crate::core::Collection::from(vec![FhirPathValue::integer(second)]),
                })
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                format!(
                    "secondOf function can only be applied to DateTime or Time values, got {}",
                    value.type_name()
                ),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}