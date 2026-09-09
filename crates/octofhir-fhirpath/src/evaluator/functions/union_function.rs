//! Union function implementation
//!
//! The union function returns the union of two collections, removing duplicates.
//! Syntax: collection1.union(collection2)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Union function evaluator
pub struct UnionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl UnionFunctionEvaluator {
    /// Create a new union function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "union".to_string(),
                description: "Returns the union of two collections, removing duplicates"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "other".to_string(),
                        parameter_type: vec!["Any".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The other collection to union with".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
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
impl PureFunctionEvaluator for UnionFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("union function expects 1 argument, got {}", args.len()),
            ));
        }

        let other = args.into_iter().next().unwrap();
        let unique_values = crate::evaluator::value_set::distinct(input.into_iter().chain(other));

        Ok(EvaluationResult {
            value: crate::core::Collection::from(unique_values),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
