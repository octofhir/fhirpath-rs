//! Distinct function implementation
//!
//! The distinct function returns a collection containing only unique items.
//! Syntax: collection.distinct()

use std::collections::HashSet;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
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
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "distinct function takes no arguments".to_string(),
            ));
        }

        // Use HashSet to track unique values based on their string representation
        let mut seen = HashSet::new();
        let mut unique_items = Vec::new();

        for item in input {
            // Create a key for comparison - this is a simplified approach
            // In a full implementation, we'd need proper equality comparison for FHIR values
            let key = format!("{item:?}");

            if !seen.contains(&key) {
                seen.insert(key);
                unique_items.push(item);
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(unique_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
