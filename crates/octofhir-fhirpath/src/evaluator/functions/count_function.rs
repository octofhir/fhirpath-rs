//! Count function implementation

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

pub struct CountFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl CountFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "count".to_string(),
                description: "Returns the number of elements in a collection".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Integer".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::Custom,
                deterministic: true,
                category: FunctionCategory::Aggregate,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for CountFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        _args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        let count = input.len() as i64;

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::integer(count)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
