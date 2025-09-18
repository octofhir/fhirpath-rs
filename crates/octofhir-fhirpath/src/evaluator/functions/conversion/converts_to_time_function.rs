//! ConvertsToTime function implementation
//!
//! This function tests if a value can be converted to a Time.

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

/// ConvertsToTime function evaluator
pub struct ConvertsToTimeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToTimeFunctionEvaluator {
    /// Create a new convertsToTime function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToTime".to_string(),
                description: "Tests if the input can be converted to a Time".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
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
impl FunctionEvaluator for ConvertsToTimeFunctionEvaluator {
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
                "convertsToTime function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::String(s, _, _) => {
                    // Test if string can be parsed as Time
                    use crate::core::temporal::PrecisionTime;
                    PrecisionTime::parse(s).is_some()
                }
                FhirPathValue::Time(_, _, _) => true, // Already a Time
                _ => false, // Other types cannot be converted to Time
            };

            results.push(FhirPathValue::boolean(can_convert));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}