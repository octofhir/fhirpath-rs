//! RepeatAll function implementation
//!
//! The repeatAll function allows duplicate items in output collection.
//! Traverses tree and selects children without checking for duplicates.
//! Undefined order of returned items. Includes safety mechanisms to prevent infinite loops.
//! Syntax: collection.repeatAll(projection)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// RepeatAll function evaluator
pub struct RepeatAllFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl RepeatAllFunctionEvaluator {
    /// Create a new repeatAll function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "repeatAll".to_string(),
                description: "Traverses tree and selects children without checking for duplicates. Allows duplicate items in output collection. Includes safety mechanisms to prevent infinite loops.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "projection".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Expression to evaluate for each item to find children".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Order is undefined, duplicates allowed
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Maximum depth to prevent infinite loops
    const MAX_DEPTH: usize = 1000;

    /// Maximum total items processed to prevent infinite expansion
    const MAX_ITEMS: usize = 10000;

    /// Maximum iterations for iterative approach
    const MAX_ITERATIONS: usize = 1000;
}

#[async_trait::async_trait]
impl FunctionEvaluator for RepeatAllFunctionEvaluator {
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
                "repeatAll function requires one argument (projection expression)".to_string(),
            ));
        }

        let projection_expr = &args[0];
        let mut results = Vec::new();
        let mut processed_count = 0;

        // Use iterative approach instead of recursion to prevent stack overflow
        self.repeat_all_iterative(input, projection_expr, context, &evaluator, &mut results, &mut processed_count).await?;

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl RepeatAllFunctionEvaluator {
    /// Iterative implementation to prevent stack overflow
    async fn repeat_all_iterative(
        &self,
        input: Vec<FhirPathValue>,
        projection_expr: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: &AsyncNodeEvaluator<'_>,
        results: &mut Vec<FhirPathValue>,
        processed_count: &mut usize,
    ) -> Result<()> {
        use std::collections::{HashSet, VecDeque};

        // Use a queue for breadth-first traversal (prevents deep stack)
        let mut queue: VecDeque<(FhirPathValue, usize)> = VecDeque::new();
        let mut seen_values: HashSet<String> = HashSet::new();

        // Initialize queue with input items
        for item in input {
            queue.push_back((item, 0));
        }

        let mut iterations = 0;

        while let Some((current_item, depth)) = queue.pop_front() {
            iterations += 1;
            *processed_count += 1;

            // Safety checks
            if iterations > Self::MAX_ITERATIONS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!("repeatAll exceeded maximum iterations ({}) to prevent infinite loops", Self::MAX_ITERATIONS),
                ));
            }

            if depth > Self::MAX_DEPTH {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!("repeatAll exceeded maximum depth ({}) to prevent infinite recursion", Self::MAX_DEPTH),
                ));
            }

            if *processed_count > Self::MAX_ITEMS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!("repeatAll exceeded maximum item limit ({}) to prevent infinite expansion", Self::MAX_ITEMS),
                ));
            }

            // Create a simple hash for cycle detection (basic but effective)
            let item_hash = format!("{:?}", current_item);

            // Add current item to results (repeatAll includes all items, even duplicates)
            results.push(current_item.clone());

            // Only expand if we haven't seen this exact item at this depth (prevents immediate cycles)
            let item_key = format!("{}@{}", item_hash, depth);
            if seen_values.contains(&item_key) {
                continue; // Skip expansion to prevent immediate cycles
            }
            seen_values.insert(item_key);

            // Create evaluation context for this item
            let single_item_collection = vec![current_item.clone()];
            let iteration_context = EvaluationContext::new(
                crate::core::Collection::from(single_item_collection),
                context.model_provider().clone(),
                context.terminology_provider().clone(),
                context.trace_provider(),
            )
            .await;

            // Evaluate projection expression to get children
            let projection_result = evaluator
                .evaluate(projection_expr, &iteration_context)
                .await?;

            // Add children to queue for processing
            for child in projection_result.value.iter() {
                queue.push_back((child.clone(), depth + 1));
            }
        }

        Ok(())
    }
}
