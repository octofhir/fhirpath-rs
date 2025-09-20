//! Not function implementation
//!
//! The not function returns the logical negation of a boolean value.
//! Syntax: boolean.not()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Not function evaluator
pub struct NotFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl NotFunctionEvaluator {
    /// Create a new not function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "not".to_string(),
                description: "Returns the logical negation of a boolean value".to_string(),
                signature: FunctionSignature {
                    input_type: "Boolean".to_string(),
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
                category: FunctionCategory::Logic,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for NotFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "not function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            match &value {
                FhirPathValue::Boolean(b, type_info, primitive) => {
                    results.push(FhirPathValue::Boolean(
                        !b,
                        type_info.clone(),
                        primitive.clone(),
                    ));
                }
                FhirPathValue::Integer(i, type_info, primitive) => {
                    // Convert integer to boolean: 0 is false, non-zero is true
                    let bool_value = *i != 0;
                    results.push(FhirPathValue::Boolean(
                        !bool_value,
                        type_info.clone(),
                        primitive.clone(),
                    ));
                }
                _ => {
                    // According to FHIRPath spec, return empty collection for type mismatches
                    // rather than throwing an error
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::empty(),
                    });
                }
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
