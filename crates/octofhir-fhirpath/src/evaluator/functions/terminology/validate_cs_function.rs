//! ValidateCS function implementation
//!
//! The validateCS function validates a code against a code system.
//! This function requires a terminology provider to perform the validation.
//! Syntax: code.validateCS(codeSystem) or validateCS(code, systemUrl)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use octofhir_fhir_model::TerminologyProvider;

/// ValidateCS function evaluator
pub struct ValidateCSFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ValidateCSFunctionEvaluator {
    /// Create a new validateCS function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "validateCS".to_string(),
                description: "Validates a code against a code system. Returns validation results as Parameters.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "codeSystem".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Code system URL or code system resource to validate against".to_string(),
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

    /// Extract code from input
    fn extract_code(input: &[FhirPathValue]) -> Result<String> {
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "validateCS function requires a single input value".to_string(),
            ));
        }

        match &input[0] {
            FhirPathValue::Resource(resource, _, _) => resource
                .get("code")
                .and_then(|c| c.as_str())
                .map(|c| c.to_string())
                .ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "Coding resource must have a code field".to_string(),
                    )
                }),
            FhirPathValue::String(code, _, _) => Ok(code.clone()),
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                "validateCS function can only be called on Coding resources or code strings"
                    .to_string(),
            )),
        }
    }

    /// Extract code system URL from parameter
    async fn get_code_system_url(
        arg: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<String> {
        let result = evaluator.evaluate(arg, context).await?;
        let values: Vec<FhirPathValue> = result.value.iter().cloned().collect();

        if values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "validateCS function codeSystem parameter must evaluate to a single value"
                    .to_string(),
            ));
        }

        match &values[0] {
            FhirPathValue::String(url, _, _) => Ok(url.clone()),
            FhirPathValue::Resource(resource, _, _) => {
                // Extract URL from CodeSystem resource
                resource.get("url")
                    .and_then(|u| u.as_str())
                    .map(|u| u.to_string())
                    .ok_or_else(|| FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "CodeSystem resource must have a url field".to_string(),
                    ))
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                "validateCS function codeSystem parameter must be a string URL or CodeSystem resource".to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for ValidateCSFunctionEvaluator {
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
                "validateCS function requires exactly one argument (codeSystem)".to_string(),
            ));
        }

        // Get terminology provider
        let terminology_provider = context.terminology_provider().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "validateCS function requires a terminology provider".to_string(),
            )
        })?;

        // Extract code
        let code = Self::extract_code(&input)?;

        // Get code system URL
        let code_system_url = Self::get_code_system_url(&args[0], context, evaluator).await?;

        // Perform validation using lookup (code systems don't have a separate validate operation)
        match terminology_provider
            .lookup_code(&code_system_url, &code, None, None)
            .await
        {
            Ok(lookup_result) => {
                // Convert lookup result to validation result
                // If lookup succeeds, the code is valid in the code system
                let mut validation_result = serde_json::Map::new();
                validation_result.insert(
                    "resourceType".to_string(),
                    serde_json::Value::String("Parameters".to_string()),
                );

                let mut parameters = Vec::new();

                // Add result parameter
                let mut result_param = serde_json::Map::new();
                result_param.insert(
                    "name".to_string(),
                    serde_json::Value::String("result".to_string()),
                );
                result_param.insert("valueBoolean".to_string(), serde_json::Value::Bool(true));
                parameters.push(serde_json::Value::Object(result_param));

                // Add display parameter if available from lookup
                if let Some(display) = &lookup_result.display {
                    let mut display_param = serde_json::Map::new();
                    display_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("display".to_string()),
                    );
                    display_param.insert(
                        "valueString".to_string(),
                        serde_json::Value::String(display.clone()),
                    );
                    parameters.push(serde_json::Value::Object(display_param));
                }

                validation_result.insert(
                    "parameter".to_string(),
                    serde_json::Value::Array(parameters),
                );

                let parameters_value =
                    FhirPathValue::resource(serde_json::Value::Object(validation_result));

                Ok(EvaluationResult {
                    value: crate::core::Collection::from(vec![parameters_value]),
                })
            }
            Err(_) => {
                // Code not found in code system
                let mut validation_result = serde_json::Map::new();
                validation_result.insert(
                    "resourceType".to_string(),
                    serde_json::Value::String("Parameters".to_string()),
                );

                let mut result_param = serde_json::Map::new();
                result_param.insert(
                    "name".to_string(),
                    serde_json::Value::String("result".to_string()),
                );
                result_param.insert("valueBoolean".to_string(), serde_json::Value::Bool(false));

                validation_result.insert(
                    "parameter".to_string(),
                    serde_json::Value::Array(vec![serde_json::Value::Object(result_param)]),
                );

                let parameters_value =
                    FhirPathValue::resource(serde_json::Value::Object(validation_result));

                Ok(EvaluationResult {
                    value: crate::core::Collection::from(vec![parameters_value]),
                })
            }
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
