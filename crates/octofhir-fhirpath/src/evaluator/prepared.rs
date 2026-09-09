//! Prepared resources. Cache entries own their original JSON allocation.
use super::{EvaluationContext, FhirPathEngine};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result, node::FhirNode};
use std::sync::Arc;

pub(super) struct PreparedRoot {
    source: Arc<serde_json::Value>,
    value: FhirPathValue,
    weight: u32,
}

pub(super) struct PreparedRoots(moka::sync::Cache<usize, Arc<PreparedRoot>>);

impl Default for PreparedRoots {
    fn default() -> Self {
        Self(
            moka::sync::Cache::builder()
                .max_capacity(32 * 1024 * 1024)
                .weigher(|_: &usize, value: &Arc<PreparedRoot>| value.weight)
                .build(),
        )
    }
}

impl PreparedRoots {
    pub(super) fn value(&self, json: Arc<serde_json::Value>) -> FhirPathValue {
        let root = self.0.get_with(Arc::as_ptr(&json) as usize, || {
            let value = FhirPathValue::resource_from_arc(json.clone());
            let weight = match &value {
                FhirPathValue::Resource(node, _, _) => node_weight(node),
                _ => std::mem::size_of::<FhirPathValue>(),
            }
            .saturating_mul(2)
            .saturating_add(256)
            .min(u32::MAX as usize) as u32;
            Arc::new(PreparedRoot {
                source: json.clone(),
                value,
                weight,
            })
        });
        // An entry owns the original Arc: it cannot be mutated in place or have
        // its address reused while cached. Check identity before returning data.
        assert!(Arc::ptr_eq(&root.source, &json));
        root.value.clone()
    }
}

// Estimate both container/Arc overhead and strings without serializing JSON.
// The caller doubles this to account for retaining the original serde tree.
// Capacity is an accounting budget, not an exact process RSS limit.
fn node_weight(root: &FhirNode) -> usize {
    let mut stack = vec![root];
    let mut bytes = 0usize;
    while let Some(node) = stack.pop() {
        bytes = bytes.saturating_add(std::mem::size_of::<FhirNode>() + 16);
        match node {
            FhirNode::Str(value) => bytes = bytes.saturating_add(value.len()),
            FhirNode::Array(values) => stack.extend(values.iter()),
            FhirNode::Object(values) => {
                for (key, value) in values.iter() {
                    bytes = bytes.saturating_add(key.len() + 32);
                    stack.push(value);
                }
            }
            _ => {}
        }
    }
    bytes
}

/// One resource for an entire validation run. Node contexts share the immutable
/// root and per-resource caches. Never reuse a session for a different resource.
pub struct ValidationSession<'a> {
    engine: &'a FhirPathEngine,
    root: FhirPathValue,
    context: EvaluationContext,
}

impl<'a> ValidationSession<'a> {
    pub(super) async fn new(
        engine: &'a FhirPathEngine,
        json: Arc<serde_json::Value>,
    ) -> Result<Self> {
        let context = engine
            .prepare_context(json, None, &Default::default())
            .await
            .map_err(|error| {
                FhirPathError::evaluation_error(crate::core::error_code::FP0054, error.to_string())
            })?;
        let root = context.input_collection().first().cloned().ok_or_else(|| {
            FhirPathError::evaluation_error(crate::core::error_code::FP0054, "Resource is empty")
        })?;
        context.set_variable("rootResource".into(), root.clone());
        Ok(Self {
            engine,
            root,
            context,
        })
    }

    /// Prepared root value. Clones share the same immutable tree.
    pub fn root(&self) -> &FhirPathValue {
        &self.root
    }

    /// Isolated node scope selected by JSON Pointer. Complex nodes share their
    /// original allocation; scalar typing follows the model evaluator interface.
    pub async fn context_at(
        &self,
        pointer: &str,
        type_name: Option<&str>,
    ) -> Result<EvaluationContext> {
        let FhirPathValue::Resource(root, _, _) = &self.root else {
            unreachable!()
        };
        let node = root.pointer(pointer).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Unknown resource pointer: {pointer}"),
            )
        })?;
        let collection = if pointer.is_empty() {
            Collection::single(self.root.clone())
        } else if node.is_object() {
            let mut value = FhirPathValue::resource_from_node(node.clone());
            let declared = node
                .get("resourceType")
                .and_then(FhirNode::as_str)
                .or(type_name);
            if let Some(name) = declared {
                let info = self
                    .engine
                    .get_model_provider()
                    .get_type(name)
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                        type_name: name.into(),
                        singleton: Some(true),
                        namespace: Some("FHIR".into()),
                        name: Some(name.into()),
                        is_empty: Some(false),
                    });
                value = FhirPathValue::Resource(node.clone(), Arc::new(info), None);
            }
            Collection::single(value)
        } else {
            Collection::from_json_typed_arc(
                Arc::new(node.to_json()),
                type_name,
                Some(self.engine.get_model_provider()),
            )
            .await?
        };
        let context = self.context.create_child_context(collection.clone());
        if let Some(focus) = collection.first() {
            context.set_this(focus.clone());
        }
        Ok(context)
    }

    /// Evaluate a group of invariants. Each expression gets its own variable
    /// scope, so defineVariable bindings cannot leak to another invariant.
    pub async fn evaluate_constraints(
        &self,
        pointer: &str,
        type_name: Option<&str>,
        expressions: &[&str],
    ) -> Result<Vec<Result<bool>>> {
        let context = self.context_at(pointer, type_name).await?;
        let mut results = Vec::with_capacity(expressions.len());
        for expression in expressions {
            results.push(
                self.engine
                    .evaluate(expression, &context.nest())
                    .await
                    .map(|result| result.is_constraint_satisfied()),
            );
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn prepared_context_preserves_untyped_scalar_and_array_inputs() {
        let engine = crate::testing::test_engine().await;
        for input in [
            serde_json::json!(5),
            serde_json::json!([1, 2]),
            serde_json::json!(true),
            serde_json::json!("abc"),
            serde_json::Value::Null,
        ] {
            let input = Arc::new(input);
            // Preserve the model interface contract: untyped JSON is an opaque
            // resource. Primitive interpretation requires an explicit FHIR type.
            let expected = Collection::from_json_typed_arc(
                input.clone(),
                None,
                Some(engine.get_model_provider()),
            )
            .await
            .unwrap();
            let context = engine
                .prepare_context(input, None, &Default::default())
                .await
                .unwrap();
            assert_eq!(context.input_collection(), &expected);
        }
    }

    #[tokio::test]
    async fn session_shares_tree_and_preserves_root_and_temporal_typing() {
        let engine = crate::testing::test_engine().await;
        let mut resource = Arc::new(serde_json::json!({
            "resourceType":"Patient","id":"one","name":[{"family":"Smith"}],"birthDate":"2000-01-02"
        }));
        let first = engine.prepared_value(resource.clone());
        let second = engine.prepared_value(resource.clone());
        let (FhirPathValue::Resource(a, _, _), FhirPathValue::Resource(b, _, _)) = (first, second)
        else {
            panic!()
        };
        assert_eq!(a.identity(), b.identity());
        let session = engine.validation_session(resource.clone()).await.unwrap();
        let context = session
            .context_at("/name/0", Some("HumanName"))
            .await
            .unwrap();
        let FhirPathValue::Resource(node, _, _) = context.input_collection().first().unwrap()
        else {
            panic!()
        };
        assert_eq!(node.identity(), a.pointer("/name/0").unwrap().identity());
        assert_eq!(
            engine
                .evaluate("%rootResource.id", &context)
                .await
                .unwrap()
                .value
                .to_json_value(),
            serde_json::json!("one")
        );
        let context = session
            .context_at("/birthDate", Some("date"))
            .await
            .unwrap();
        assert_eq!(
            engine
                .evaluate("$this is date", &context)
                .await
                .unwrap()
                .value
                .to_json_value(),
            serde_json::json!(true)
        );
        assert!(session.context_at("/missing", None).await.is_err());
        // Mutating the caller's Arc must produce a separate resource identity.
        Arc::make_mut(&mut resource)["id"] = serde_json::json!("two");
        let updated = engine.prepared_value(resource);
        assert_eq!(updated.to_json_value()["id"], "two");
        assert_eq!(session.root().to_json_value()["id"], "one");
    }
}
