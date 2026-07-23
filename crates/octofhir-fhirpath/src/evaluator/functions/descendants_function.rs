//! descendants function implementation
//!
//! Retrieves all descendant elements of the current context.
//!
//! Descendants are produced by repeatedly taking [`typed_children`], so every
//! node carries the FHIR element type resolved from its parent via the model
//! provider. Type-sensitive operations on the result — `ofType(canonical)`,
//! `as(uri)`, `is Reference` — therefore work, which invariants like `dom-3`
//! (checking that contained resources are referenced) depend on.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
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

/// Pre-order depth-first collection of every descendant of `item`. FHIR
/// resources are trees, so this terminates without cycle tracking.
fn collect_descendants<'a>(
    item: &'a FhirPathValue,
    context: &'a EvaluationContext,
    out: &'a mut Vec<FhirPathValue>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        for child in typed_children(item, context).await {
            out.push(child.clone());
            collect_descendants(&child, context, out).await;
        }
    })
}

/// Append every descendant of `item` to `out`, going through the context's
/// memoization cache.
///
/// Expressions that walk the same subtree repeatedly — notably the `dom-3`
/// invariant, which evaluates `%resource.descendants()` four times per contained
/// resource — would otherwise re-traverse (and re-resolve the type of) every node
/// on each pass, making the cost quadratic in resource size.
async fn extend_with_descendants(
    item: &FhirPathValue,
    context: &EvaluationContext,
    out: &mut Vec<FhirPathValue>,
) {
    if let Some(cached) = context.cached_descendants(item) {
        out.extend(cached.iter().cloned());
        return;
    }

    let mut collected = Vec::new();
    collect_descendants(item, context, &mut collected).await;

    let collected = Arc::new(collected);
    context.cache_descendants(item, collected.clone());
    out.extend(collected.iter().cloned());
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
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::FP0053,
                "descendants function takes no arguments".to_string(),
            ));
        }

        let mut all_descendants = Vec::new();
        for item in input.iter() {
            if let FhirPathValue::Collection(inner) = item {
                for it in inner.iter() {
                    extend_with_descendants(it, context, &mut all_descendants).await;
                }
            } else {
                extend_with_descendants(item, context, &mut all_descendants).await;
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
