//! Single function implementation
//!
//! The single function returns the single item in the input if there is just one item.
//! If the input collection is empty, the result is empty. If there are multiple items, an error is signaled.
//! Syntax: collection.single()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Single function evaluator
pub struct SingleFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SingleFunctionEvaluator {
    /// Create a new single function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "single".to_string(),
                description: "Returns the single item in the input if there is just one item. If the input collection is empty, the result is empty. If there are multiple items, an error is signaled.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
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
impl FunctionEvaluator for SingleFunctionEvaluator {
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
                "single function takes no arguments".to_string(),
            ));
        }

        // If input is empty, return empty
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // If there is exactly one item, return it
        if input.len() == 1 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(input),
            });
        }

        // If there are multiple items, signal an error
        Err(FhirPathError::evaluation_error(
            crate::core::error_code::FP0053,
            format!("single() expected 0 or 1 items, got {}", input.len()),
        ))
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
