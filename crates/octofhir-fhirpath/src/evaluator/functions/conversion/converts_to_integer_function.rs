//! ConvertsToInteger function implementation
//!
//! The convertsToInteger function tests whether a value can be converted to an integer.
//! Syntax: value.convertsToInteger()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ConvertsToInteger function evaluator
pub struct ConvertsToIntegerFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToIntegerFunctionEvaluator {
    /// Create a new convertsToInteger function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToInteger".to_string(),
                description: "Tests whether a value can be converted to an integer".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
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

    /// Check if a value can be converted to integer
    fn can_convert_to_integer(&self, value: &FhirPathValue) -> bool {
        match value {
            // Already integer
            FhirPathValue::Integer(_, _, _) => true,
            // Decimal can be converted if it has no fractional part
            FhirPathValue::Decimal(d, _, _) => d.fract().is_zero(),
            // String can be converted if it represents a valid integer
            FhirPathValue::String(s, _, _) => s.parse::<i64>().is_ok(),
            // Boolean can be converted to integer (false=0, true=1)
            FhirPathValue::Boolean(_, _, _) => true,
            // Other types cannot be converted
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToIntegerFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "convertsToInteger function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // For single input, return single boolean result
        if input.len() == 1 {
            let can_convert = self.can_convert_to_integer(&input[0]);
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(vec![FhirPathValue::boolean(can_convert)]),
            });
        }

        // For multiple inputs, check if all can be converted
        let all_convertible = input.iter().all(|v| self.can_convert_to_integer(v));
        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::boolean(all_convertible)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
