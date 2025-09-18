//! Combine function implementation
//!
//! The combine function merges two collections without deduplication.
//! Syntax: collection1.combine(collection2)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Combine function evaluator
pub struct CombineFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl CombineFunctionEvaluator {
    /// Create a new combine function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
                        is_expression: true,
                        description: "The collection to combine with".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for CombineFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("combine function expects 1 argument, got {}", args.len()),
            ));
        }

        let root_collection = context.get_root_evaluation_context().clone();
        let function_context = context.for_function_evaluation(root_collection);
        let other_result = evaluator.evaluate(&args[0], &function_context).await?;

        // Combine the two collections without deduplication
        let mut combined = Vec::new();

        // Add all items from the input collection
        for item in input {
            combined.push(item);
        }

        // Add all items from the other collection
        for item in other_result.value.into_iter() {
            combined.push(item);
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(combined),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
