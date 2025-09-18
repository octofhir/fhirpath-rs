//! Trace function implementation

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

pub struct TraceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TraceFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "trace".to_string(),
                description: "Logs the input collection and returns it unchanged".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "name".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Name for the trace output".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "selector".to_string(),
                            parameter_type: vec!["Collection".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Optional expression to evaluate for each item".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(2),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: false, // trace has side effects
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for TraceFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "trace function requires at least one argument (name parameter)".to_string(),
            ));
        }

        // Evaluate the name parameter
        let name_result = evaluator
            .evaluate(
                &args[0],
                &context.create_child_context(crate::core::Collection::empty()),
            )
            .await?;

        let name = if let Some(name_value) = name_result.value.first() {
            match name_value {
                FhirPathValue::String(s, _, _) => s.clone(),
                _ => {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        "trace name parameter must be a string".to_string(),
                    ));
                }
            }
        } else {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "trace name parameter is required".to_string(),
            ));
        };

        // Get trace provider from context
        let trace_provider = context.trace_provider();

        // If there's a second parameter (selector), evaluate it for each item
        if args.len() == 2 {
            for (index, item) in input.iter().enumerate() {
                let item_context = context.create_iteration_context(
                    item.clone(),
                    index as i64,
                    input.len() as i64,
                );
                let selector_result = evaluator.evaluate(&args[1], &item_context).await?;

                // Format the trace message with the selector result
                let selector_str = if selector_result.value.is_empty() {
                    "{}".to_string()
                } else {
                    // Use Display formatting to show only values without type info
                    selector_result.value.values()
                        .iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                // Use trace provider if available, otherwise fall back to eprintln
                if let Some(ref provider) = trace_provider {
                    provider.trace(&name, index, &selector_str);
                } else {
                    eprintln!("TRACE[{}][{}]: {}", name, index, selector_str);
                }
            }
        } else {
            // Simple trace without selector
            for (index, item) in input.iter().enumerate() {
                // Use Display formatting to show only values without type info
                let item_str = format!("{}", item);

                // Use trace provider if available, otherwise fall back to eprintln
                if let Some(ref provider) = trace_provider {
                    provider.trace(&name, index, &item_str);
                } else {
                    eprintln!("TRACE[{}][{}]: {}", name, index, item_str);
                }
            }
        }

        // Return the input collection unchanged
        Ok(EvaluationResult {
            value: crate::core::Collection::from_values(input),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
