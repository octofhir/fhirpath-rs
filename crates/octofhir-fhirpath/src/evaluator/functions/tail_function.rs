//! Tail function implementation
//!
//! The tail function returns a collection containing all but the first item in the input collection.
//! Will return an empty collection if the input collection has no items, or only one item.
//! Syntax: collection.tail()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Tail function evaluator
pub struct TailFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TailFunctionEvaluator {
    /// Create a new tail function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "tail".to_string(),
                description: "Returns a collection containing all but the first item in the input collection. Will return an empty collection if the input collection has no items, or only one item.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Subsetting,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for TailFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "tail function takes no arguments".to_string(),
            ));
        }

        // If input has 0 or 1 items, return empty collection
        if input.len() <= 1 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // Return all but the first item
        let tail_items: Vec<FhirPathValue> = input.into_iter().skip(1).collect();
        Ok(EvaluationResult {
            value: crate::core::Collection::from(tail_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
