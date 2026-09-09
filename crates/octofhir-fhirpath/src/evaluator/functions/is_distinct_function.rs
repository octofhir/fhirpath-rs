//! isDistinct function implementation
//!
//! Returns true if all items in the collection are distinct (no duplicates)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct IsDistinctFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl IsDistinctFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "isDistinct".to_string(),
                description:
                    "Returns true if all items in the collection are distinct (no duplicates)"
                        .to_string(),
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
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Logic,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for IsDistinctFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::FP0053,
                "isDistinct function takes no arguments".to_string(),
            ));
        }

        // Empty collection is considered distinct
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // Single item is always distinct
        if input.len() == 1 {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        let mut seen = crate::evaluator::value_set::ValueSet::default();
        for value in input {
            if !seen.insert(value) {
                return Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(false)),
                });
            }
        }

        // No duplicates found
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::boolean(true)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
