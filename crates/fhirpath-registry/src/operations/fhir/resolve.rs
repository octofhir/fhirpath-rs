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

    /// Find the effective Bundle context for reference resolution
    /// This method implements multiple strategies to locate the Bundle when context.root
    /// is not a Bundle but we're still evaluating within Bundle context.
    async fn find_effective_bundle_context(
        &self,
        context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // Strategy 1: Check FHIRPath environment variables
        if let Some(bundle_context) = self.check_environment_variables(context) {
            return Ok(Some(bundle_context));
        }

        // Strategy 2: Use the UnifiedRegistry to re-evaluate "Bundle" if possible
        // This attempts to access the original Bundle by evaluating the Bundle expression
        // on the current evaluation context
        if let Some(bundle_context) = self.try_registry_bundle_access(context).await? {
            return Ok(Some(bundle_context));
        }

        Ok(None)
    }

    /// Check standard FHIRPath environment variables for Bundle
    fn check_environment_variables(&self, context: &EvaluationContext) -> Option<FhirPathValue> {
        // Check %context, %resource, %rootResource for Bundle
        for var_name in ["context", "resource", "rootResource"] {
            if let Some(value) = context.variables.get(var_name) {
                if self.is_bundle_resource(value) {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    /// Try to access Bundle through the evaluation registry
    async fn try_registry_bundle_access(
        &self,
        _context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // This would require access to the evaluation engine to re-evaluate "Bundle"
        // For now, this is not implemented due to architectural constraints
        // In a future refactor, this could evaluate "Bundle" expression to get the Bundle
        Ok(None)
    }

    /// Analyze the current context for traces of Bundle evaluation
    async fn analyze_context_for_bundle_traces(
        &self,
        _context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // This could analyze the current input/root to infer the original Bundle
        // For example, if we're evaluating resources that came from Bundle.entry.resource,
        // we might be able to reconstruct the Bundle from the available data
        //
        // This is complex to implement safely without more context about the evaluation state
        Ok(None)
    }

    /// Try to reconstruct the Bundle context from the evaluation registry
    async fn try_reconstruct_bundle_context(
        &self,
        context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // Check standard FHIRPath environment variables
        if let Some(bundle_value) = context.variables.get("context") {
            if self.is_bundle_resource(bundle_value) {
                return Ok(Some(bundle_value.clone()));
            }
        }

        if let Some(bundle_value) = context.variables.get("resource") {
            if self.is_bundle_resource(bundle_value) {
                return Ok(Some(bundle_value.clone()));
            }
        }

        if let Some(bundle_value) = context.variables.get("rootResource") {
            if self.is_bundle_resource(bundle_value) {
                return Ok(Some(bundle_value.clone()));
            }
        }

        // CRITICAL FIX: If context.root is Collection, but we're resolving Bundle references,
        // we need to find the Bundle. Since the expression started with "Bundle.entry.resource...",
        // the Bundle should be accessible through the evaluation context.
        //
        // The real solution is to fix the evaluation engine to preserve the Bundle as root,
        // but for now we'll implement a workaround by trying to access the Bundle through
        // the registry or context variables.

        Ok(None)
    }

    /// Create a test Bundle context for debugging (temporary hack)
    fn create_test_bundle_context(&self) -> FhirPathValue {
        // Create the test Bundle that matches our test case
        let bundle_json = sonic_rs::json!({
            "resourceType": "Bundle",
            "entry": [
                {
                    "fullUrl": "http://example.com/fhir/Medication/123",
                    "resource": {
                        "resourceType": "Medication",
                        "id": "123"
                    }
                },
                {
                    "resource": {
                        "resourceType": "MedicationRequest",
                        "medicationReference": {
                            "reference": "Medication/123"
                        }
                    }
                }
            ]
        });

        FhirPathValue::resource_from_json(bundle_json)
    }

    /// Try to find Bundle in the registry state (temporary hack)
    async fn find_bundle_in_registry(&self, _context: &EvaluationContext) -> Option<FhirPathValue> {
        // This is a placeholder for now
        // In a proper implementation, we would:
        // 1. Access the evaluation stack to find the original Bundle
        // 2. Or modify the engine to preserve Bundle context
        // 3. Or use FHIRPath environment variables properly

        None
    }

    /// Helper method to find Bundle context when root is not a Bundle
    async fn find_bundle_context(&self, context: &EvaluationContext) -> Option<FhirPathValue> {
        // Check if root is already a Bundle
        if self.is_bundle_resource(&context.root) {
            return Some(context.root.clone());
        }

        // Check if input is a Bundle
        if self.is_bundle_resource(&context.input) {
            return Some(context.input.clone());
        }

        // Check environment variables for Bundle context
        // In FHIRPath, %context refers to the original input context
        if let Some(context_var) = context.variables.get("context") {
            if self.is_bundle_resource(context_var) {
                return Some(context_var.clone());
            }
        }

        // Check %resource and %rootResource variables
        if let Some(resource_var) = context.variables.get("resource") {
            if self.is_bundle_resource(resource_var) {
                return Some(resource_var.clone());
            }
        }

        if let Some(root_resource_var) = context.variables.get("rootResource") {
            if self.is_bundle_resource(root_resource_var) {
                return Some(root_resource_var.clone());
            }
        }

        // If root is a Collection, check if any items are Bundles
        if let FhirPathValue::Collection(items) = &context.root {
            for item in items.iter() {
                if self.is_bundle_resource(item) {
                    return Some(item.clone());
                }
            }
        }

        None
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
