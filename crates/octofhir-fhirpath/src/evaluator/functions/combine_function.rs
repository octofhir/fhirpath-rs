//! Combine function implementation
//!
//! The combine function merges two collections without deduplication.
//! Syntax: collection1.combine(collection2)

use std::sync::Arc;

use crate::core::{FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Combine function evaluator - simple concatenation of two collections
pub struct CombineFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl CombineFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "combine".to_string(),
                description: "Merges two collections without deduplication".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "other".to_string(),
                        parameter_type: vec!["Any".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The collection to combine with".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::None,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Combining,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for CombineFunctionEvaluator {
    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(input),
            });
        }

        // Get the other collection from pre-evaluated arguments
        let other_values: Vec<FhirPathValue> = args[0].clone();

        // Combine collections (simple concatenation)
        let combined: Vec<FhirPathValue> = input.into_iter().chain(other_values).collect();

        Ok(EvaluationResult {
            value: crate::core::Collection::from(combined),
        })
    }
}
