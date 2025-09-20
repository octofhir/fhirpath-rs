//! ConformsTo function implementation
//!
//! The conformsTo function checks if a resource conforms to a specified profile.
//! Syntax: resource.conformsTo(profile_url)
//!
//! This function uses the ValidationProvider to perform profile validation,
//! breaking the circular dependency between ModelProvider and FhirPathEvaluator.

use std::sync::Arc;

use crate::core::{FP0063, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// ConformsTo function evaluator
pub struct ConformsToFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConformsToFunctionEvaluator {
    /// Create a new conformsTo function evaluator
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "conformsTo".to_string(),
                description: "Tests if the input resource conforms to a specified profile"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Resource".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "profile".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The canonical URL of the profile to validate against"
                            .to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: true, // Requires model for ValidationProvider access
            },
        })
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for ConformsToFunctionEvaluator {
    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Validate we have exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "conformsTo() requires exactly one argument".to_string(),
            ));
        }

        // FHIR spec: Return empty if input is not a single item
        if input.len() != 1 {
            return Ok(EvaluationResult::new(crate::core::Collection::empty()));
        }

        // Extract profile URL as string from pre-evaluated argument
        let profile_arg = &args[0];
        if profile_arg.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "conformsTo() profile argument must be a single string".to_string(),
            ));
        }

        let profile_url = match &profile_arg[0] {
            FhirPathValue::String(url, _, _) => {
                // FHIR spec: Return empty if structure is empty
                if url.trim().is_empty() {
                    return Ok(EvaluationResult::new(crate::core::Collection::empty()));
                }
                url.clone()
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "conformsTo() profile argument must be a string".to_string(),
                ));
            }
        };

        // Check conformance for the single input item
        match self
            .check_conformance(context, &input[0], &profile_url)
            .await
        {
            Ok(Some(conforms)) => {
                // Successful validation
                Ok(EvaluationResult::new(crate::core::Collection::from_iter(
                    vec![FhirPathValue::boolean(conforms)],
                )))
            }
            Ok(None) => {
                // FHIR spec: Return empty if structure cannot be resolved to valid profile
                Ok(EvaluationResult::new(crate::core::Collection::empty()))
            }
            Err(err) => {
                // System error - propagate
                Err(err)
            }
        }
    }
}

impl ConformsToFunctionEvaluator {
    /// Check if a value conforms to the specified profile
    /// Returns:
    /// - Ok(Some(true/false)) for successful validation
    /// - Ok(None) for profile not found or validation not available
    /// - Err for system errors
    async fn check_conformance(
        &self,
        context: &EvaluationContext,
        value: &FhirPathValue,
        profile_url: &str,
    ) -> Result<Option<bool>> {
        // Get the ValidationProvider from the context
        let validation_provider = match context.validation_provider() {
            Some(provider) => provider,
            None => {
                // If no ValidationProvider is available, return None (empty result)
                // This allows graceful degradation in environments without full validation
                return Ok(None);
            }
        };

        // Convert FhirPathValue to JSON for validation
        let json_value = match value {
            FhirPathValue::Resource(json, _, _) => (**json).clone(),
            _ => {
                // Non-resource values cannot conform to profiles - return None (empty)
                return Ok(None);
            }
        };

        // Check for obviously invalid URLs first
        if !profile_url.starts_with("http://") && !profile_url.starts_with("https://") {
            return Err(FhirPathError::evaluation_error(
                FP0063,
                format!("Invalid profile URL: {profile_url}"),
            ));
        }

        // Use ValidationProvider to check conformance
        match validation_provider.validate(&json_value, profile_url).await {
            Ok(conforms) => Ok(Some(conforms)),
            Err(err) => {
                // For invalid/unknown profiles, return error instead of empty
                Err(FhirPathError::evaluation_error(
                    FP0063,
                    format!("Profile validation failed: {err}"),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;
    use serde_json::json;

    #[tokio::test]
    async fn test_conforms_to_function_metadata() {
        let function = ConformsToFunctionEvaluator::create();
        let metadata = function.metadata();

        assert_eq!(metadata.name, "conformsTo");
        assert_eq!(metadata.signature.parameters.len(), 1);
        assert_eq!(metadata.signature.parameters[0].name, "profile");
        assert_eq!(metadata.signature.return_type, "Boolean");
        assert!(metadata.requires_model);
    }

    async fn create_test_context() -> EvaluationContext {
        EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None, // No terminology provider
            None, // No validation provider
            None, // No trace provider
        )
        .await
    }

    #[tokio::test]
    async fn test_conforms_to_without_validation_provider() {
        let function = ConformsToFunctionEvaluator::create();
        let context = create_test_context().await;

        // Create a resource input
        let resource = json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [{"family": "Doe", "given": ["John"]}]
        });
        let input_value = FhirPathValue::resource(resource);
        let input_values = vec![input_value];

        // Create pre-evaluated profile argument
        let profile_url =
            FhirPathValue::string("http://example.org/StructureDefinition/MyPatient".to_string());
        let args = vec![vec![profile_url]];

        // Evaluate function - should return empty when no ValidationProvider
        let result = function
            .evaluate(input_values, args, &context)
            .await
            .expect("Function evaluation should succeed");

        let values: Vec<_> = result.value.iter().collect();
        assert_eq!(values.len(), 0); // Empty result per FHIR spec
    }

    #[tokio::test]
    async fn test_conforms_to_empty_input() {
        let function = ConformsToFunctionEvaluator::create();
        let context = create_test_context().await;

        // Empty input collection
        let input_values = vec![];

        // Create pre-evaluated profile argument
        let profile_url =
            FhirPathValue::string("http://example.org/StructureDefinition/MyPatient".to_string());
        let args = vec![vec![profile_url]];

        // Evaluate function - should return empty for non-single input
        let result = function
            .evaluate(input_values, args, &context)
            .await
            .expect("Function evaluation should succeed");

        let values: Vec<_> = result.value.iter().collect();
        assert_eq!(values.len(), 0); // Empty result per FHIR spec
    }

    #[tokio::test]
    async fn test_conforms_to_multiple_input() {
        let function = ConformsToFunctionEvaluator::create();
        let context = create_test_context().await;

        // Multiple input items
        let resource1 = json!({"resourceType": "Patient", "id": "1"});
        let resource2 = json!({"resourceType": "Patient", "id": "2"});
        let input_values = vec![
            FhirPathValue::resource(resource1),
            FhirPathValue::resource(resource2),
        ];

        // Create pre-evaluated profile argument
        let profile_url =
            FhirPathValue::string("http://example.org/StructureDefinition/MyPatient".to_string());
        let args = vec![vec![profile_url]];

        // Evaluate function - should return empty for non-single input
        let result = function
            .evaluate(input_values, args, &context)
            .await
            .expect("Function evaluation should succeed");

        let values: Vec<_> = result.value.iter().collect();
        assert_eq!(values.len(), 0); // Empty result per FHIR spec
    }

    #[tokio::test]
    async fn test_conforms_to_empty_profile_url() {
        let function = ConformsToFunctionEvaluator::create();
        let context = create_test_context().await;

        // Create a resource input
        let resource = json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [{"family": "Doe", "given": ["John"]}]
        });
        let input_value = FhirPathValue::resource(resource);
        let input_values = vec![input_value];

        // Create pre-evaluated empty profile argument
        let profile_url = FhirPathValue::string("".to_string());
        let args = vec![vec![profile_url]];

        // Evaluate function - should return empty for empty structure
        let result = function
            .evaluate(input_values, args, &context)
            .await
            .expect("Function evaluation should succeed");

        let values: Vec<_> = result.value.iter().collect();
        assert_eq!(values.len(), 0); // Empty result per FHIR spec
    }
}
