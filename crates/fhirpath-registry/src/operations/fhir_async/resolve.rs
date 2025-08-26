//! Resolve function implementation - async version (simplified)

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{AsyncOperation, EvaluationContext, validation};
use async_trait::async_trait;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;
use sonic_rs::JsonValueTrait;

/// Resolve function - resolves FHIR references using ModelProvider
#[derive(Debug, Clone)]
pub struct ResolveFunction;

impl ResolveFunction {
    pub fn new() -> Self {
        Self
    }

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

        // Use ModelProvider for reference resolution
        // Use the root context (Bundle) for reference resolution, not the current input (Reference)
        if let Some(resolved) = context
            .model_provider
            .resolve_reference_in_context(reference, &context.root, Some(&context.input))
            .await
        {
            return Ok(resolved);
        }

        // Return empty if not found
        Ok(FhirPathValue::Empty)
    }
}

#[async_trait]
impl AsyncOperation for ResolveFunction {
    fn name(&self) -> &'static str {
        "resolve"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "resolve",
                parameters: vec![],
                return_type: ValueType::Any,
                variadic: false,
            });
        &SIGNATURE
    }

    async fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "resolve")?;

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
                    self.execute(args, &item_context).await
                } else {
                    // Resolve each reference in the collection
                    let mut resolved = Vec::new();
                    for item in c.iter() {
                        let item_context = context.with_input(item.clone());
                        let result = self.execute(args, &item_context).await?;
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
}

impl Default for ResolveFunction {
    fn default() -> Self {
        Self::new()
    }
}
