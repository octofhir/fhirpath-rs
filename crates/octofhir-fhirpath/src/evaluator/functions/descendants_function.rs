//! descendants function implementation
//!
//! Retrieves all descendant elements of the current context.
//!
//! Descendants are produced by repeatedly taking `typed_children`, so every
//! node carries the FHIR element type resolved from its parent via the model
//! provider. Type-sensitive operations on the result — `ofType(canonical)`,
//! `as(uri)`, `is Reference` — therefore work, which invariants like `dom-3`
//! (checking that contained resources are referenced) depend on.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result, node::FhirNode};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::functions::children_function::typed_children;
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

pub struct DescendantsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DescendantsFunctionEvaluator {
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "descendants".to_string(),
                description: "Returns all descendant elements of the current context".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: true,
            },
        })
    }
}

/// Collect descendants in preorder, sharing cached output for repeated walks.
async fn descendants_of(
    item: &FhirPathValue,
    context: &EvaluationContext,
) -> Arc<Vec<FhirPathValue>> {
    if let Some(cached) = context.cached_descendants(item) {
        return cached;
    }

    let mut collected = Vec::new();
    let mut pending = typed_children(item, context).await;
    pending.reverse();
    while let Some(child) = pending.pop() {
        if matches!(
            child,
            FhirPathValue::Resource(FhirNode::Object(_) | FhirNode::Array(_), _, _)
        ) {
            pending.extend(typed_children(&child, context).await.into_iter().rev());
        }
        collected.push(child);
    }
    let collected = Arc::new(collected);
    context.cache_descendants(item, collected.clone());
    collected
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for DescendantsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        self.evaluate_borrowed(input, context, &args, _evaluator)
            .await
    }

    async fn evaluate_borrowed(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: &[ExpressionNode],
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::FP0053,
                "descendants function takes no arguments".to_string(),
            ));
        }

        if input.len() == 1
            && let Some(item) = input.first()
            && !matches!(item, FhirPathValue::Collection(_))
        {
            return Ok(EvaluationResult {
                value: Collection::from_shared_values(descendants_of(item, context).await),
            });
        }

        let mut all_descendants = Vec::new();
        for item in input.iter() {
            if let FhirPathValue::Collection(inner) = item {
                for it in inner.iter() {
                    all_descendants.extend(descendants_of(it, context).await.iter().cloned());
                }
            } else {
                all_descendants.extend(descendants_of(item, context).await.iter().cloned());
            }
        }

        Ok(EvaluationResult {
            value: Collection::from(all_descendants),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
