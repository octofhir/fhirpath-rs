//! Resolve function implementation
//!
//! The resolve function resolves references to resources within the current resource tree.
//! This includes contained resources and Bundle entries.
//! Syntax: reference.resolve()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use serde_json::Value as JsonValue;

/// Resolve function evaluator
pub struct ResolveFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ResolveFunctionEvaluator {
    /// Create a new resolve function evaluator
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "resolve".to_string(),
                description: "Resolves references to resources".to_string(),
                signature: FunctionSignature {
                    input_type: "Reference|uri|canonical|url".to_string(),
                    parameters: vec![],
                    return_type: "Resource".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Resolution may depend on external state
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: true, // Requires model provider for resolution
            },
        })
    }

    /// Extract reference string from various input types
    fn extract_reference(&self, value: &FhirPathValue) -> Option<String> {
        match value {
            // Reference resource - extract reference field
            FhirPathValue::Resource(resource_json, _, _) => {
                // Check if this is a Reference resource type
                if let Some(reference_value) = resource_json.get("reference") {
                    reference_value.as_str().map(|ref_str| ref_str.to_string())
                } else {
                    None
                }
            }
            // Direct string URI/canonical/url
            FhirPathValue::String(uri, _, _) => Some(uri.clone()),
            _ => None,
        }
    }

    fn ensure_bundle_index(&self, resource_json: &serde_json::Value, context: &EvaluationContext) {
        const BUNDLE_INDEX_MARKER: &str = "bundle-index-ready";
        let cache = context.resolution_cache();
        let cache_guard = cache.pin();
        if cache_guard.get(BUNDLE_INDEX_MARKER).is_some() {
            return;
        }

        if let Some(rt) = resource_json.get("resourceType")
            && let Some(rt_str) = rt.as_str()
            && rt_str == "Bundle"
            && let Some(entries_array) = resource_json.get("entry")
            && let Some(entries) = entries_array.as_array()
        {
            for entry in entries {
                if let Some(entry_resource) = entry.get("resource")
                    && let Some(resource_type) = entry_resource
                        .get("resourceType")
                        .and_then(|rt| rt.as_str())
                    && let Some(id) = entry_resource.get("id").and_then(|id_val| id_val.as_str())
                {
                    let key = format!("bundle:{resource_type}/{id}");
                    cache_guard.insert(key, Arc::new(entry_resource.clone()));
                }
            }
        }

        cache_guard.insert(BUNDLE_INDEX_MARKER.to_string(), Arc::new(JsonValue::Null));
    }

    fn resolve_contained_reference_from_json(
        &self,
        resource_json: &serde_json::Value,
        id: &str,
    ) -> Option<std::sync::Arc<serde_json::Value>> {
        if let Some(contained_array) = resource_json.get("contained")
            && let Some(contained_list) = contained_array.as_array()
        {
            for contained_resource in contained_list {
                if let Some(item_id) = contained_resource.get("id")
                    && let Some(id_str) = item_id.as_str()
                    && id_str == id
                {
                    return Some(Arc::new(contained_resource.clone()));
                }
            }
        }
        None
    }

    /// Resolve reference within the current resource tree
    fn resolve_reference_in_tree(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Option<std::sync::Arc<serde_json::Value>> {
        // Build cache key based on reference form
        let cache_key = if let Some(stripped) = reference.strip_prefix('#') {
            format!("contained:{}", stripped)
        } else if reference.contains('/') {
            // e.g., "Patient/123"
            format!("bundle:{}", reference)
        } else {
            // simple id referring to contained
            format!("contained:{}", reference)
        };

        // Fast path: check shared resolution cache
        if let Some(cached) = context.resolution_cache().pin().get(&cache_key) {
            return Some(cached.clone());
        }

        // Determine the appropriate root for resolution.
        // Prefer variables that actually hold a Resource, walking parent scopes via get_variable.
        let mut root_resource_opt = context
            .root_resource_value()
            .filter(|value| matches!(value, FhirPathValue::Resource(_, _, _)));

        // Fall back to finding any Resource in the current input collection
        if root_resource_opt.is_none() {
            for v in context.input_collection().iter() {
                if matches!(v, FhirPathValue::Resource(_, _, _)) {
                    root_resource_opt = Some(v.clone());
                    break;
                }
            }
        }

        let result = if let Some(FhirPathValue::Resource(resource_json, _type_info, _primitive)) =
            root_resource_opt.as_ref()
        {
            if reference.contains('/') {
                if let Some(rt) = resource_json.get("resourceType")
                    && let Some(rt_str) = rt.as_str()
                {
                    if rt_str == "Bundle" {
                        self.ensure_bundle_index(resource_json.as_ref(), context);
                        return context.resolution_cache().pin().get(&cache_key).cloned();
                    }

                    if let Some((resource_type, id)) = reference.split_once('/') {
                        let matches_type = rt_str == resource_type;
                        let matches_id = resource_json
                            .get("id")
                            .and_then(|id_val| id_val.as_str())
                            .map(|id_str| id_str == id)
                            .unwrap_or(false);

                        if matches_type && matches_id {
                            return Some(resource_json.clone());
                        }
                    }
                }

                return None;
            }

            // Handle different reference types
            if let Some(stripped) = reference.strip_prefix('#') {
                // Internal contained resource reference
                self.resolve_contained_reference_from_json(resource_json.as_ref(), stripped)
            } else {
                // Simple id reference - check contained resources
                self.resolve_contained_reference_from_json(resource_json.as_ref(), reference)
            }
        } else {
            None
        };

        // Populate cache on hit
        if let Some(ref arc_json) = result {
            context
                .resolution_cache()
                .pin()
                .insert(cache_key, arc_json.clone());
        }

        result
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for ResolveFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "resolve function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let mut resolved_resources: Vec<FhirPathValue> = Vec::new();

        for value in &input {
            if let Some(reference_string) = self.extract_reference(value) {
                // Resolve reference within the current resource tree
                if let Some(resolved_json) =
                    self.resolve_reference_in_tree(&reference_string, context)
                {
                    // Determine resource type
                    let resource_type = resolved_json
                        .get("resourceType")
                        .and_then(|rt| rt.as_str())
                        .unwrap_or("Resource");

                    // Obtain precise TypeInfo from ModelProvider if available
                    let type_info = context
                        .get_or_fetch_type_info(resource_type)
                        .await
                        .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                            type_name: resource_type.to_string(),
                            singleton: Some(true),
                            namespace: Some("FHIR".to_string()),
                            name: Some(resource_type.to_string()),
                            is_empty: Some(false),
                        });

                    // Wrap the resolved JSON as a Resource with correct type info
                    resolved_resources.push(FhirPathValue::Resource(
                        resolved_json,
                        type_info,
                        None,
                    ));
                }
                // If resolution fails, the item is ignored (per FHIR spec)
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(resolved_resources),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
