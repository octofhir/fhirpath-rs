//! One validation run: fresh resource, all node groups, warm expression plans.
//! Includes preparation/indexing and source cloning in both paths.
use divan::{Bencher, black_box};
use octofhir_fhir_model::{EmptyModelProvider, FhirPathEvaluator, JsonVariables};
use octofhir_fhirpath::{FhirPathEngine, create_function_registry};
use std::sync::Arc;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench(consts = [false, true], args = [10, 100, 1000])]
fn validation_run<const PREPARED: bool>(bencher: Bencher, count: usize) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(create_function_registry()),
            Arc::new(EmptyModelProvider),
        ))
        .unwrap();
    let fixture = serde_json::json!({
        "resourceType": "Patient", "id": "root",
        "name": (0..count).map(|index| serde_json::json!({
            "family": format!("Family-{index}"),
            "given": (0..8).map(|i| format!("Given-{index}-{i}")).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });
    let expressions = [
        "family.exists()",
        "given.count() = 8",
        "%context.family.exists() and %rootResource.id = 'root'",
    ];
    let plans: Vec<_> = expressions
        .iter()
        .map(|expression| Arc::new(runtime.block_on(engine.compile_handle(expression)).unwrap()))
        .collect();
    bencher.bench_local(|| {
        runtime.block_on(async {
            let source = Arc::new(fixture.clone());
            let variables = JsonVariables::from([("rootResource".into(), source.clone())]);
            let prepared = if PREPARED {
                Some(
                    engine
                        .prepare_resource(source.clone(), &variables)
                        .await
                        .unwrap()
                        .unwrap(),
                )
            } else {
                None
            };
            for node in source["name"].as_array().unwrap() {
                let results = match &prepared {
                    Some(prepared) => engine
                        .evaluate_prepared_constraints(
                            prepared,
                            prepared.node_id(node).unwrap(),
                            node,
                            Some("HumanName"),
                            &variables,
                            &plans,
                        )
                        .await
                        .unwrap(),
                    None => engine
                        .evaluate_constraints_shared_context_typed(
                            Arc::new(node.clone()),
                            Some("HumanName"),
                            &variables,
                            &expressions,
                        )
                        .await
                        .unwrap(),
                };
                assert!(black_box(results).into_iter().all(|result| result.unwrap()));
            }
        })
    });
}
