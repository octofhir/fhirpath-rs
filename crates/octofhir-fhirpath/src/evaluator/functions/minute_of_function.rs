//! MinuteOf function implementation
//!
//! The minuteOf function extracts the minute component from a datetime or time.
//! Syntax: datetime.minuteOf() or time.minuteOf()

use chrono::Timelike;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// MinuteOf function evaluator
pub struct MinuteOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MinuteOfFunctionEvaluator {
    /// Create a new minuteOf function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
impl FunctionEvaluator for MinuteOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "minuteOf function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let minute = match &value {
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

            results.push(FhirPathValue::integer(minute));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
