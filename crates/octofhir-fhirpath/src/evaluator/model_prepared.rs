//! Prepared validation bridge for consumers of the fhir-model trait.
use super::{EvaluationContext, FhirPathEngine, plan::CompiledPlan};
use crate::core::{Collection, FhirPathValue, model_provider::TypeInfo, node::FhirNode};
use octofhir_fhir_model::{
    ExecutableExpression, JsonVariables, ModelError, NodeId, PreparedResource, Result,
};
use serde_json::Value;
use std::sync::Arc;

struct State {
    owner: Arc<()>,
    context: EvaluationContext,
    nodes: Vec<FhirNode>,
    variables: JsonVariables,
}

pub(super) async fn prepare(
    engine: &FhirPathEngine,
    source: Arc<Value>,
    variables: &JsonVariables,
) -> Result<Option<PreparedResource>> {
    let context = engine
        .prepare_context(source.clone(), None, variables)
        .await?;
    let Some(FhirPathValue::Resource(root, _, _)) = context.input_collection().first() else {
        return Ok(None);
    };
    // Same preorder as PreparedResource, using the JSON order even if the
    // consumer enabled serde_json/preserve_order. Never materialize subtrees.
    let mut nodes = Vec::new();
    let mut stack = vec![(source.as_ref(), root)];
    while let Some((json, node)) = stack.pop() {
        nodes.push(node.clone());
        match json {
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate().rev() {
                    stack.push((value, &node.as_array().unwrap()[index]));
                }
            }
            Value::Object(values) => {
                for (key, value) in values.iter().rev() {
                    stack.push((value, node.get(key).unwrap()));
                }
            }
            _ => {}
        }
    }
    Ok(Some(PreparedResource::new(
        source,
        Arc::new(State {
            owner: engine.plan_owner.clone(),
            context,
            nodes,
            variables: variables.clone(),
        }),
    )))
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn evaluate(
    engine: &FhirPathEngine,
    resource: &PreparedResource,
    id: NodeId,
    json: &Value,
    type_name: Option<&str>,
    variables: &JsonVariables,
    expressions: &[Arc<ExecutableExpression>],
) -> Result<Vec<Result<bool>>> {
    let state = resource
        .payload::<State>()
        .ok_or_else(|| ModelError::evaluation_error("Foreign prepared resource"))?;
    if !Arc::ptr_eq(&state.owner, &engine.plan_owner) || resource.node_id(json) != Some(id) {
        return Err(ModelError::evaluation_error(
            "Foreign prepared resource or node",
        ));
    }
    let node = &state.nodes[id.index()];
    state
        .context
        .ensure_current_schema()
        .map_err(|error| ModelError::evaluation_error(error.to_string()))?;
    if state.variables.len() != variables.len()
        || state.variables.iter().any(|(name, value)| {
            !variables
                .get(name)
                .is_some_and(|other| Arc::ptr_eq(value, other))
        })
    {
        return Err(ModelError::evaluation_error(
            "Prepared variables changed; prepare a new resource",
        ));
    }
    let input = if id.index() == 0 && (type_name.is_none() || json.get("resourceType").is_some()) {
        state.context.input_collection().clone()
    } else if json.get("resourceType").is_some() || type_name.is_none() {
        let mut value = FhirPathValue::resource_from_node(node.clone());
        if let Some(name) = json.get("resourceType").and_then(Value::as_str)
            && let Ok(Some(info)) = engine.get_model_provider().get_type(name).await
        {
            value = FhirPathValue::Resource(node.clone(), Arc::new(info), None);
        }
        Collection::single(value)
    } else if json.is_object() {
        let name = type_name.unwrap().to_string();
        Collection::single(FhirPathValue::Resource(
            node.clone(),
            Arc::new(TypeInfo {
                type_name: name.clone(),
                singleton: Some(true),
                namespace: Some("FHIR".into()),
                name: Some(name),
                is_empty: Some(false),
            }),
            None,
        ))
    } else if json.is_array() {
        Collection::single(FhirPathValue::resource_from_node(node.clone()))
    } else {
        // Scalar typing (including Date/DateTime/Time parsing) reuses the
        // established conversion; cloning a scalar never rebuilds a subtree.
        Collection::from_json_typed_arc(
            Arc::new(json.clone()),
            type_name,
            Some(engine.get_model_provider()),
        )
        .await
        .map_err(|e| ModelError::evaluation_error(e.to_string()))?
    };
    let context = state.context.constraint_context(input);
    let mut results = Vec::with_capacity(expressions.len());
    for expression in expressions {
        let Some(plan) = expression.payload::<CompiledPlan>() else {
            results.push(Err(ModelError::evaluation_error(
                "Foreign compiled expression",
            )));
            continue;
        };
        results.push(
            engine
                .evaluate_plan(plan, &context.nest())
                .await
                .map(|result| result.is_constraint_satisfied())
                .map_err(|e| ModelError::evaluation_error(e.to_string())),
        );
    }
    state
        .context
        .ensure_current_schema()
        .map_err(|error| ModelError::evaluation_error(error.to_string()))?;
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::FhirPathEvaluator;

    #[tokio::test]
    async fn typed_root_matches_legacy_and_expression_scopes_are_independent() {
        let engine = crate::testing::test_engine().await;
        let source = Arc::new(serde_json::json!("2000-01-02"));
        let vars = JsonVariables::new();
        let prepared = engine
            .prepare_resource(source.clone(), &vars)
            .await
            .unwrap()
            .unwrap();
        let id = prepared.node_id(&source).unwrap();
        let expressions = [
            "$this is date",
            "$this = @2000-01-02",
            "true.defineVariable('local', 1)",
            "true.defineVariable('local', 2)",
        ];
        let mut plans = Vec::new();
        for expression in expressions {
            plans.push(Arc::new(engine.compile_handle(expression).await.unwrap()));
        }
        let expected = engine
            .evaluate_constraints_shared_context_typed(
                source.clone(),
                Some("date"),
                &vars,
                &expressions,
            )
            .await
            .unwrap();
        let actual = engine
            .evaluate_prepared_constraints(&prepared, id, &source, Some("date"), &vars, &plans)
            .await
            .unwrap();
        assert!(expected.into_iter().all(|result| result.unwrap()));
        assert!(actual.into_iter().all(|result| result.unwrap()));
    }

    #[tokio::test]
    async fn changed_variables_and_foreign_plans_are_rejected_without_aborting_other_slots() {
        let engine = crate::testing::test_engine().await;
        let source = Arc::new(serde_json::json!({"resourceType":"Patient","id":"root"}));
        let vars = JsonVariables::from([("rootResource".into(), source.clone())]);
        let prepared = engine
            .prepare_resource(source.clone(), &vars)
            .await
            .unwrap()
            .unwrap();
        let id = prepared.node_id(&source).unwrap();
        let other = crate::testing::test_engine().await;
        let plans = vec![
            Arc::new(other.compile_handle("true").await.unwrap()),
            Arc::new(ExecutableExpression::new("true")),
            Arc::new(engine.compile_handle("true").await.unwrap()),
            Arc::new(engine.compile_handle("false").await.unwrap()),
        ];
        let results = engine
            .evaluate_prepared_constraints(&prepared, id, &source, None, &vars, &plans)
            .await
            .unwrap();
        assert!(results[0].is_err());
        assert!(results[1].is_err());
        assert!(matches!(results[2], Ok(true)));
        assert!(matches!(results[3], Ok(false)));
        // Equal JSON is not the same prepared variable allocation.
        let replaced = JsonVariables::from([("rootResource".into(), Arc::new((*source).clone()))]);
        for changed in [replaced, JsonVariables::new()] {
            assert!(
                engine
                    .evaluate_prepared_constraints(&prepared, id, &source, None, &changed, &plans,)
                    .await
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn prepared_nodes_match_legacy_typing_root_aliases_and_reject_foreign_handles() {
        let engine = crate::testing::test_engine().await;
        let source = Arc::new(serde_json::json!({"resourceType":"Patient","id":"root",
            "name":[{"family":"Smith"}],"birthDate":"2000-01-02"}));
        let vars = JsonVariables::from([("rootResource".into(), source.clone())]);
        let prepared = engine
            .prepare_resource(source.clone(), &vars)
            .await
            .unwrap()
            .unwrap();
        for (pointer, typ, expressions) in [
            (
                "/name/0",
                None,
                vec![
                    "family = 'Smith'",
                    "%context.family = 'Smith'",
                    "%rootResource.id = 'root'",
                ],
            ),
            (
                "/birthDate",
                Some("date"),
                vec!["$this is date", "$this = @2000-01-02"],
            ),
        ] {
            let node = source.pointer(pointer).unwrap();
            let id = prepared.node_id(node).unwrap();
            let mut plans = Vec::new();
            for expression in &expressions {
                plans.push(Arc::new(engine.compile_handle(expression).await.unwrap()));
            }
            let expected = engine
                .evaluate_constraints_shared_context_typed(
                    Arc::new(node.clone()),
                    typ,
                    &vars,
                    &expressions,
                )
                .await
                .unwrap();
            let actual = engine
                .evaluate_prepared_constraints(&prepared, id, node, typ, &vars, &plans)
                .await
                .unwrap();
            assert!(expected.into_iter().all(|r| r.unwrap()));
            assert!(actual.into_iter().all(|r| r.unwrap()));
            let other = crate::testing::test_engine().await;
            assert!(
                other
                    .evaluate_prepared_constraints(&prepared, id, node, typ, &vars, &plans)
                    .await
                    .is_err()
            );
            assert!(
                engine
                    .evaluate_prepared_constraints(&prepared, id, &node.clone(), typ, &vars, &plans)
                    .await
                    .is_err()
            );
        }
    }
}
