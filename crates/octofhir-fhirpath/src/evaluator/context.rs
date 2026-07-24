//! Simple evaluation context for FHIRPath
//!
//! This module provides a simplified evaluation context with proper variable scoping using
//! parent chain pattern for variable scoping.

use papaya::HashMap as LockFreeHashMap;
use std::sync::{Arc, LazyLock};

use crate::core::model_provider::TypeInfo;
use crate::core::trace::SharedTraceProvider;
use crate::core::{Collection, FhirPathValue, ModelProvider};
use octofhir_fhir_model::{ServerProvider, TerminologyProvider, ValidationProvider};

/// Cached base environment variables (sct, loinc, ucum, vs-*, ext-*).
/// Built once via LazyLock, shared across all EvaluationContext instances.
static BASE_ENV_VARIABLES: LazyLock<Arc<LockFreeHashMap<String, FhirPathValue>>> =
    LazyLock::new(|| {
        let env_vars = crate::evaluator::environment_variables::EnvironmentVariables::new();
        let map = LockFreeHashMap::new();

        {
            let guard = map.pin();

            // Standard FHIR environment variables
            if let Some(sct_url) = env_vars.sct_url {
                guard.insert("sct".to_string(), FhirPathValue::string(sct_url));
            }
            if let Some(loinc_url) = env_vars.loinc_url {
                guard.insert("loinc".to_string(), FhirPathValue::string(loinc_url));
            }

            // Value set variables (%vs-*)
            for (name, url) in env_vars.value_sets {
                guard.insert(format!("vs-{name}"), FhirPathValue::string(url));
            }

            // Extension variables (%ext-*)
            for (name, url) in env_vars.extensions {
                guard.insert(format!("ext-{name}"), FhirPathValue::string(url));
            }

            // Custom variables (strip % prefix)
            for (key, value) in env_vars.custom_variables {
                let var_name = if let Some(stripped) = key.strip_prefix('%') {
                    stripped.to_string()
                } else {
                    key
                };
                guard.insert(var_name, value);
            }
        }

        Arc::new(map)
    });

/// Shared state that is identical across all child/nested contexts created
/// from the same root. Consolidating these behind a single Arc reduces clone
/// cost from ~12 Arc increments to ~5.
struct SharedContextState {
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    validation_provider: Option<Arc<dyn ValidationProvider>>,
    server_provider: Option<Arc<dyn ServerProvider>>,
    trace_provider: Option<SharedTraceProvider>,
    resolution_cache: Arc<LockFreeHashMap<String, Arc<serde_json::Value>>>,
    type_info_cache: Arc<LockFreeHashMap<String, Arc<TypeInfo>>>,
    descendants_cache: Arc<LockFreeHashMap<DescendantsKey, DescendantsEntry>>,
    element_type_cache: Arc<LockFreeHashMap<String, Option<Arc<TypeInfo>>>>,
    server_registry: Arc<LockFreeHashMap<String, Arc<dyn ServerProvider>>>,
    base_env_variables: Arc<LockFreeHashMap<String, FhirPathValue>>,
}

/// Key for [`SharedContextState::descendants_cache`]: node allocation address plus
/// the type it was navigated as. The type is part of the key because the same JSON
/// subtree yields differently-typed children depending on the element it came from.
type DescendantsKey = (usize, Arc<str>);

/// Memoized `descendants()` output.
///
/// `_root` is the node the traversal started from, kept alive purely so its
/// allocation address stays reserved — see [`crate::core::node::FhirNode::identity`].
#[derive(Clone)]
struct DescendantsEntry {
    _root: FhirPathValue,
    values: Arc<Vec<FhirPathValue>>,
}

/// Pre-evaluated results for loop-invariant subexpressions of a lambda argument.
///
/// Produced by [`crate::evaluator::lambda_hoisting`] when a lambda function is about
/// to iterate, and inherited by every child context created during that iteration.
///
/// Entries are keyed by the *address* of the AST node. That is sound only because
/// the scope is owned by the lambda call frame: the argument AST it points into
/// outlives the scope, so no address it holds can be recycled while lookups can
/// still reach it. Nested lambdas push a new scope whose `parent` keeps the outer
/// one reachable.
pub struct HoistScope {
    entries: std::collections::HashMap<usize, Collection>,
    parent: Option<Arc<HoistScope>>,
}

impl HoistScope {
    /// Build a scope from `(node address, value)` pairs, nested inside `parent`.
    pub fn new(
        entries: std::collections::HashMap<usize, Collection>,
        parent: Option<Arc<HoistScope>>,
    ) -> Self {
        Self { entries, parent }
    }

    fn lookup(&self, key: usize) -> Option<&Collection> {
        match self.entries.get(&key) {
            Some(value) => Some(value),
            None => self.parent.as_ref().and_then(|parent| parent.lookup(key)),
        }
    }
}

/// Simple evaluation context for FHIRPath
/// Uses parent chain for variable scoping
pub struct EvaluationContext {
    /// Input collection being evaluated
    input_collection: Collection,
    /// Shared state (providers, caches) — single Arc clone on context clone
    shared: Arc<SharedContextState>,
    /// Variables defined in current scope (includes system variables like $this, $index, $total)
    /// Using lock-free HashMap for high-performance variable access
    variables: Arc<LockFreeHashMap<String, FhirPathValue>>,
    /// Parent context for variable scoping
    /// Variables are resolved by checking current scope, then walking parent chain
    /// Using Arc instead of Box to avoid deep cloning of parent chain
    parent_context: Option<Arc<EvaluationContext>>,
    /// Shared root resource value for $this, %resource, %context aliases
    /// Stored as Arc to avoid cloning the same value 5 times during context creation
    root_resource: Option<Arc<FhirPathValue>>,
    /// Loop-invariant subexpressions already evaluated by an enclosing lambda.
    /// `None` on the common path, which keeps the lookup in the node evaluator
    /// down to a null check.
    hoist_scope: Option<Arc<HoistScope>>,
}

/// Helper to create dynamic-only variables (terminologies, factory, server).
/// Base environment variables are cached in BASE_ENV_VARIABLES via LazyLock.
fn create_dynamic_variables(
    terminology_provider: &Option<Arc<dyn TerminologyProvider>>,
    server_provider: &Option<Arc<dyn ServerProvider>>,
) -> LockFreeHashMap<String, FhirPathValue> {
    let map = LockFreeHashMap::new();

    {
        let guard = map.pin();

        // Add %terminologies variable if terminology provider is available
        if let Some(tp) = terminology_provider {
            let terminologies_var =
                crate::evaluator::terminologies_variable::TerminologiesVariable::new(tp.clone());
            guard.insert(
                "terminologies".to_string(),
                terminologies_var.to_fhir_path_value(),
            );
        }

        // Add %factory variable (always available)
        guard.insert(
            "factory".to_string(),
            crate::evaluator::factory_variable::FactoryVariable::to_fhir_path_value(),
        );

        // Add %server variable if server provider is available
        if let Some(sp) = server_provider {
            let server_var = crate::evaluator::server_variable::ServerVariable::new(sp.clone());
            guard.insert("server".to_string(), server_var.to_fhir_path_value());
        }
    }

    map
}

impl EvaluationContext {
    /// Create new evaluation context
    pub fn new(
        input_collection: Collection,
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
        validation_provider: Option<Arc<dyn ValidationProvider>>,
        trace_provider: Option<SharedTraceProvider>,
    ) -> Self {
        Self::new_with_server(
            input_collection,
            model_provider,
            terminology_provider,
            validation_provider,
            trace_provider,
            None,
        )
    }

    /// Create new evaluation context with optional server provider
    pub fn new_with_server(
        input_collection: Collection,
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
        validation_provider: Option<Arc<dyn ValidationProvider>>,
        trace_provider: Option<SharedTraceProvider>,
        server_provider: Option<Arc<dyn ServerProvider>>,
    ) -> Self {
        // Fresh, context-local element-type cache: each root context gets its own.
        Self::new_with_server_and_element_type_cache(
            input_collection,
            model_provider,
            terminology_provider,
            validation_provider,
            trace_provider,
            server_provider,
            Arc::new(LockFreeHashMap::new()),
        )
    }

    /// Like [`new_with_server`](Self::new_with_server), but reuses a caller-owned
    /// `element_type_cache`.
    ///
    /// Element-type resolution (`get_element_type(parentType, property)`) is a
    /// pure function of the loaded schema set, so its results are safe to share
    /// across every context an engine builds. The FHIRPath validator resolves the
    /// same `(type, property)` pairs on every element of every resource (`ele-1`'s
    /// `children()`, `dom-3`'s `descendants()`), so a fresh per-context cache
    /// re-walks the schema from cold each time. Threading one cache through lets
    /// those lookups plateau. The key space is bounded by the schema set, so this
    /// does not grow unboundedly; the engine clears it when schemas change.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_server_and_element_type_cache(
        input_collection: Collection,
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
        validation_provider: Option<Arc<dyn ValidationProvider>>,
        trace_provider: Option<SharedTraceProvider>,
        server_provider: Option<Arc<dyn ServerProvider>>,
        element_type_cache: Arc<LockFreeHashMap<String, Option<Arc<TypeInfo>>>>,
    ) -> Self {
        // Only create dynamic variables (terminologies, factory, server).
        // Base env vars (sct, loinc, ucum, vs-*, ext-*) are in BASE_ENV_VARIABLES.
        let variables = create_dynamic_variables(&terminology_provider, &server_provider);

        // Wrap root value in Arc once to avoid 5x cloning for aliases
        // (this, %resource, resource, %context, context all point to same value)
        let root_resource = input_collection.first().cloned().map(Arc::new);

        // Initialize server registry and register default provider by its base_url
        let server_registry = Arc::new(LockFreeHashMap::new());
        if let Some(ref sp) = server_provider {
            let base = sp.base_url();
            if !base.is_empty() {
                server_registry.pin().insert(base.to_string(), sp.clone());
            }
        }

        let shared = Arc::new(SharedContextState {
            model_provider,
            terminology_provider,
            validation_provider,
            server_provider,
            trace_provider,
            resolution_cache: Arc::new(LockFreeHashMap::new()),
            type_info_cache: Arc::new(LockFreeHashMap::new()),
            descendants_cache: Arc::new(LockFreeHashMap::new()),
            element_type_cache,
            server_registry,
            base_env_variables: BASE_ENV_VARIABLES.clone(),
        });

        Self {
            input_collection: input_collection.clone(),
            shared,
            variables: Arc::new(variables),
            parent_context: None,
            root_resource,
            hoist_scope: None,
        }
    }

    /// Get root resource if available
    pub fn root_resource_value(&self) -> Option<FhirPathValue> {
        self.root_resource
            .as_ref()
            .map(|root| root.as_ref().clone())
    }

    /// Get variable value using parent chain pattern
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        // Check for root resource aliases first (cheap Arc clone instead of HashMap lookup)
        // These are: $this, %resource, resource, %context, context
        if let Some(ref root) = self.root_resource {
            match name {
                "this" => return Some(root.as_ref().clone()),
                "resource" | "%resource" | "context" | "%context" => {
                    // Only return if it's actually a Resource type
                    if matches!(root.as_ref(), FhirPathValue::Resource(_, _, _)) {
                        return Some(root.as_ref().clone());
                    }
                }
                _ => {}
            }
        }

        // Check current scope - papaya HashMap requires pin for access
        if let Some(value) = self.variables.as_ref().pin().get(name) {
            return Some(value.clone());
        }

        // Check cached base environment variables (sct, loinc, ucum, vs-*, ext-*)
        if let Some(value) = self.shared.base_env_variables.pin().get(name) {
            return Some(value.clone());
        }

        // Walk parent chain to resolve variable
        if let Some(parent) = &self.parent_context {
            return parent.get_variable(name);
        }

        None
    }

    /// Set variable in current scope
    pub fn set_variable(&self, name: String, value: FhirPathValue) {
        // papaya HashMap provides lock-free concurrent insertion with pin
        self.variables.as_ref().pin().insert(name, value);
    }

    /// Create independent context for union operations (isolates user-defined variables)
    pub fn create_independent_context(&self) -> Self {
        // Only dynamic variables; base env vars are in shared.base_env_variables
        let variables = create_dynamic_variables(
            &self.shared.terminology_provider,
            &self.shared.server_provider,
        );

        Self {
            input_collection: self.input_collection.clone(),
            shared: self.shared.clone(),
            variables: Arc::new(variables),
            parent_context: None, // Independent context has no parent
            root_resource: self.root_resource.clone(), // Share Arc reference
            hoist_scope: self.hoist_scope.clone(),
        }
    }

    /// Create nested context for defineVariable scoping
    /// Uses Arc to share parent context without deep cloning
    pub fn nest(&self) -> Self {
        Self {
            input_collection: self.input_collection.clone(),
            shared: self.shared.clone(),
            variables: Arc::new(LockFreeHashMap::new()), // Empty variables in nested scope
            parent_context: Some(Arc::new(self.clone())), // Arc avoids recursive deep clone
            root_resource: self.root_resource.clone(),   // Share Arc reference
            hoist_scope: self.hoist_scope.clone(),
        }
    }

    /// Create child context with new input collection
    /// Uses Arc to share parent context without deep cloning
    pub fn create_child_context(&self, new_input: Collection) -> Self {
        Self {
            input_collection: new_input,
            shared: self.shared.clone(),
            variables: Arc::new(LockFreeHashMap::new()), // Empty variables for child context
            parent_context: Some(Arc::new(self.clone())), // Arc avoids recursive deep clone
            root_resource: self.root_resource.clone(),   // Share Arc reference
            hoist_scope: self.hoist_scope.clone(),
        }
    }

    /// Resolve `%vs-[name]` and `%ext-[name]` environment variables dynamically
    /// Dynamic environment variable resolution
    pub fn resolve_environment_variable(&self, name: &str) -> Option<FhirPathValue> {
        // First check if this is a user-defined variable (stored without prefix)
        if let Some(value) = self.get_variable(name) {
            return Some(value);
        }

        // Handle %vs-[name] pattern for value sets
        if let Some(vs_name) = name.strip_prefix("vs-") {
            // Remove "vs-" prefix
            return Some(FhirPathValue::string(format!(
                "http://hl7.org/fhir/ValueSet/{vs_name}"
            )));
        }

        // Handle %ext-[name] pattern for extensions
        if let Some(ext_name) = name.strip_prefix("ext-") {
            // Remove "ext-" prefix
            return Some(FhirPathValue::string(format!(
                "http://hl7.org/fhir/StructureDefinition/{ext_name}"
            )));
        }

        None
    }

    /// Get input collection
    pub fn input_collection(&self) -> &Collection {
        &self.input_collection
    }

    /// Get model provider
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider + Send + Sync> {
        &self.shared.model_provider
    }

    /// Get terminology provider
    pub fn terminology_provider(&self) -> Option<&Arc<dyn TerminologyProvider>> {
        self.shared.terminology_provider.as_ref()
    }

    /// Get validation provider
    pub fn validation_provider(&self) -> Option<&Arc<dyn ValidationProvider>> {
        self.shared.validation_provider.as_ref()
    }

    /// Get server provider
    pub fn server_provider(&self) -> Option<&Arc<dyn ServerProvider>> {
        self.shared.server_provider.as_ref()
    }

    /// Get or create a server provider for a given URL from the registry.
    /// If the URL is already registered, returns the existing provider.
    /// Otherwise, asks the default provider to create a new instance via `with_base_url`.
    pub fn get_or_register_server(&self, url: &str) -> Option<Arc<dyn ServerProvider>> {
        // Check registry first
        if let Some(provider) = self.shared.server_registry.pin().get(url) {
            return Some(provider.clone());
        }
        // Ask default provider to create instance for this URL
        let default = self.shared.server_provider.as_ref()?;
        let new_provider = default.with_base_url(url)?;
        self.shared
            .server_registry
            .pin()
            .insert(url.to_string(), new_provider.clone());
        Some(new_provider)
    }

    /// Register a server provider for a given URL.
    /// Allows external code to pre-register custom providers (e.g., internal FHIR storage).
    pub fn register_server(&self, url: String, provider: Arc<dyn ServerProvider>) {
        self.shared.server_registry.pin().insert(url, provider);
    }

    /// Get the shared server registry
    pub fn server_registry(&self) -> &Arc<LockFreeHashMap<String, Arc<dyn ServerProvider>>> {
        &self.shared.server_registry
    }

    /// Get trace provider
    pub fn trace_provider(&self) -> Option<&SharedTraceProvider> {
        self.shared.trace_provider.as_ref()
    }

    /// Get the shared resolution cache used by resolve() and other reference operations
    pub fn resolution_cache(&self) -> &Arc<LockFreeHashMap<String, Arc<serde_json::Value>>> {
        &self.shared.resolution_cache
    }

    /// Get or fetch TypeInfo from cache, falling back to model provider on cache miss
    /// This reduces redundant model provider calls for the same type
    pub async fn get_or_fetch_type_info(&self, type_name: &str) -> Option<Arc<TypeInfo>> {
        // Check cache first
        if let Some(cached) = self.shared.type_info_cache.pin().get(type_name) {
            return Some(cached.clone());
        }

        // Cache miss - fetch from model provider
        match self.shared.model_provider.get_type(type_name).await {
            Ok(Some(type_info)) => {
                let arc_type_info = Arc::new(type_info);
                self.shared
                    .type_info_cache
                    .pin()
                    .insert(type_name.to_string(), arc_type_info.clone());
                Some(arc_type_info)
            }
            _ => None,
        }
    }

    /// Get the shared TypeInfo cache
    pub fn type_info_cache(&self) -> &Arc<LockFreeHashMap<String, Arc<TypeInfo>>> {
        &self.shared.type_info_cache
    }

    /// Return this context with `scope` pushed on top of any inherited scope.
    pub fn with_hoist_scope(
        mut self,
        entries: std::collections::HashMap<usize, Collection>,
    ) -> Self {
        if entries.is_empty() {
            return self;
        }
        self.hoist_scope = Some(Arc::new(HoistScope::new(entries, self.hoist_scope.take())));
        self
    }

    /// The hoist scope in effect, if a lambda established one.
    pub fn hoist_scope(&self) -> Option<&Arc<HoistScope>> {
        self.hoist_scope.as_ref()
    }

    /// Look up the pre-evaluated value of the AST node at address `key`.
    ///
    /// Returns `None` immediately when no enclosing lambda hoisted anything, which
    /// is the case for almost every node evaluated.
    pub fn hoisted_value(&self, key: usize) -> Option<Collection> {
        self.hoist_scope.as_ref()?.lookup(key).cloned()
    }

    /// Resolve the type of the `property` element of `parent_type`, memoized.
    ///
    /// Tree walks resolve the same handful of (type, property) pairs thousands of
    /// times. Caching skips the schema lookup, and — because the result is handed
    /// out as a shared `Arc` — also the per-node `TypeInfo` clone that tagging a
    /// child value would otherwise cost.
    pub async fn cached_element_type(
        &self,
        parent_type: &TypeInfo,
        property: &str,
    ) -> Option<Arc<TypeInfo>> {
        let key = match &parent_type.name {
            Some(name) => format!("{name}.{property}"),
            None => format!("{}.{property}", parent_type.type_name),
        };

        if let Some(cached) = self.shared.element_type_cache.pin().get(&key) {
            return cached.clone();
        }

        let resolved = self
            .shared
            .model_provider
            .get_element_type(parent_type, property)
            .await
            .ok()
            .flatten()
            .map(Arc::new);

        self.shared
            .element_type_cache
            .pin()
            .insert(key, resolved.clone());
        resolved
    }

    /// Look up a memoized `descendants()` result for `value`.
    ///
    /// Returns `None` for scalars (no stable identity) and on a miss.
    pub fn cached_descendants(&self, value: &FhirPathValue) -> Option<Arc<Vec<FhirPathValue>>> {
        let key = descendants_key(value)?;
        self.shared
            .descendants_cache
            .pin()
            .get(&key)
            .map(|entry| entry.values.clone())
    }

    /// Memoize `descendants` as the full descendant list of `value`.
    ///
    /// No-op for values without a stable identity. The cache lives for the
    /// lifetime of the shared context state, so it is bounded by the resources
    /// actually navigated during one evaluation.
    pub fn cache_descendants(&self, value: &FhirPathValue, descendants: Arc<Vec<FhirPathValue>>) {
        let Some(key) = descendants_key(value) else {
            return;
        };
        self.shared.descendants_cache.pin().insert(
            key,
            DescendantsEntry {
                _root: value.clone(),
                values: descendants,
            },
        );
    }
}

/// Build the memoization key for `value`, or `None` if it has no stable identity.
fn descendants_key(value: &FhirPathValue) -> Option<DescendantsKey> {
    let FhirPathValue::Resource(node, type_info, _) = value else {
        return None;
    };
    let identity = node.identity()?;
    let type_name: Arc<str> = match &type_info.name {
        Some(name) => Arc::from(name.as_str()),
        None => Arc::from(""),
    };
    Some((identity, type_name))
}

/// Clone implementation for EvaluationContext
/// Note: shared fields use Arc, so cloning increments reference counts
/// instead of creating deep copies. With SharedContextState, clone is
/// only 5 Arc increments instead of 12.
impl Clone for EvaluationContext {
    fn clone(&self) -> Self {
        Self {
            input_collection: self.input_collection.clone(),
            shared: self.shared.clone(),
            variables: self.variables.clone(),
            parent_context: self.parent_context.clone(),
            root_resource: self.root_resource.clone(),
            hoist_scope: self.hoist_scope.clone(),
        }
    }
}
