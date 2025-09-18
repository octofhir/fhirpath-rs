//! First function implementation

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

pub struct FirstFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FirstFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "first".to_string(),
                description: "Returns the first element of a collection".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::Custom,
                deterministic: true,
                category: FunctionCategory::Subsetting,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for FirstFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        _args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        let result = if input.is_empty() {
            Vec::new()
        } else {
            vec![input[0].clone()]
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(result),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
