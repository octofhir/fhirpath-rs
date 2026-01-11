//! Simple evaluation context for FHIRPath
//!
//! This module provides a simplified evaluation context with proper variable scoping using
//! parent chain pattern for variable scoping.

use papaya::HashMap as LockFreeHashMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::trace::SharedTraceProvider;
use crate::core::{Collection, FhirPathValue, ModelProvider};
use octofhir_fhir_model::{TerminologyProvider, ValidationProvider};

/// Simple evaluation context for FHIRPath
/// Uses parent chain for variable scoping
pub struct EvaluationContext {
    /// Input collection being evaluated
    input_collection: Collection,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    /// Optional terminology provider
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    /// Optional validation provider
    validation_provider: Option<Arc<dyn ValidationProvider>>,
    /// Optional trace provider
    trace_provider: Option<SharedTraceProvider>,
    /// Variables defined in current scope (includes system variables like $this, $index, $total)
    /// Using lock-free HashMap for high-performance variable access
    variables: Arc<LockFreeHashMap<String, FhirPathValue>>,
    /// Parent context for variable scoping
    /// Variables are resolved by checking current scope, then walking parent chain
    /// Using Arc instead of Box to avoid deep cloning of parent chain
    parent_context: Option<Arc<EvaluationContext>>,
    /// Shared cache for resolved references to avoid repeated cloning and scanning
    resolution_cache: std::sync::Arc<LockFreeHashMap<String, std::sync::Arc<serde_json::Value>>>,
    /// Shared cache for TypeInfo to avoid repeated model provider calls
    /// Key: type name (e.g., "Patient", "HumanName"), Value: TypeInfo
    type_info_cache: std::sync::Arc<LockFreeHashMap<String, TypeInfo>>,
    /// Shared root resource value for $this, %resource, %context aliases
    /// Stored as Arc to avoid cloning the same value 5 times during context creation
    root_resource: Option<Arc<FhirPathValue>>,
}

/// Helper to create well-known environment variables following FHIR specification
/// Setup standard environment variables
fn create_environment_variables() -> HashMap<String, FhirPathValue> {
    let mut env = HashMap::new();

    // Use the EnvironmentVariables struct to get all standard variables
    let env_vars = crate::evaluator::environment_variables::EnvironmentVariables::new();

    // Add standard environment variables from FHIR specification
    if let Some(sct_url) = env_vars.sct_url {
        env.insert("sct".to_string(), FhirPathValue::string(sct_url));
    }
    if let Some(loinc_url) = env_vars.loinc_url {
        env.insert("loinc".to_string(), FhirPathValue::string(loinc_url));
    }

    // Add value set variables (%vs-*)
    for (name, url) in env_vars.value_sets {
        env.insert(format!("vs-{name}"), FhirPathValue::string(url));
    }

    // Add extension variables (%ext-*)
    for (name, url) in env_vars.extensions {
        env.insert(format!("ext-{name}"), FhirPathValue::string(url));
    }

    // Add custom variables (strip % prefix if present since that's just FHIRPath syntax)
    for (key, value) in env_vars.custom_variables {
        let var_name = if let Some(stripped) = key.strip_prefix('%') {
            stripped.to_string()
        } else {
            key
        };
        env.insert(var_name, value);
    }

    env
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
        let mut variables = create_environment_variables();

        // Add %terminologies variable if terminology provider is available
        if let Some(ref terminology_provider) = terminology_provider {
            let terminologies_var =
                crate::evaluator::terminologies_variable::TerminologiesVariable::new(
                    terminology_provider.clone(),
                );
            variables.insert(
                "terminologies".to_string(),
                terminologies_var.to_fhir_path_value(),
            );
        }

        // Add %vs-[name] and %ext-[name] support
        // These will be dynamically resolved when accessed

        // Wrap root value in Arc once to avoid 5x cloning for aliases
        // (this, %resource, resource, %context, context all point to same value)
        let root_resource = input_collection.first().cloned().map(Arc::new);

        Self {
            input_collection: input_collection.clone(),
            model_provider,
            terminology_provider,
            validation_provider,
            trace_provider,
            variables: {
                let lock_free_map = LockFreeHashMap::new();
                for (key, value) in variables {
                    lock_free_map.pin().insert(key, value);
                }
                Arc::new(lock_free_map)
            },
            parent_context: None,
            resolution_cache: std::sync::Arc::new(LockFreeHashMap::new()),
            type_info_cache: std::sync::Arc::new(LockFreeHashMap::new()),
            root_resource,
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
        Self {
            input_collection: self.input_collection.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            validation_provider: self.validation_provider.clone(),
            trace_provider: self.trace_provider.clone(),
            variables: {
                // Only include system environment variables, not user-defined ones
                let system_vars = create_environment_variables();
                // Add %terminologies variable if terminology provider is available
                let mut vars = system_vars;
                if let Some(ref terminology_provider) = self.terminology_provider {
                    let terminologies_var =
                        crate::evaluator::terminologies_variable::TerminologiesVariable::new(
                            terminology_provider.clone(),
                        );
                    vars.insert(
                        "terminologies".to_string(),
                        terminologies_var.to_fhir_path_value(),
                    );
                }
                let lock_free_map = LockFreeHashMap::new();
                for (key, value) in vars {
                    lock_free_map.pin().insert(key, value);
                }
                Arc::new(lock_free_map)
            },
            parent_context: None, // Independent context has no parent
            resolution_cache: self.resolution_cache.clone(),
            type_info_cache: self.type_info_cache.clone(),
            root_resource: self.root_resource.clone(), // Share Arc reference
        }
    }

    /// Create nested context for defineVariable scoping
    /// Uses Arc to share parent context without deep cloning
    pub fn nest(&self) -> Self {
        Self {
            input_collection: self.input_collection.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            validation_provider: self.validation_provider.clone(),
            trace_provider: self.trace_provider.clone(),
            variables: Arc::new(LockFreeHashMap::new()), // Empty variables in nested scope
            parent_context: Some(Arc::new(self.clone())), // Arc avoids recursive deep clone
            resolution_cache: self.resolution_cache.clone(),
            type_info_cache: self.type_info_cache.clone(),
            root_resource: self.root_resource.clone(), // Share Arc reference
        }
    }

    /// Create child context with new input collection
    /// Uses Arc to share parent context without deep cloning
    pub fn create_child_context(&self, new_input: Collection) -> Self {
        Self {
            input_collection: new_input,
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            validation_provider: self.validation_provider.clone(),
            trace_provider: self.trace_provider.clone(),
            variables: Arc::new(LockFreeHashMap::new()), // Empty variables for child context
            parent_context: Some(Arc::new(self.clone())), // Arc avoids recursive deep clone
            resolution_cache: self.resolution_cache.clone(),
            type_info_cache: self.type_info_cache.clone(),
            root_resource: self.root_resource.clone(), // Share Arc reference
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
        &self.model_provider
    }

    /// Get terminology provider
    pub fn terminology_provider(&self) -> Option<&Arc<dyn TerminologyProvider>> {
        self.terminology_provider.as_ref()
    }

    /// Get validation provider
    pub fn validation_provider(&self) -> Option<&Arc<dyn ValidationProvider>> {
        self.validation_provider.as_ref()
    }

    /// Get trace provider
    pub fn trace_provider(&self) -> Option<&SharedTraceProvider> {
        self.trace_provider.as_ref()
    }

    /// Get the shared resolution cache used by resolve() and other reference operations
    pub fn resolution_cache(
        &self,
    ) -> &std::sync::Arc<LockFreeHashMap<String, std::sync::Arc<serde_json::Value>>> {
        &self.resolution_cache
    }

    /// Get or fetch TypeInfo from cache, falling back to model provider on cache miss
    /// This reduces redundant model provider calls for the same type
    pub async fn get_or_fetch_type_info(&self, type_name: &str) -> Option<TypeInfo> {
        // Check cache first
        if let Some(cached) = self.type_info_cache.pin().get(type_name) {
            return Some(cached.clone());
        }

        // Cache miss - fetch from model provider
        match self.model_provider.get_type(type_name).await {
            Ok(Some(type_info)) => {
                // Insert into cache for future use
                self.type_info_cache
                    .pin()
                    .insert(type_name.to_string(), type_info.clone());
                Some(type_info)
            }
            _ => None,
        }
    }

    /// Get the shared TypeInfo cache
    pub fn type_info_cache(&self) -> &std::sync::Arc<LockFreeHashMap<String, TypeInfo>> {
        &self.type_info_cache
    }
}

/// Clone implementation for EvaluationContext
/// Note: shared fields use Arc, so cloning increments reference counts
/// instead of creating deep copies
impl Clone for EvaluationContext {
    fn clone(&self) -> Self {
        Self {
            input_collection: self.input_collection.clone(),
            model_provider: self.model_provider.clone(),
            terminology_provider: self.terminology_provider.clone(),
            validation_provider: self.validation_provider.clone(),
            trace_provider: self.trace_provider.clone(),
            variables: self.variables.clone(),
            parent_context: self.parent_context.clone(),
            resolution_cache: self.resolution_cache.clone(),
            type_info_cache: self.type_info_cache.clone(),
            root_resource: self.root_resource.clone(), // Arc clone is cheap
        }
    }
}
