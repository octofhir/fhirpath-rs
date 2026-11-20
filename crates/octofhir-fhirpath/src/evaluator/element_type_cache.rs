//! Engine-wide element types, with invalidation ordered against miss publication.
use crate::core::{ModelProvider, model_provider::TypeInfo};
use papaya::{Equivalent, HashMap};
use parking_lot::RwLock;
use std::hash::{Hash, Hasher};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

type Value = Option<Arc<TypeInfo>>;
type LegacyCache = Arc<HashMap<String, Value>>;

#[derive(PartialEq)]
struct ElementKey {
    parent: TypeInfo,
    property: String,
}

impl Eq for ElementKey {}

struct ElementKeyRef<'a> {
    parent: &'a TypeInfo,
    property: &'a str,
}

impl Hash for ElementKeyRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parent.type_name.hash(state);
        self.parent.singleton.hash(state);
        self.parent.is_empty.hash(state);
        self.parent.namespace.hash(state);
        self.parent.name.hash(state);
        self.property.hash(state);
    }
}

impl Hash for ElementKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ElementKeyRef {
            parent: &self.parent,
            property: &self.property,
        }
        .hash(state);
    }
}

impl Equivalent<ElementKey> for ElementKeyRef<'_> {
    fn equivalent(&self, key: &ElementKey) -> bool {
        self.parent == &key.parent && self.property == key.property
    }
}

#[derive(Default)]
pub(super) struct ElementTypeCache {
    entries: HashMap<ElementKey, (u64, Value)>,
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
        if let Some(cache) = &self.legacy {
            let parent_name = parent.name.as_deref().unwrap_or(&parent.type_name);
            let key = format!("{parent_name}.{property}");
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
        let query = ElementKeyRef { parent, property };
        if let Some((entry_generation, value)) = self.entries.pin().get(&query)
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
        let key = ElementKey {
            parent: parent.clone(),
            property: property.to_owned(),
        };
        self.entries.pin().insert(key, (generation, value.clone()));
        Ok(value)
    }
}
