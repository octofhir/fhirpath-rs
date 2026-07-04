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
use crate::core::model_provider::ModelProvider;
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
    model_provider: &'a Arc<dyn ModelProvider + Send + Sync>,
    out: &'a mut Vec<FhirPathValue>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        for child in typed_children(item, model_provider).await {
            out.push(child.clone());
            collect_descendants(&child, model_provider, out).await;
        }
    })
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

        let model_provider = context.model_provider();
        let mut all_descendants = Vec::new();
        for item in input.iter() {
            if let FhirPathValue::Collection(inner) = item {
                for it in inner.iter() {
                    collect_descendants(it, model_provider, &mut all_descendants).await;
                }
            } else {
                collect_descendants(item, model_provider, &mut all_descendants).await;
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
