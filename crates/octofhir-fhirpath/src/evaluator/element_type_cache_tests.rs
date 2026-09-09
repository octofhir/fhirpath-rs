use super::element_type_cache::ElementTypeCache;
use super::{EvaluationContext, FhirPathEngine, create_function_registry};
use crate::core::{Collection, FhirPathValue};
use octofhir_fhir_model::{
    ElementInfo, EmptyModelProvider, FhirPathEvaluator, JsonVariables, ModelError, ModelProvider,
    Result, TypeInfo,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::sync::Semaphore;

impl ElementTypeCache {
    async fn resolve(
        &self,
        provider: &(dyn ModelProvider + Send + Sync),
        parent: &TypeInfo,
        property: &str,
    ) -> Result<Option<Arc<TypeInfo>>> {
        self.resolve_at(self.generation(), provider, parent, property)
            .await
    }
}

#[derive(Debug)]
struct Provider {
    mode: AtomicUsize,
    calls: AtomicUsize,
    pause_element: AtomicBool,
    pause_type: AtomicBool,
    entered: Semaphore,
    resume: Semaphore,
}

impl Provider {
    fn new(mode: usize) -> Arc<Self> {
        Arc::new(Self {
            mode: AtomicUsize::new(mode),
            calls: AtomicUsize::new(0),
            pause_element: AtomicBool::new(false),
            pause_type: AtomicBool::new(false),
            entered: Semaphore::new(0),
            resume: Semaphore::new(0),
        })
    }

    async fn pause(&self, flag: &AtomicBool) {
        if flag.swap(false, Ordering::SeqCst) {
            self.entered.add_permits(1);
            self.resume.acquire().await.unwrap().forget();
        }
    }

    async fn wait_until_entered(&self) {
        tokio::time::timeout(Duration::from_secs(5), self.entered.acquire())
            .await
            .unwrap()
            .unwrap()
            .forget();
    }
}

#[async_trait::async_trait]
impl ModelProvider for Provider {
    async fn get_type(&self, name: &str) -> Result<Option<TypeInfo>> {
        let result = TypeInfo::new_complex(name);
        self.pause(&self.pause_type).await;
        Ok(Some(result))
    }

    async fn get_element_type(&self, _: &TypeInfo, _: &str) -> Result<Option<TypeInfo>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mode = self.mode.load(Ordering::SeqCst);
        self.pause(&self.pause_element).await;
        match mode {
            0 => Ok(Some(TypeInfo::new_complex("string"))),
            1 => Ok(Some(TypeInfo::new_complex("uri"))),
            2 => Ok(None),
            _ => Err(ModelError::evaluation_error("temporary provider failure")),
        }
    }

    fn of_type(&self, info: &TypeInfo, target: &str) -> Option<TypeInfo> {
        EmptyModelProvider.of_type(info, target)
    }
    fn get_element_names(&self, _: &TypeInfo) -> Vec<String> {
        vec!["value".into()]
    }
    async fn get_children_type(&self, _: &TypeInfo) -> Result<Option<TypeInfo>> {
        Ok(None)
    }
    async fn get_elements(&self, _: &str) -> Result<Vec<ElementInfo>> {
        Ok(vec![])
    }
    async fn get_resource_types(&self) -> Result<Vec<String>> {
        Ok(vec!["Patient".into()])
    }
    async fn get_complex_types(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
    async fn get_primitive_types(&self) -> Result<Vec<String>> {
        Ok(vec!["string".into(), "uri".into()])
    }
}

async fn engine(provider: Arc<Provider>) -> Arc<FhirPathEngine> {
    Arc::new(
        FhirPathEngine::new(Arc::new(create_function_registry()), provider)
            .await
            .unwrap(),
    )
}

fn source() -> Arc<serde_json::Value> {
    Arc::new(serde_json::json!({"resourceType":"Patient", "value":"https://example.org"}))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalidation_discards_inflight_success_absence_and_error_without_overwriting_new_hits() {
    for (mode, warm_new_generation) in [
        (0, true),
        (2, true),
        (3, true),
        (0, false),
        (2, false),
        (3, false),
    ] {
        let provider = Provider::new(mode);
        let cache = Arc::new(ElementTypeCache::default());
        provider.pause_element.store(true, Ordering::SeqCst);
        let pending = tokio::spawn({
            let provider = provider.clone();
            let cache = cache.clone();
            async move {
                cache
                    .resolve(
                        provider.as_ref(),
                        &TypeInfo::new_complex("Patient"),
                        "value",
                    )
                    .await
            }
        });
        provider.wait_until_entered().await;
        provider.mode.store(1, Ordering::SeqCst);
        cache.invalidate();
        // Exercise both a new waiter and an otherwise cold new generation.
        if warm_new_generation {
            assert_eq!(
                cache
                    .resolve(
                        provider.as_ref(),
                        &TypeInfo::new_complex("Patient"),
                        "value"
                    )
                    .await
                    .unwrap()
                    .unwrap()
                    .name
                    .as_deref(),
                Some("uri")
            );
        }
        provider.resume.add_permits(1);
        let error = tokio::time::timeout(Duration::from_secs(5), pending)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert!(error.to_string().contains("Schema cache invalidated"));
        assert_eq!(
            provider.calls.load(Ordering::SeqCst),
            if warm_new_generation { 2 } else { 1 }
        );
        assert_eq!(
            cache
                .resolve(
                    provider.as_ref(),
                    &TypeInfo::new_complex("Patient"),
                    "value"
                )
                .await
                .unwrap()
                .unwrap()
                .name
                .as_deref(),
            Some("uri")
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
    }
}

#[tokio::test]
async fn only_authoritative_absence_is_cached_and_legacy_constructor_stays_compatible() {
    for cache in [
        ElementTypeCache::default(),
        ElementTypeCache::legacy(Arc::new(papaya::HashMap::new())),
    ] {
        let provider = Provider::new(3);
        let parent = TypeInfo::new_complex("Patient");
        assert!(
            cache
                .resolve(provider.as_ref(), &parent, "value")
                .await
                .is_err()
        );
        provider.mode.store(2, Ordering::SeqCst);
        assert!(
            cache
                .resolve(provider.as_ref(), &parent, "value")
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            cache
                .resolve(provider.as_ref(), &parent, "value")
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
    }
    let raw = Arc::new(papaya::HashMap::new());
    raw.pin().insert(
        "Patient.value".into(),
        Some(Arc::new(TypeInfo::new_complex("uri"))),
    );
    let provider = Provider::new(0);
    let context = EvaluationContext::new_with_server_and_element_type_cache(
        Collection::empty(),
        provider.clone(),
        None,
        None,
        None,
        None,
        raw,
    );
    assert_eq!(
        context
            .cached_element_type(&TypeInfo::new_complex("Patient"), "value")
            .await
            .unwrap()
            .name
            .as_deref(),
        Some("uri")
    );
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn cancellation_does_not_hold_the_publication_lock() {
    let provider = Provider::new(0);
    let cache = Arc::new(ElementTypeCache::default());
    provider.pause_element.store(true, Ordering::SeqCst);
    let pending = tokio::spawn({
        let provider = provider.clone();
        let cache = cache.clone();
        async move {
            cache
                .resolve(
                    provider.as_ref(),
                    &TypeInfo::new_complex("Patient"),
                    "value",
                )
                .await
        }
    });
    provider.wait_until_entered().await;
    // This also proves invalidation does not wait for provider I/O.
    cache.invalidate();
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    assert!(
        cache
            .resolve(
                provider.as_ref(),
                &TypeInfo::new_complex("Patient"),
                "value"
            )
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn invalidation_rejects_retained_contexts_sessions_and_resources_but_keeps_plans() {
    let provider = Provider::new(0);
    let engine = engine(provider.clone()).await;
    let source = source();
    let vars = JsonVariables::new();
    let context = engine
        .prepare_context(source.clone(), None, &vars)
        .await
        .unwrap();
    let session = engine.validation_session(source.clone()).await.unwrap();
    let resource = engine
        .prepare_resource(source.clone(), &vars)
        .await
        .unwrap()
        .unwrap();
    let plans = [Arc::new(
        engine
            .compile_handle("descendants().exists()")
            .await
            .unwrap(),
    )];
    let plan = engine.compile_plan("descendants()").unwrap();
    let ast = engine.compile_ast("descendants()").unwrap();
    let old = engine.evaluate_plan(&plan, &context).await.unwrap();
    assert_eq!(
        old.value.first().unwrap().type_info().name.as_deref(),
        Some("string")
    );
    assert!(
        context
            .cached_descendants(context.input_collection().first().unwrap())
            .is_some()
    );
    context.get_or_fetch_type_info("Patient").await.unwrap();
    provider.mode.store(1, Ordering::SeqCst);
    engine.clear_element_type_cache();
    assert!(
        context
            .cached_descendants(context.input_collection().first().unwrap())
            .is_none()
    );
    assert!(context.get_or_fetch_type_info("Patient").await.is_none());
    assert!(
        engine
            .evaluate_plan(&plan, &context)
            .await
            .unwrap_err()
            .to_string()
            .contains("prepare a new")
    );
    assert!(engine.evaluate_ast(&ast, &context).await.is_err());
    assert!(
        engine
            .evaluate_with_metadata("descendants()", &context)
            .await
            .is_err()
    );
    assert!(session.context_at("", None).await.is_err());
    assert!(
        engine
            .evaluate_prepared_constraints(
                &resource,
                resource.node_id(&source).unwrap(),
                &source,
                None,
                &vars,
                &plans
            )
            .await
            .is_err()
    );
    let fresh = engine
        .prepare_context(source.clone(), None, &vars)
        .await
        .unwrap();
    let result = engine.evaluate_plan(&plan, &fresh).await.unwrap();
    assert_eq!(
        result.value.first().unwrap().type_info().name.as_deref(),
        Some("uri")
    );
    let resource = engine
        .prepare_resource(source.clone(), &vars)
        .await
        .unwrap()
        .unwrap();
    assert!(
        engine
            .evaluate_prepared_constraints(
                &resource,
                resource.node_id(&source).unwrap(),
                &source,
                None,
                &vars,
                &plans
            )
            .await
            .unwrap()
            .remove(0)
            .unwrap()
    );
}

#[tokio::test]
async fn invalidation_during_evaluation_rejects_mixed_generation_results() {
    let provider = Provider::new(0);
    let engine = engine(provider.clone()).await;
    let context = engine
        .prepare_context(source(), None, &JsonVariables::new())
        .await
        .unwrap();
    provider.pause_element.store(true, Ordering::SeqCst);
    let pending = tokio::spawn({
        let engine = engine.clone();
        async move { engine.evaluate("descendants()", &context).await }
    });
    provider.wait_until_entered().await;
    provider.mode.store(1, Ordering::SeqCst);
    engine.clear_element_type_cache();
    provider.resume.add_permits(1);
    assert!(
        tokio::time::timeout(Duration::from_secs(5), pending)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
}

#[tokio::test]
async fn invalidation_during_root_preparation_cannot_label_old_types_as_current() {
    let provider = Provider::new(0);
    let engine = engine(provider.clone()).await;
    provider.pause_type.store(true, Ordering::SeqCst);
    let pending = tokio::spawn({
        let engine = engine.clone();
        async move {
            engine
                .prepare_context(source(), None, &JsonVariables::new())
                .await
        }
    });
    provider.wait_until_entered().await;
    engine.clear_element_type_cache();
    provider.resume.add_permits(1);
    assert!(
        tokio::time::timeout(Duration::from_secs(5), pending)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert!(
        engine
            .prepare_context(source(), None, &JsonVariables::new())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn transient_element_error_does_not_poison_retained_descendants() {
    let provider = Provider::new(3);
    let engine = engine(provider.clone()).await;
    let context = engine
        .prepare_context(source(), None, &JsonVariables::new())
        .await
        .unwrap();
    let root = context.input_collection().first().unwrap();
    let old = engine.evaluate("descendants()", &context).await.unwrap();
    assert!(matches!(old.value.first(), Some(FhirPathValue::String(..))));
    assert!(context.cached_descendants(root).is_none());
    provider.mode.store(1, Ordering::SeqCst);
    let fresh = engine.evaluate("descendants()", &context).await.unwrap();
    assert_eq!(
        fresh.value.first().unwrap().type_info().name.as_deref(),
        Some("uri")
    );
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
    engine.evaluate("descendants()", &context).await.unwrap();
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn stale_parent_metadata_cannot_seed_a_new_generation() {
    let provider = Provider::new(0);
    let engine = engine(provider.clone()).await;
    let context = engine
        .prepare_context(source(), None, &JsonVariables::new())
        .await
        .unwrap();
    provider.mode.store(1, Ordering::SeqCst);
    engine.clear_element_type_cache();
    // No lookup was in flight: an old context starts navigating only after clear.
    assert!(
        context
            .cached_element_type(&TypeInfo::new_complex("Patient"), "value")
            .await
            .is_none()
    );
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    let fresh = engine
        .prepare_context(source(), None, &JsonVariables::new())
        .await
        .unwrap();
    assert_eq!(
        fresh
            .cached_element_type(&TypeInfo::new_complex("Patient"), "value")
            .await
            .unwrap()
            .name
            .as_deref(),
        Some("uri")
    );
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
}
