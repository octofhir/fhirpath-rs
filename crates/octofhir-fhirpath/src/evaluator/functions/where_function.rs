//! Where function implementation
//!
//! The where function filters a collection based on a boolean expression.
//! Syntax: collection.where(criteria)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Where function evaluator
pub struct WhereFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl WhereFunctionEvaluator {
    /// Create a new where function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "where".to_string(),
                description: "Filters a collection based on a boolean expression".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "criteria".to_string(),
                        parameter_type: vec!["Boolean".to_string()],
                        optional: false,
                        is_expression: true,
                        description: "Boolean expression used to filter items".to_string(),
                        default_value: None,
                    }],
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
impl LazyFunctionEvaluator for WhereFunctionEvaluator {
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
                "where function requires one argument (criteria expression)".to_string(),
            ));
        }

        let criteria_expr = &args[0];
        let mut filtered = Vec::new();

        for (index, item) in input.iter().enumerate() {
            let child_context = context.create_child_context(Collection::single(item.clone()));
            child_context.set_variable("$this".to_string(), item.clone());
            child_context.set_variable("$index".to_string(), FhirPathValue::integer(index as i64));

            // Evaluate criteria expression with child context
            let result = evaluator.evaluate(criteria_expr, &child_context).await?;

            // Check if result is truthy
            if is_truthy(&result.value) {
                filtered.push(item.clone());
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(filtered),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Helper function to determine if a collection is truthy
fn is_truthy(values: &crate::core::Collection) -> bool {
    if values.is_empty() {
        return false;
    }

    for value in values.iter() {
        match value {
            FhirPathValue::Boolean(b, _, _) => {
                if !b {
                    return false;
                }
            }
            _ => return false, // Non-boolean values are falsy in boolean context
        }
    }

    true
}
