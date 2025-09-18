//! Where function implementation
//!
//! The where function filters a collection based on a boolean expression.
//! Syntax: collection.where(criteria)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Where function evaluator
pub struct WhereFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl WhereFunctionEvaluator {
    /// Create a new where function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
impl FunctionEvaluator for WhereFunctionEvaluator {
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
            // Create single-element collection for this item (focus)
            let single_item_collection = vec![item.clone()];

            // Create nested context for this iteration
            let mut iteration_context = EvaluationContext::new(
                crate::core::Collection::from(single_item_collection.clone()),
                context.model_provider().clone(),
                context.terminology_provider().clone(),
                context.trace_provider(),
            )
            .await;

            // Set lambda variables: $this = single item collection, $index = current index
            iteration_context.set_system_this(item.clone());
            iteration_context.set_system_index(index as i64);

            // Evaluate criteria expression in the iteration context
            let result = evaluator
                .evaluate(criteria_expr, &iteration_context)
                .await?;

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
