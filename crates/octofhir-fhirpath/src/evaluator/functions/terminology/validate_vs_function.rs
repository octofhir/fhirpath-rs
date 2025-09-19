//! ValidateVS function implementation
//!
//! The validateVS function validates a code against a value set.
//! This function requires a terminology provider to perform the validation.
//! Syntax: code.validateVS(valueSet) or validateVS(code, valueSetUrl)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use octofhir_fhir_model::TerminologyProvider;

/// ValidateVS function evaluator
pub struct ValidateVSFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ValidateVSFunctionEvaluator {
    /// Create a new validateVS function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "validateVS".to_string(),
                description: "Validates a code against a value set. Returns validation results as Parameters.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "valueSet".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Value set URL or value set resource to validate against".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Parameters".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Terminology operations may change over time
                category: FunctionCategory::Utility,
                requires_terminology: true,
                requires_model: false,
            },
        })
    }

    /// Extract coding information from input
    fn extract_coding_info(input: &[FhirPathValue]) -> Result<(Option<String>, String)> {
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "validateVS function requires a single input value".to_string(),
            ));
        }

        match &input[0] {
            FhirPathValue::Resource(resource, _, _) => {
                let system = resource
                    .get("system")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                let code = resource
                    .get("code")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "Coding resource must have a code field".to_string(),
                        )
                    })?;

                Ok((system, code.to_string()))
            }
            FhirPathValue::String(code, _, _) => {
                // Just a code string without system
                Ok((None, code.clone()))
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                "validateVS function can only be called on Coding resources or code strings"
                    .to_string(),
            )),
        }
    }

    /// Extract value set URL from parameter
    async fn get_value_set_url(
        arg: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<String> {
        let result = evaluator.evaluate(arg, context).await?;
        let values: Vec<FhirPathValue> = result.value.iter().cloned().collect();

        if values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "validateVS function valueSet parameter must evaluate to a single value"
                    .to_string(),
            ));
        }

        match &values[0] {
            FhirPathValue::String(url, _, _) => Ok(url.clone()),
            FhirPathValue::Resource(resource, _, _) => {
                // Extract URL from ValueSet resource
                resource
                    .get("url")
                    .and_then(|u| u.as_str())
                    .map(|u| u.to_string())
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "ValueSet resource must have a url field".to_string(),
                        )
                    })
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                "validateVS function valueSet parameter must be a string URL or ValueSet resource"
                    .to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for ValidateVSFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "validateVS function requires exactly one argument (valueSet)".to_string(),
            ));
        }

        // Get terminology provider
        let terminology_provider = context.terminology_provider().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "validateVS function requires a terminology provider".to_string(),
            )
        })?;

        // Extract coding information
        let (system, code) = Self::extract_coding_info(&input)?;

        // Get value set URL
        let value_set_url = Self::get_value_set_url(&args[0], context, evaluator).await?;

        // Perform validation
        match terminology_provider
            .validate_code_vs(&value_set_url, system.as_deref(), &code, None)
            .await
        {
            Ok(validation_result) => {
                // Convert validation result to FHIR Parameters resource structure
                let mut parameters = serde_json::Map::new();
                parameters.insert(
                    "resourceType".to_string(),
                    serde_json::Value::String("Parameters".to_string()),
                );

                let mut parameter_list = Vec::new();

                // Add result parameter
                let mut result_param = serde_json::Map::new();
                result_param.insert(
                    "name".to_string(),
                    serde_json::Value::String("result".to_string()),
                );
                result_param.insert(
                    "valueBoolean".to_string(),
                    serde_json::Value::Bool(validation_result.result),
                );
                parameter_list.push(serde_json::Value::Object(result_param));

                // Add display parameter if available
                if let Some(display) = validation_result.display {
                    let mut display_param = serde_json::Map::new();
                    display_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("display".to_string()),
                    );
                    display_param.insert(
                        "valueString".to_string(),
                        serde_json::Value::String(display),
                    );
                    parameter_list.push(serde_json::Value::Object(display_param));
                }

                // Add message parameter if available
                if let Some(message) = validation_result.message {
                    let mut message_param = serde_json::Map::new();
                    message_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("message".to_string()),
                    );
                    message_param.insert(
                        "valueString".to_string(),
                        serde_json::Value::String(message),
                    );
                    parameter_list.push(serde_json::Value::Object(message_param));
                }

                parameters.insert(
                    "parameter".to_string(),
                    serde_json::Value::Array(parameter_list),
                );

                let parameters_value =
                    FhirPathValue::resource(serde_json::Value::Object(parameters));

                Ok(EvaluationResult {
                    value: crate::core::Collection::from(vec![parameters_value]),
                })
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0059,
                format!("Code validation against value set failed: {}", e),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
