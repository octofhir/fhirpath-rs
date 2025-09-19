//! Select function implementation
//!
//! The select function evaluates a projection expression for each item in the input collection.
//! Results are flattened into a single collection.
//! Syntax: collection.select(projection)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Select function evaluator
pub struct SelectFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SelectFunctionEvaluator {
    /// Create a new select function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "select".to_string(),
                description: "Evaluates a projection expression for each item in the input collection. Results are flattened.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "projection".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Expression to evaluate for each item".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::FilteringProjection,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for SelectFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "select function requires one argument (projection expression)".to_string(),
            ));
        }

        let projection_expr = &args[0];
        let mut results = Vec::new();

        // Process each item in the input collection
        for (index, item) in input.iter().enumerate() {
            let mut child_context = context.create_child_context(Collection::single(item.clone()));
            child_context.set_variable("$this".to_string(), item.clone());
            child_context.set_variable("$index".to_string(), FhirPathValue::integer(index as i64));

            // Evaluate projection expression with child context
            let result = evaluator
                .evaluate(projection_expr, &child_context)
                .await?;

            // Flatten the results - add all items from the result collection
            for result_item in result.value.iter() {
                results.push(result_item.clone());
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
