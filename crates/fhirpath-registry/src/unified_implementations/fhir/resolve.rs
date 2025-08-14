// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unified resolve() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, resource::FhirResource};

/// Unified resolve() function implementation
/// 
/// Resolves FHIR references to resources. For each item in the collection, if it is a string 
/// that is a uri (or canonical or url), locate the target of the reference, and add it to the 
/// resulting collection. If the item does not resolve to a resource, the item is ignored and 
/// nothing is added to the output collection. The items in the collection may also represent 
/// a Reference, in which case the Reference.reference is resolved.
pub struct UnifiedResolveFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedResolveFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("resolve", FunctionCategory::FhirSpecific)
            .display_name("Resolve Reference")
            .description("Resolves FHIR references to resources")
            .example("Patient.managingOrganization.resolve()")
            .example("Bundle.entry.resource.reference.resolve()")
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Resource)))
            .execution_mode(ExecutionMode::Async) // Async because it may need to fetch resources
            .pure(false) // Not pure because it may access external resources
            .lsp_snippet("resolve()")
            .keywords(vec!["resolve", "reference", "resource", "bundle", "contained"])
            .usage_pattern(
                "Resolve FHIR references",
                "reference.resolve()",
                "Reference resolution and resource lookup"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedResolveFunction {
    fn name(&self) -> &str {
        "resolve"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Async
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // For sync execution, only resolve contained resources and simple bundle lookups
        // More complex resolution would require async
        
        // Validate no arguments
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }

        let mut resolved_resources = Vec::new();

        // Process the input collection
        let items = match &context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => vec![single],
        };

        for item in items {
            if let Some(resolved) = self.resolve_item_sync(item, context) {
                resolved_resources.push(resolved);
            }
            // Items that cannot be resolved are ignored as per spec
        }

        Ok(FhirPathValue::collection(resolved_resources))
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // For async execution, we can perform full resolution including external lookups
        // For now, delegate to sync version since external resolution would require ModelProvider
        self.evaluate_sync(args, context)
    }
}

impl UnifiedResolveFunction {
    /// Resolve a single item synchronously (contained resources and bundle lookups only)
    fn resolve_item_sync(
        &self,
        item: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Option<FhirPathValue> {
        match item {
            // Handle string URIs/references
            FhirPathValue::String(uri) => self.resolve_string_reference(uri, context),

            // Handle Reference resources
            FhirPathValue::Resource(resource) => {
                if self.is_reference(resource) {
                    self.resolve_reference_resource(resource, context)
                } else {
                    // Not a reference - ignore
                    None
                }
            }

            // Other types cannot be resolved
            _ => None,
        }
    }

    /// Check if a resource is a Reference type
    fn is_reference(&self, resource: &FhirResource) -> bool {
        // Check if this is a Reference resource by looking for 'reference' field
        if let Some(obj) = resource.as_json().as_object() {
            obj.contains_key("reference")
        } else {
            false
        }
    }

    /// Resolve a Reference resource by extracting its reference field
    fn resolve_reference_resource(
        &self,
        resource: &FhirResource,
        context: &EvaluationContext,
    ) -> Option<FhirPathValue> {
        if let Some(obj) = resource.as_json().as_object() {
            if let Some(reference_value) = obj.get("reference") {
                if let Some(reference_str) = reference_value.as_str() {
                    return self.resolve_string_reference(reference_str, context);
                }
            }
        }
        None
    }

    /// Resolve a string reference (URI/URL)
    fn resolve_string_reference(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Option<FhirPathValue> {
        // Handle fragment references to contained resources (e.g., "#obs1")
        if let Some(contained_id) = reference.strip_prefix('#') {
            return self.resolve_contained_resource(contained_id, context);
        }

        // Try to resolve from Bundle if we're in Bundle context
        if let Some(resolved) = self.resolve_from_bundle(reference, context) {
            return Some(resolved);
        }

        // For external references, we'd need async resolution with ModelProvider
        // Return None for now as these cannot be resolved synchronously
        None
    }

    /// Resolve a contained resource by ID
    fn resolve_contained_resource(
        &self,
        id: &str,
        context: &EvaluationContext,
    ) -> Option<FhirPathValue> {
        // Get the root resource from context
        if let FhirPathValue::Resource(root_resource) = &context.root {
            if let Some(root_obj) = root_resource.as_json().as_object() {
                // Look for 'contained' array
                if let Some(contained_array) = root_obj.get("contained") {
                    if let Some(contained_items) = contained_array.as_array() {
                        // Search for resource with matching id
                        for contained_item in contained_items {
                            if let Some(contained_obj) = contained_item.as_object() {
                                if let Some(contained_id) = contained_obj.get("id") {
                                    if let Some(contained_id_str) = contained_id.as_str() {
                                        if contained_id_str == id {
                                            // Found the contained resource - return it
                                            let resource =
                                                FhirResource::from_json(contained_item.clone());
                                            return Some(FhirPathValue::Resource(resource.into()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Resource not found in contained resources
        None
    }

    /// Resolve a reference from a Bundle context
    fn resolve_from_bundle(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Option<FhirPathValue> {
        // Check if we're in a Bundle context
        let bundle = self.find_bundle_in_context(context)?;

        if let Some(bundle_obj) = bundle.as_json().as_object() {
            if let Some(entries) = bundle_obj.get("entry") {
                if let Some(entry_array) = entries.as_array() {
                    // Try to find matching entry
                    for entry in entry_array {
                        if let Some(entry_obj) = entry.as_object() {
                            // Check if fullUrl matches the reference
                            if let Some(full_url) = entry_obj.get("fullUrl") {
                                if let Some(full_url_str) = full_url.as_str() {
                                    if self.reference_matches(reference, full_url_str) {
                                        // Found matching entry, return its resource
                                        if let Some(resource) = entry_obj.get("resource") {
                                            let fhir_resource =
                                                FhirResource::from_json(resource.clone());
                                            return Some(FhirPathValue::Resource(
                                                fhir_resource.into(),
                                            ));
                                        }
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

    /// Find a Bundle in the evaluation context
    fn find_bundle_in_context<'a>(
        &self,
        context: &'a EvaluationContext,
    ) -> Option<&'a FhirResource> {
        // First check if root is a Bundle
        if let FhirPathValue::Resource(resource) = &context.root {
            if let Some(obj) = resource.as_json().as_object() {
                if let Some(resource_type) = obj.get("resourceType") {
                    if let Some(type_str) = resource_type.as_str() {
                        if type_str == "Bundle" {
                            return Some(resource);
                        }
                    }
                }
            }
        }

        // Could also check parent contexts in the future
        None
    }

    /// Check if a reference matches a fullUrl
    fn reference_matches(&self, reference: &str, full_url: &str) -> bool {
        // Direct match
        if reference == full_url {
            return true;
        }

        // Check if reference is relative and fullUrl ends with it
        // e.g., "Patient/123" matches "http://example.com/Patient/123"
        if !reference.starts_with("http://")
            && !reference.starts_with("https://")
            && !reference.starts_with("urn:")
        {
            // It's a relative reference
            return full_url.ends_with(&format!("/{reference}"));
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::resource::FhirResource;
    use serde_json::json;

    #[tokio::test]
    async fn test_unified_resolve_function() {
        let resolve_func = UnifiedResolveFunction::new();
        
        // Test with contained resource
        let patient_with_contained = json!({
            "resourceType": "Patient",
            "id": "patient1",
            "contained": [
                {
                    "resourceType": "Organization",
                    "id": "org1",
                    "name": "Test Organization"
                }
            ],
            "managingOrganization": {
                "reference": "#org1"
            }
        });
        
        let patient_resource = FhirResource::from_json(patient_with_contained);
        let reference_value = FhirPathValue::String("#org1".into());
        let mut context = EvaluationContext::new(reference_value);
        context.root = FhirPathValue::Resource(patient_resource.into());
        
        let result = resolve_func.evaluate_sync(&[], &context).unwrap();
        
        // Should resolve to the contained organization
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Resource(resolved)) = items.get(0) {
                    assert_eq!(
                        resolved.get_property("resourceType").and_then(|v| v.as_str()),
                        Some("Organization")
                    );
                    assert_eq!(
                        resolved.get_property("id").and_then(|v| v.as_str()),
                        Some("org1")
                    );
                } else {
                    panic!("Expected Resource result");
                }
            },
            _ => panic!("Expected Collection result"),
        }
        
        // Test with non-resolvable reference
        let non_resolvable = FhirPathValue::String("Patient/nonexistent".into());
        let context = EvaluationContext::new(non_resolvable);
        let result = resolve_func.evaluate_sync(&[], &context).unwrap();
        
        // Should return empty collection
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 0);
            },
            _ => panic!("Expected empty Collection result"),
        }
        
        // Test metadata
        assert_eq!(resolve_func.name(), "resolve");
        assert_eq!(resolve_func.execution_mode(), ExecutionMode::Async);
        assert_eq!(resolve_func.metadata().basic.display_name, "Resolve Reference");
        assert!(!resolve_func.metadata().basic.is_pure);
    }
    
    #[tokio::test]
    async fn test_resolve_from_bundle() {
        let resolve_func = UnifiedResolveFunction::new();
        
        // Create a bundle with entries
        let bundle = json!({
            "resourceType": "Bundle",
            "id": "bundle1",
            "entry": [
                {
                    "fullUrl": "http://example.com/Patient/123",
                    "resource": {
                        "resourceType": "Patient",
                        "id": "123",
                        "name": [{"family": "Doe"}]
                    }
                },
                {
                    "fullUrl": "http://example.com/Organization/456",
                    "resource": {
                        "resourceType": "Organization",
                        "id": "456",
                        "name": "Test Org"
                    }
                }
            ]
        });
        
        let bundle_resource = FhirResource::from_json(bundle);
        
        // Test resolving absolute reference
        let absolute_ref = FhirPathValue::String("http://example.com/Patient/123".into());
        let mut context = EvaluationContext::new(absolute_ref);
        context.root = FhirPathValue::Resource(bundle_resource.clone().into());
        
        let result = resolve_func.evaluate_sync(&[], &context).unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Resource(resolved)) = items.get(0) {
                    assert_eq!(
                        resolved.get_property("resourceType").and_then(|v| v.as_str()),
                        Some("Patient")
                    );
                    assert_eq!(
                        resolved.get_property("id").and_then(|v| v.as_str()),
                        Some("123")
                    );
                } else {
                    panic!("Expected Resource result");
                }
            },
            _ => panic!("Expected Collection result"),
        }
        
        // Test resolving relative reference
        let relative_ref = FhirPathValue::String("Patient/123".into());
        let mut context = EvaluationContext::new(relative_ref);
        context.root = FhirPathValue::Resource(bundle_resource.into());
        
        let result = resolve_func.evaluate_sync(&[], &context).unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Resource(resolved)) = items.get(0) {
                    assert_eq!(
                        resolved.get_property("resourceType").and_then(|v| v.as_str()),
                        Some("Patient")
                    );
                } else {
                    panic!("Expected Resource result");
                }
            },
            _ => panic!("Expected Collection result"),
        }
    }
}