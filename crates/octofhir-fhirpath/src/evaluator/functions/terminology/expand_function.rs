//! Expand function implementation
//!
//! The expand function expands value sets to get all contained concepts.
//! This function requires a terminology provider to perform the expansion.
//! Syntax: valueSet.expand() or expand(valueSetUrl)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::terminologies_variable::{
    extract_terminology_provider_from_terminologies_variable, is_terminologies_variable,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Expand function evaluator
pub struct ExpandFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ExpandFunctionEvaluator {
    /// Create a new expand function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "expand".to_string(),
                description: "Expands a value set to return all contained concepts. Returns a collection of Coding values.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "valueSetUrl".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Optional value set URL. If not provided, uses the input as the value set reference.".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Coding".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(1),
                },
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Terminology operations may change over time
                category: FunctionCategory::Utility,
                requires_terminology: true,
                requires_model: false,
            },
        })
    }

    /// Extract value set URL from input or parameter
    async fn get_value_set_url(
        input: &[FhirPathValue],
        args: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<String> {
        // Check if this is being called on %terminologies variable
        let is_terminologies_call = input.len() == 1 && is_terminologies_variable(&input[0]);

        if is_terminologies_call {
            // When called on %terminologies, the first argument must be the value set URL
            if args.is_empty() {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "expand function called on %terminologies requires valueSetUrl parameter"
                        .to_string(),
                ));
            }

            let url_result = evaluator.evaluate(&args[0], context).await?;
            let url_values: Vec<FhirPathValue> = url_result.value.iter().cloned().collect();

            if url_values.len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "expand function valueSetUrl parameter must evaluate to a single value"
                        .to_string(),
                ));
            }

            match &url_values[0] {
                FhirPathValue::String(url, _, _) => Ok(url.clone()),
                _ => Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "expand function valueSetUrl parameter must be a string".to_string(),
                )),
            }
        } else if args.len() == 1 {
            // Value set URL provided as parameter
            let url_result = evaluator.evaluate(&args[0], context).await?;
            let url_values: Vec<FhirPathValue> = url_result.value.iter().cloned().collect();

            if url_values.len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "expand function valueSetUrl parameter must evaluate to a single value"
                        .to_string(),
                ));
            }

            match &url_values[0] {
                FhirPathValue::String(url, _, _) => Ok(url.clone()),
                _ => Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "expand function valueSetUrl parameter must be a string".to_string(),
                )),
            }
        } else {
            // Extract URL from input value set
            if input.len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    "expand function requires a single value set input when no URL parameter is provided".to_string(),
                ));
            }

            match &input[0] {
                FhirPathValue::String(url, _, _) => Ok(url.clone()),
                FhirPathValue::Resource(resource, _, _) => {
                    // Extract URL from ValueSet resource
                    if let Some(url_value) = resource.get("url") {
                        if let Some(url_str) = url_value.as_str() {
                            Ok(url_str.to_string())
                        } else {
                            Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0058,
                                "ValueSet resource url field is not a string".to_string(),
                            ))
                        }
                    } else {
                        Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "ValueSet resource must have a url field".to_string(),
                        ))
                    }
                }
                _ => Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "expand function can only be called on ValueSet resources or value set URLs"
                        .to_string(),
                )),
            }
        }
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for ExpandFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "expand function takes at most one argument (valueSetUrl)".to_string(),
            ));
        }

        // Get terminology provider - either from context or from %terminologies variable
        let terminology_provider = if input.len() == 1 && is_terminologies_variable(&input[0]) {
            // When called on %terminologies variable, extract provider from it
            extract_terminology_provider_from_terminologies_variable(&input[0], context)
                .ok_or_else(|| FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    "expand function called on %terminologies but no terminology provider available".to_string(),
                ))?
        } else {
            // Standard call - get provider from context
            context.terminology_provider().ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    "expand function requires a terminology provider".to_string(),
                )
            })?
        };

        // Get value set URL
        let value_set_url = Self::get_value_set_url(&input, &args, context, evaluator).await?;

        // Perform value set expansion
        match terminology_provider
            .expand_valueset(&value_set_url, None)
            .await
        {
            Ok(expansion_result) => {
                // Convert expansion result to FhirPathValue collection
                let mut codings = Vec::new();

                // Create Coding FhirPathValues for each concept in the expansion
                for concept in expansion_result.contains {
                    // Create a Coding resource from the concept
                    let mut coding = serde_json::Map::new();

                    if let Some(system) = concept.system {
                        coding.insert("system".to_string(), serde_json::Value::String(system));
                    }

                    coding.insert("code".to_string(), serde_json::Value::String(concept.code));

                    if let Some(display) = concept.display {
                        coding.insert("display".to_string(), serde_json::Value::String(display));
                    }

                    codings.push(FhirPathValue::resource(serde_json::Value::Object(coding)));
                }

                Ok(EvaluationResult {
                    value: crate::core::Collection::from(codings),
                })
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0059,
                format!("Value set expansion failed: {}", e),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
