//! Resolve function implementation
//!
//! The resolve function resolves references to resources within the current resource tree.
//! This includes contained resources and Bundle entries.
//! Syntax: reference.resolve()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Resolve function evaluator
pub struct ResolveFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ResolveFunctionEvaluator {
    /// Create a new resolve function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
                    if let Some(ref_str) = reference_value.as_str() {
                        Some(ref_str.to_string())
                    } else {
                        None
                    }
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
    ) -> Option<FhirPathValue> {
        // Get the root resource from context
        let root_collection = context.get_root_context();

        // Handle different reference types
        if reference.starts_with('#') {
            // Internal contained resource reference
            self.resolve_contained_reference(&reference[1..], root_collection)
        } else if reference.contains('/') {
            // Relative reference (Resource/id)
            self.resolve_bundle_reference(reference, root_collection)
        } else {
            // Simple id reference - check contained resources
            self.resolve_contained_reference(reference, root_collection)
        }
    }

    /// Resolve contained resource reference
    fn resolve_contained_reference(
        &self,
        id: &str,
        root_collection: &crate::core::Collection,
    ) -> Option<FhirPathValue> {
        // Look through all items in the root collection
        for item in root_collection.iter() {
            if let FhirPathValue::Resource(resource_json, type_info, primitive) = item {
                // Check contained resources
                if let Some(contained_array) = resource_json.get("contained") {
                    if let Some(contained_list) = contained_array.as_array() {
                        for contained_resource in contained_list {
                            if let Some(item_id) = contained_resource.get("id") {
                                if let Some(id_str) = item_id.as_str() {
                                    if id_str == id {
                                        // Return Arc-shared reference to avoid copying
                                        return Some(FhirPathValue::Resource(
                                            Arc::new(contained_resource.clone()),
                                            type_info.clone(),
                                            primitive.clone(),
                                        ));
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

    /// Resolve Bundle entry reference
    fn resolve_bundle_reference(
        &self,
        reference: &str,
        root_collection: &crate::core::Collection,
    ) -> Option<FhirPathValue> {
        // Parse reference (e.g., "Patient/123")
        let parts: Vec<&str> = reference.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        let (resource_type, id) = (parts[0], parts[1]);

        // Look through all items in the root collection
        for item in root_collection.iter() {
            if let FhirPathValue::Resource(resource_json, type_info, primitive) = item {
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
                                                // Return Arc-shared reference to avoid copying
                                                return Some(FhirPathValue::Resource(
                                                    Arc::new(entry_resource.clone()),
                                                    type_info.clone(),
                                                    primitive.clone(),
                                                ));
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
                                // Return Arc-shared reference to avoid copying
                                return Some(item.clone());
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
impl FunctionEvaluator for ResolveFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
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

        let mut resolved_resources = Vec::new();

        for value in &input {
            if let Some(reference_string) = self.extract_reference(value) {
                // Resolve reference within the current resource tree
                if let Some(resolved_resource) =
                    self.resolve_reference_in_tree(&reference_string, context)
                {
                    resolved_resources.push(resolved_resource);
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
