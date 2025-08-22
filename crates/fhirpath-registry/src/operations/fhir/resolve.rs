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
use sonic_rs::JsonValueTrait;

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
                // Handle Reference objects - extract reference field
                if let Some(reference_val) = json.as_inner().get("reference") {
                    if let Some(reference) = reference_val.as_str() {
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
    /// This method now delegates to ModelProvider's enhanced resolution
    async fn resolve_local_reference(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // Determine the correct resource context for resolution
        // The resolve() function can find resources in Bundle entries OR contained resources
        // So we need to check multiple potential contexts in order of preference:

        let resource_context = if self.can_contain_resources(&context.root) {
            // Root can contain resources (Bundle, or resource with contained), use it directly
            &context.root
        } else {
            // Root doesn't contain resources, check FHIRPath environment variables
            // These variables should contain the original resource context
            if let Some(context_var) = context.get_variable("context") {
                if self.can_contain_resources(context_var) {
                    context_var
                } else {
                    &context.root
                }
            } else if let Some(resource_var) = context.get_variable("resource") {
                if self.can_contain_resources(resource_var) {
                    resource_var
                } else {
                    &context.root
                }
            } else if let Some(root_resource_var) = context.get_variable("rootResource") {
                if self.can_contain_resources(root_resource_var) {
                    root_resource_var
                } else {
                    &context.root
                }
            } else {
                &context.root
            }
        };

        // Use ModelProvider for all resolution logic with the correct resource context
        if let Some(resolved) = context
            .model_provider
            .resolve_reference_in_context(reference, resource_context, Some(&context.input))
            .await
        {
            return Ok(Some(resolved));
        }

        Ok(None)
    }

    /// Helper method to check if a resource is a Bundle
    fn is_bundle_resource(&self, resource: &FhirPathValue) -> bool {
        match resource {
            FhirPathValue::Resource(res) => res.resource_type() == Some("Bundle"),
            FhirPathValue::JsonValue(json) => json
                .as_inner()
                .get("resourceType")
                .and_then(|rt| rt.as_str())
                .map(|rt| rt == "Bundle")
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Check if a resource can potentially contain other resources for resolution
    /// This includes Bundle resources (via entry.resource) and any resource with contained property
    fn can_contain_resources(&self, resource: &FhirPathValue) -> bool {
        // Bundle resources can contain resources via entries
        if self.is_bundle_resource(resource) {
            return true;
        }

        // Any FHIR resource can potentially have contained resources
        match resource {
            FhirPathValue::Resource(_) => true,
            FhirPathValue::JsonValue(json) => {
                // Check if it looks like a FHIR resource with resourceType
                json.as_inner()
                    .get("resourceType")
                    .and_then(|rt| rt.as_str())
                    .is_some()
            }
            _ => false,
        }
    }

    /// Try to resolve reference using ModelProvider for external resolution
    async fn resolve_external_reference(
        &self,
        reference: &str,
        context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // Use ModelProvider's enhanced reference resolution
        if let Some(resolved) = context
            .model_provider
            .resolve_reference_in_context(reference, &context.root, Some(&context.input))
            .await
        {
            return Ok(Some(resolved));
        }

        // If resolution failed, return None
        Ok(None)
    }
}
