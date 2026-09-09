//! Warm evaluator benchmarks: runtime, parsing and fixtures are outside the timer.
use divan::{Bencher, black_box};
use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{Collection, EvaluationContext, FhirPathEngine, FhirPathValue};
use std::sync::Arc;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench(args = [
    "1 + 2",
    "{}.exists()",
    "'Patient-123'.matches('^[A-Za-z]+-[0-9]+$')",
    "'Patient-123'.matchesFull('[A-Za-z]+-[0-9]+')",
    "'Patient-123'.replaceMatches('[0-9]+', '456')",
    "item.where(active).id",
    "item.where(id.matches('^item-[0-9]+$')).count()",
])]
fn warm_evaluation(bencher: Bencher, expression: &str) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let provider = Arc::new(EmptyModelProvider);
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(octofhir_fhirpath::create_function_registry()),
            provider.clone(),
        ))
        .unwrap();
    let resource = serde_json::json!({
        "resourceType": "Patient",
        "item": (0..100).map(|i| serde_json::json!({
            "id": format!("item-{i}"), "active": i % 2 == 0,
        })).collect::<Vec<_>>()
    });
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(resource)),
        provider,
        None,
        None,
        None,
    );
    runtime
        .block_on(engine.evaluate(expression, &context))
        .unwrap();
    bencher.bench_local(|| {
        black_box(
            runtime
                .block_on(engine.evaluate(black_box(expression), &context))
                .unwrap(),
        )
    });
}

#[divan::bench]
fn empty_collection(bencher: Bencher) {
    bencher.bench_local(|| black_box(Collection::empty()));
}

/// Compare caller-managed and generation-aware engine caches on warm hits.
/// Both retain the same TypeInfo Arc; preparation and cold lookups are untimed.
#[divan::bench(consts = [false, true], threads = [1, 8])]
fn element_type_cache_hit<const MANAGED: bool>(bencher: Bencher) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let provider = Arc::new(EmptyModelProvider);
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(octofhir_fhirpath::create_function_registry()),
            provider.clone(),
        ))
        .unwrap();
    let source = Arc::new(serde_json::json!({"resourceType":"Patient"}));
    let context = if MANAGED {
        runtime
            .block_on(engine.prepare_context(source, None, &Default::default()))
            .unwrap()
    } else {
        EvaluationContext::new(
            Collection::single(FhirPathValue::resource_from_arc(source)),
            provider,
            None,
            None,
            None,
        )
    };
    let parent = octofhir_fhir_model::TypeInfo::new_complex("Patient");
    runtime.block_on(context.cached_element_type(&parent, "name"));
    bencher.bench(|| {
        black_box(runtime.block_on(context.cached_element_type(&parent, black_box("name"))))
    });
}

#[divan::bench]
fn single_collection(bencher: Bencher) {
    bencher.bench_local(|| black_box(Collection::single(FhirPathValue::integer(1))));
}

#[divan::bench(args = [100, 1000, 10000])]
fn distinct_integers(bencher: Bencher, count: usize) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let provider = Arc::new(EmptyModelProvider);
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(octofhir_fhirpath::create_function_registry()),
            provider.clone(),
        ))
        .unwrap();
    let context = EvaluationContext::new(
        Collection::from_values(
            (0..count)
                .map(|i| FhirPathValue::integer(i as i64))
                .collect(),
        ),
        provider,
        None,
        None,
        None,
    );
    runtime
        .block_on(engine.evaluate("distinct()", &context))
        .unwrap();
    bencher.bench_local(|| {
        black_box(
            runtime
                .block_on(engine.evaluate("distinct()", &context))
                .unwrap(),
        )
    });
}

#[divan::bench(threads = [1, 8, 32])]
fn parallel_regex(bencher: Bencher) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let provider = Arc::new(EmptyModelProvider);
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(octofhir_fhirpath::create_function_registry()),
            provider.clone(),
        ))
        .unwrap();
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::string("Patient-123")),
        provider,
        None,
        None,
        None,
    );
    let expression = "matches('^[A-Za-z]+-[0-9]+$')";
    runtime
        .block_on(engine.evaluate(expression, &context))
        .unwrap();
    // A shared engine and context intentionally stress cache/regex contention.
    bencher.bench(|| {
        black_box(
            runtime
                .block_on(engine.evaluate(expression, &context))
                .unwrap(),
        )
    });
}

#[divan::bench(args = [10, 100])]
fn validation_groups(bencher: Bencher, groups: usize) {
    use octofhir_fhir_model::{FhirPathEvaluator, JsonVariables};
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let provider = Arc::new(EmptyModelProvider);
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(octofhir_fhirpath::create_function_registry()),
            provider,
        ))
        .unwrap();
    let resource = serde_json::json!({
        "resourceType":"Patient", "id":"validation",
        "name": (0..1000).map(|i| serde_json::json!({"family":format!("item-{i}")})).collect::<Vec<_>>()
    });
    let fragment = Arc::new(serde_json::json!({"family":"item"}));
    let expressions = ["family.exists()", "%rootResource.id.exists()"];
    bencher.bench_local(|| {
        // New allocation per validation request, reused across its groups.
        // The initial JSON clone is included equally in before/after timings.
        let root = Arc::new(resource.clone());
        let mut variables = JsonVariables::new();
        variables.insert("rootResource".into(), root);
        for _ in 0..groups {
            let results = runtime
                .block_on(engine.evaluate_constraints_shared_context_typed(
                    fragment.clone(),
                    Some("HumanName"),
                    &variables,
                    &expressions,
                ))
                .unwrap();
            assert!(results.into_iter().all(|result| result.unwrap()));
        }
    });
}

/// Shared compiled lambda: expose instruction-state refcount contention, which
/// single-thread warm evaluation cannot detect.
#[divan::bench(threads = [1, 8, 32])]
fn parallel_lambda(bencher: Bencher) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let provider = Arc::new(EmptyModelProvider);
    let engine = runtime
        .block_on(FhirPathEngine::new(
            Arc::new(octofhir_fhirpath::create_function_registry()),
            provider.clone(),
        ))
        .unwrap();
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(serde_json::json!({
            "item": (0..100).map(|i| serde_json::json!({
                "id": format!("item-{i}"), "active": i % 2 == 0,
            })).collect::<Vec<_>>(),
        }))),
        provider,
        None,
        None,
        None,
    );
    let expression = "item.where(id.matches('^item-[0-9]+$')).count()";
    runtime
        .block_on(engine.evaluate(expression, &context))
        .unwrap();
    bencher.bench(|| {
        black_box(
            runtime
                .block_on(engine.evaluate(expression, &context))
                .unwrap(),
        )
    });
}
