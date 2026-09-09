//! Distinct function implementation
//!
//! The distinct function returns a collection containing only unique items.
//! Syntax: collection.distinct()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Distinct function evaluator
pub struct DistinctFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DistinctFunctionEvaluator {
    /// Create a new distinct function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "distinct".to_string(),
                description: "Returns a collection containing only unique items.".to_string(),
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
impl PureFunctionEvaluator for DistinctFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        _args: Vec<Collection>,
    ) -> Result<EvaluationResult> {
        self.evaluate_sync(input, _args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, _args: Vec<Collection>) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "distinct function takes no arguments".to_string(),
            ));
        }

        let unique_items = crate::evaluator::value_set::distinct(input);

        Ok(EvaluationResult {
            value: crate::core::Collection::from(unique_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
