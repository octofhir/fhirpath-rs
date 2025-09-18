//! ToTime function implementation
//!
//! The toTime function converts a value to a time.
//! Syntax: value.toTime()

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

pub struct ToTimeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToTimeFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toTime".to_string(),
                description: "Converts a value to a time".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Time".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
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
impl FunctionEvaluator for ToTimeFunctionEvaluator {
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
                "toTime function takes no arguments".to_string(),
            ));
        }

        if input.len() != 1 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let result = match &input[0] {
            FhirPathValue::Time(precision_time, _, _) => {
                // Time is already a time, return as-is
                Some(input[0].clone())
            }
            FhirPathValue::DateTime(precision_datetime, _, _) => {
                // Extract time from datetime
                use crate::core::temporal::PrecisionTime;
                let time = precision_datetime.datetime.time();
                let precision_time = PrecisionTime::new(time, precision_datetime.precision);
                Some(FhirPathValue::time(precision_time))
            }
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as time
                use crate::core::temporal::PrecisionTime;
                if let Some(precision_time) = PrecisionTime::parse(s) {
                    Some(FhirPathValue::time(precision_time))
                } else {
                    None
                }
            }
            _ => {
                // Other types cannot be converted to time
                None
            }
        };

        Ok(EvaluationResult {
            value: match result {
                Some(time_value) => crate::core::Collection::from(vec![time_value]),
                None => crate::core::Collection::empty(),
            },
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
