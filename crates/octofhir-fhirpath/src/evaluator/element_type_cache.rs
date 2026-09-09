//! Engine-wide element types, with invalidation ordered against miss publication.
use crate::core::{ModelProvider, model_provider::TypeInfo};
use papaya::HashMap;
use parking_lot::RwLock;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

type Value = Option<Arc<TypeInfo>>;
type LegacyCache = Arc<HashMap<String, Value>>;

#[derive(Default)]
pub(super) struct ElementTypeCache {
    entries: HashMap<String, (u64, Value)>,
    generation: AtomicU64,
    // Only misses and invalidation take this lock. Never held across provider I/O.
    publication: RwLock<()>,
    // Preserve the public caller-owned-cache constructor's source compatibility.
    legacy: Option<LegacyCache>,
}

impl ElementTypeCache {
    pub(super) fn legacy(cache: LegacyCache) -> Self {
        Self {
            legacy: Some(cache),
            ..Default::default()
        }
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub(super) fn invalidate(&self) {
        let _guard = self.publication.write();
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.entries.pin().clear();
    }

    pub(super) async fn resolve_at(
        &self,
        generation: u64,
        provider: &(dyn ModelProvider + Send + Sync),
        parent: &TypeInfo,
        property: &str,
    ) -> octofhir_fhir_model::Result<Value> {
        let invalidated = || {
            octofhir_fhir_model::ModelError::evaluation_error(
                "Schema cache invalidated; prepare a new evaluation context or validation resource",
            )
        };
        if generation != self.generation() {
            return Err(invalidated());
        }
        let parent_name = parent.name.as_deref().unwrap_or(&parent.type_name);
        let mut key = String::with_capacity(parent_name.len() + 1 + property.len());
        key.push_str(parent_name);
        key.push('.');
        key.push_str(property);
        if let Some(cache) = &self.legacy {
            if let Some(value) = cache.pin().get(&key) {
                return Ok(value.clone());
            }
            // Errors are not authoritative absence and must not be cached.
            let value = provider
                .get_element_type(parent, property)
                .await?
                .map(Arc::new);
            cache.pin().insert(key, value.clone());
            return Ok(value);
        }
        if let Some((entry_generation, value)) = self.entries.pin().get(&key)
            && *entry_generation == generation
        {
            return Ok(value.clone());
        }
        let resolved = provider.get_element_type(parent, property).await;
        let _guard = self.publication.read();
        if generation != self.generation() {
            // The parent TypeInfo also belongs to the old schema generation.
            // Never retry using it or label its result as a new-generation entry.
            return Err(invalidated());
        }
        let value = resolved?.map(Arc::new);
        self.entries.pin().insert(key, (generation, value.clone()));
        Ok(value)
    }
}
