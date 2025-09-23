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

    /// Resolve reference within the current resource tree
    fn resolve_reference_in_tree(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Option<std::sync::Arc<serde_json::Value>> {
        // Determine the appropriate root for resolution.
        // Prefer variables that actually hold a Resource, walking parent scopes via get_variable.
        let mut root_resource_opt: Option<FhirPathValue> = None;
        let var_names = ["%resource", "resource", "%context", "context", "this", "$this"];
        for name in var_names {
            if let Some(v) = context.get_variable(name) {
                if matches!(v, FhirPathValue::Resource(_, _, _)) {
                    root_resource_opt = Some(v);
                    break;
                }
            }
        }
        // Fall back to finding any Resource in the current input collection
        if root_resource_opt.is_none() {
            for v in context.input_collection().iter() {
                if matches!(v, FhirPathValue::Resource(_, _, _)) {
                    root_resource_opt = Some(v.clone());
                    break;
                }
            }
        }

        if let Some(root_resource) = root_resource_opt {
            // Create a single-item collection containing the root resource
            let root_collection = crate::core::Collection::from(vec![root_resource]);

            // Handle different reference types
            if let Some(stripped) = reference.strip_prefix('#') {
                // Internal contained resource reference
                self.resolve_contained_reference(stripped, &root_collection)
            } else if reference.contains('/') {
                // Relative reference (Resource/id)
                self.resolve_bundle_reference(reference, &root_collection)
            } else {
                // Simple id reference - check contained resources
                self.resolve_contained_reference(reference, &root_collection)
            }
        } else {
            None
        }
    }

    /// Resolve contained resource reference and return the resolved JSON
    fn resolve_contained_reference(
        &self,
        id: &str,
        root_collection: &crate::core::Collection,
    ) -> Option<std::sync::Arc<serde_json::Value>> {
        // Look through all items in the root collection
        for item in root_collection.iter() {
            if let FhirPathValue::Resource(resource_json, _type_info, _primitive) = item {
                // Check contained resources
                if let Some(contained_array) = resource_json.get("contained") {
                    if let Some(contained_list) = contained_array.as_array() {
                        for contained_resource in contained_list {
                            if let Some(item_id) = contained_resource.get("id") {
                                if let Some(id_str) = item_id.as_str() {
                                    if id_str == id {
                                        return Some(Arc::new(contained_resource.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Resolve Bundle entry reference and return the resolved JSON
    fn resolve_bundle_reference(
        &self,
        reference: &str,
        root_collection: &crate::core::Collection,
    ) -> Option<std::sync::Arc<serde_json::Value>> {
        // Parse reference (e.g., "Patient/123")
        let parts: Vec<&str> = reference.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        let (resource_type, id) = (parts[0], parts[1]);

        // Look through all items in the root collection
        for item in root_collection.iter() {
            if let FhirPathValue::Resource(resource_json, _type_info, _primitive) = item {
                // Check if this is a Bundle
                if let Some(rt) = resource_json.get("resourceType") {
                    if let Some(rt_str) = rt.as_str() {
                        if rt_str == "Bundle" {
                            // Search in Bundle entries
                            if let Some(entries_array) = resource_json.get("entry") {
                                if let Some(entries) = entries_array.as_array() {
                                    for entry in entries {
                                        if let Some(entry_resource) = entry.get("resource") {
                                            // Check resource type and id
                                            let matches_type = entry_resource
                                                .get("resourceType")
                                                .and_then(|rt| rt.as_str())
                                                .map(|rt_str| rt_str == resource_type)
                                                .unwrap_or(false);

                                            let matches_id = entry_resource
                                                .get("id")
                                                .and_then(|id_val| id_val.as_str())
                                                .map(|id_str| id_str == id)
                                                .unwrap_or(false);

                                            if matches_type && matches_id {
                                                return Some(Arc::new(entry_resource.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Direct resource - check if it matches
                            let matches_type = resource_json
                                .get("resourceType")
                                .and_then(|rt| rt.as_str())
                                .map(|rt_str| rt_str == resource_type)
                                .unwrap_or(false);

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
                }
            }
        }
        None
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
                        .model_provider()
                        .get_type(resource_type)
                        .await
                        .unwrap_or(None)
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
