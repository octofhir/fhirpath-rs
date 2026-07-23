//! Children function implementation
//!
//! The children function returns the direct child elements of a resource or element.
//! Unlike descendants(), this only returns immediate children, not all nested descendants.
//! Syntax: collection.children()
//!
//! Each child is tagged with its FHIR element type, resolved from the parent's
//! type via the model provider. This lets `children()`/`descendants()` feed
//! type-sensitive operations such as `ofType(canonical)` and `as(uri)` — which
//! invariants like `dom-3` rely on to find references among contained resources.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::model_provider::TypeInfo;
use crate::core::node::FhirNode;
use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Children function evaluator
pub struct ChildrenFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ChildrenFunctionEvaluator {
    /// Create a new children function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "children".to_string(),
                description: "Returns the direct child elements of a resource or element"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Returns unordered collection
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: true,
            },
        })
    }
}

/// Produce the direct children of `item`, each tagged with its FHIR element type
/// resolved from the parent's type via the model provider. Falls back to an
/// untyped value whenever the element type cannot be resolved, so behavior is
/// never worse than an untyped traversal.
pub(crate) async fn typed_children(
    item: &FhirPathValue,
    context: &EvaluationContext,
) -> Vec<FhirPathValue> {
    let mut out = Vec::new();
    let FhirPathValue::Resource(node, parent_type, _) = item else {
        return out;
    };
    match node {
        FhirNode::Object(_) => {
            for (key, value) in node.entries() {
                if key.starts_with('_') || key == "resourceType" {
                    continue;
                }
                let child_type = context.cached_element_type(parent_type, key).await;
                push_typed(value, child_type.as_ref(), &mut out);
            }
        }
        FhirNode::Array(arr) => {
            for value in arr.iter() {
                push_typed(value, Some(parent_type), &mut out);
            }
        }
        _ => {}
    }
    out
}

/// Convert a JSON node into typed FhirPathValue(s), tagging each with
/// `child_type` when available. Arrays flatten to one value per element.
fn push_typed(node: &FhirNode, child_type: Option<&Arc<TypeInfo>>, out: &mut Vec<FhirPathValue>) {
    match node {
        FhirNode::Array(arr) => {
            for e in arr.iter() {
                push_typed(e, child_type, out);
            }
        }
        FhirNode::Null => {}
        _ => {
            let base = match node {
                FhirNode::Str(s) => FhirPathValue::string(s.to_string()),
                FhirNode::Bool(b) => FhirPathValue::boolean(*b),
                FhirNode::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        FhirPathValue::integer(i)
                    } else if let Some(f) = n.as_f64() {
                        match rust_decimal::Decimal::try_from(f) {
                            Ok(d) => FhirPathValue::decimal(d),
                            Err(_) => return,
                        }
                    } else {
                        return;
                    }
                }
                FhirNode::Object(_) => FhirPathValue::resource_from_node(node.clone()),
                FhirNode::Array(_) | FhirNode::Null => return,
            };
            let value = match child_type {
                Some(t) => base.with_type_info(t.clone()),
                None => base,
            };
            out.push(value);
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for ChildrenFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "children function expects no arguments".to_string(),
            ));
        }

        let mut all_children = Vec::new();
        for item in input.iter() {
            if let FhirPathValue::Collection(inner) = item {
                for it in inner.iter() {
                    all_children.extend(typed_children(it, context).await);
                }
            } else {
                all_children.extend(typed_children(item, context).await);
            }
        }

        Ok(EvaluationResult {
            value: Collection::from(all_children),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
