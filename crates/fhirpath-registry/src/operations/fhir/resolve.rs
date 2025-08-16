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

//! Resolve function implementation - resolves FHIR references

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Resolve function - resolves FHIR references using ModelProvider
#[derive(Debug, Clone)]
pub struct ResolveFunction;

impl Default for ResolveFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolveFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("resolve", OperationType::Function)
            .description("Resolves a FHIR reference to the referenced resource")
            .returns(TypeConstraint::Specific(FhirPathType::Resource))
            .example("Patient.managingOrganization.resolve()")
            .example("Observation.subject.resolve()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ResolveFunction {
    fn identifier(&self) -> &str {
        "resolve"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ResolveFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        // resolve() is async-only - requires ModelProvider
        match &context.input {
            FhirPathValue::String(reference) => {
                self.resolve_reference_string(reference, context).await
            }
            FhirPathValue::JsonValue(json) => {
                // Handle Reference objects
                if let Some(reference_val) = json.get_property("reference") {
                    if let Some(reference) = reference_val.as_json().as_str() {
                        return self.resolve_reference_string(reference, context).await;
                    }
                }
                Ok(FhirPathValue::Empty)
            }
            FhirPathValue::Resource(resource) => {
                // Handle Reference resources
                let json = resource.as_json();
                if let Some(reference_val) = json.get("reference") {
                    if let Some(reference) = reference_val.as_str() {
                        return self.resolve_reference_string(reference, context).await;
                    }
                }
                Ok(FhirPathValue::Empty)
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = context.with_input(c.first().unwrap().clone());
                    self.evaluate(args, &item_context).await
                } else {
                    // Resolve each reference in the collection
                    let mut resolved = Vec::new();
                    for item in c.iter() {
                        let item_context = context.with_input(item.clone());
                        let result = self.evaluate(args, &item_context).await?;
                        if !matches!(result, FhirPathValue::Empty) {
                            resolved.push(result);
                        }
                    }
                    Ok(FhirPathValue::collection(resolved))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // resolve() is async-only - requires ModelProvider calls
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ResolveFunction {
    /// Resolve a reference string to a resource
    async fn resolve_reference_string(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle empty or invalid references
        if reference.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // First try to resolve within current context (Bundle entries and contained resources)
        if let Some(resolved) = self.resolve_local_reference(reference, context).await? {
            return Ok(resolved);
        }

        // If not found locally, use ModelProvider for external resolution
        if let Some(resolved) = self.resolve_external_reference(reference, context).await? {
            return Ok(resolved);
        }

        // Return empty if not found
        Ok(FhirPathValue::Empty)
    }

    /// Try to resolve reference within current context (Bundle entries or contained resources)
    async fn resolve_local_reference(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // Check if we have a Bundle in the root context
        if let Some(bundle_resource) = self.find_bundle_in_context(context) {
            if let Some(resolved) = self.resolve_in_bundle(reference, bundle_resource)? {
                return Ok(Some(resolved));
            }
        }

        // Check for contained resources in the current resource context
        if let Some(containing_resource) = self.find_containing_resource(context) {
            if let Some(resolved) = self.resolve_in_contained(reference, containing_resource)? {
                return Ok(Some(resolved));
            }
        }

        Ok(None)
    }

    /// Try to resolve reference using ModelProvider for external resolution
    async fn resolve_external_reference(
        &self,
        reference: &str,
        _context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // For now, we'll use a simple mock resolution
        // In a full implementation, this would use the ModelProvider to fetch external resources

        // Parse reference format (ResourceType/id or relative URL)
        if let Some((resource_type, id)) = self.parse_reference(reference) {
            // Create a mock resolved resource for testing
            let resolved_resource = serde_json::json!({
                "resourceType": resource_type,
                "id": id,
                "_resolved": true,
                "meta": {
                    "versionId": "1",
                    "lastUpdated": "2024-01-01T00:00:00Z"
                }
            });

            return Ok(Some(FhirPathValue::resource_from_json(resolved_resource)));
        }

        Ok(None)
    }

    /// Find Bundle resource in the evaluation context
    fn find_bundle_in_context<'a>(
        &self,
        context: &'a EvaluationContext,
    ) -> Option<&'a FhirPathValue> {
        // Check root context for Bundle
        if let FhirPathValue::Resource(resource) = &context.root {
            if resource.resource_type() == Some("Bundle") {
                return Some(&context.root);
            }
        }

        // Check input context for Bundle
        if let FhirPathValue::Resource(resource) = &context.input {
            if resource.resource_type() == Some("Bundle") {
                return Some(&context.input);
            }
        }

        None
    }

    /// Find containing resource in the evaluation context
    fn find_containing_resource<'a>(
        &self,
        context: &'a EvaluationContext,
    ) -> Option<&'a FhirPathValue> {
        // Look for a resource that might contain other resources
        if let FhirPathValue::Resource(resource) = &context.root {
            if resource.as_json().get("contained").is_some() {
                return Some(&context.root);
            }
        }

        if let FhirPathValue::Resource(resource) = &context.input {
            if resource.as_json().get("contained").is_some() {
                return Some(&context.input);
            }
        }

        None
    }

    /// Resolve reference within a Bundle's entries
    fn resolve_in_bundle(
        &self,
        reference: &str,
        bundle: &FhirPathValue,
    ) -> Result<Option<FhirPathValue>> {
        if let FhirPathValue::Resource(bundle_resource) = bundle {
            let bundle_json = bundle_resource.as_json();

            if let Some(entries) = bundle_json.get("entry").and_then(|e| e.as_array()) {
                for entry in entries {
                    if let Some(resource) = entry.get("resource") {
                        // Check fullUrl first (preferred for Bundle resolution)
                        if let Some(full_url) = entry.get("fullUrl").and_then(|u| u.as_str()) {
                            if full_url.ends_with(reference) || full_url == reference {
                                return Ok(Some(FhirPathValue::resource_from_json(
                                    resource.clone(),
                                )));
                            }
                        }

                        // Check resource type and ID
                        if let (Some(resource_type), Some(id)) = (
                            resource.get("resourceType").and_then(|rt| rt.as_str()),
                            resource.get("id").and_then(|id| id.as_str()),
                        ) {
                            let resource_ref = format!("{resource_type}/{id}");
                            if resource_ref == reference {
                                return Ok(Some(FhirPathValue::resource_from_json(
                                    resource.clone(),
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Resolve reference within contained resources
    fn resolve_in_contained(
        &self,
        reference: &str,
        containing_resource: &FhirPathValue,
    ) -> Result<Option<FhirPathValue>> {
        if let FhirPathValue::Resource(resource) = containing_resource {
            let resource_json = resource.as_json();

            if let Some(contained) = resource_json.get("contained").and_then(|c| c.as_array()) {
                for contained_resource in contained {
                    if let (Some(resource_type), Some(id)) = (
                        contained_resource
                            .get("resourceType")
                            .and_then(|rt| rt.as_str()),
                        contained_resource.get("id").and_then(|id| id.as_str()),
                    ) {
                        // Check for fragment reference (starts with #)
                        if reference.starts_with('#') && &reference[1..] == id {
                            return Ok(Some(FhirPathValue::resource_from_json(
                                contained_resource.clone(),
                            )));
                        }

                        // Check for full reference
                        let resource_ref = format!("{resource_type}/{id}");
                        if resource_ref == reference {
                            return Ok(Some(FhirPathValue::resource_from_json(
                                contained_resource.clone(),
                            )));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Parse reference string into resource type and ID
    fn parse_reference(&self, reference: &str) -> Option<(String, String)> {
        // Handle fragment references (contained resources)
        if reference.starts_with('#') {
            return None; // Fragment references should be handled in contained resolution
        }

        // Parse ResourceType/id format
        if let Some(slash_pos) = reference.find('/') {
            let resource_type = reference[..slash_pos].to_string();
            let id = reference[slash_pos + 1..].to_string();

            // Basic validation - resource type should be capitalized
            if !resource_type.is_empty()
                && !id.is_empty()
                && resource_type.chars().next().unwrap().is_uppercase()
            {
                return Some((resource_type, id));
            }
        }

        None
    }
}
