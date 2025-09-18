//! Empty function implementation

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

pub struct EmptyFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl EmptyFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "empty".to_string(),
                description: "Returns true if the collection is empty".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::Custom,
                deterministic: true,
                category: FunctionCategory::Existence,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for EmptyFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        _args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        let is_empty = input.is_empty();

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(is_empty)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
