//! ValidateVS function implementation
//!
//! The validateVS function validates a code against a value set.
//! This function requires a terminology provider to perform the validation.
//! Syntax: code.validateVS(valueSet) or validateVS(code, valueSetUrl)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

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
                            is_expression: true,
                            description: "Value set URL or value set resource to validate against".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "code".to_string(),
                            parameter_type: vec!["Coding".to_string(), "CodeableConcept".to_string(), "String".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Code to validate (optional for static function call)".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Parameters".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(2),
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
        if input.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "validateVS function requires at least one input value".to_string(),
            ));
        }

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
        evaluator: &AsyncNodeEvaluator<'_>,
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
        // Check if this is called on %terminologies variable
        let is_on_terminologies = input.len() == 1
            && crate::evaluator::terminologies_variable::is_terminologies_variable(&input[0]);

        if is_on_terminologies {
            // When called on %terminologies, expect 2 arguments: (valueSet, code)
            if args.len() != 2 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "validateVS function on %terminologies requires exactly two arguments (valueSet, code)".to_string(),
                ));
            }

            // Get terminology provider from context
            let terminology_provider = context.terminology_provider().ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    "validateVS function requires a terminology provider".to_string(),
                )
            })?;

            // Get value set URL from first argument
            let value_set_url = Self::get_value_set_url(&args[0], context, &evaluator).await?;

            // Get code from second argument
            let code_result = evaluator.evaluate(&args[1], context).await?;
            let code_values: Vec<FhirPathValue> = code_result.value.iter().cloned().collect();
            let (system, code) = Self::extract_coding_info(&code_values)?;

            // First try a robust, server-independent approach: expand the ValueSet and check membership locally
            let mut validation_result_opt: Option<
                octofhir_fhir_model::terminology::ValidationResult,
            > = None;
            if let Ok(expansion) = terminology_provider
                .expand_valueset(&value_set_url, None)
                .await
            {
                let mut found = false;
                let mut display: Option<String> = None;
                for concept in expansion.contains.iter() {
                    if concept.code == code
                        && (system.is_none() || concept.system.as_deref() == system.as_deref())
                    {
                        found = true;
                        display = concept.display.clone();
                        break;
                    }
                }
                validation_result_opt = Some(octofhir_fhir_model::terminology::ValidationResult {
                    result: found,
                    message: None,
                    display,
                });
            }

            // If expansion-based membership check didn't yield true, try provider's validate endpoint
            if !validation_result_opt
                .as_ref()
                .map(|r| r.result)
                .unwrap_or(false)
            {
                validation_result_opt = terminology_provider
                    .validate_code_vs(&code, system.as_deref(), &value_set_url, None)
                    .await
                    .ok();

                // If still not true and no system provided, try inferring system from the ValueSet URL
                if validation_result_opt.as_ref().map(|r| r.result) != Some(true)
                    && system.is_none()
                    && value_set_url.starts_with("http://hl7.org/fhir/ValueSet/")
                {
                    let tail = value_set_url.trim_start_matches("http://hl7.org/fhir/ValueSet/");
                    let inferred_system = format!("http://hl7.org/fhir/{tail}");
                    if let Ok(r2) = terminology_provider
                        .validate_code_vs(
                            &code,
                            Some(inferred_system.as_str()),
                            &value_set_url,
                            None,
                        )
                        .await
                    {
                        validation_result_opt = Some(r2);
                    }
                }
            }

            if let Some(validation_result) = validation_result_opt {
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
                // Use generic 'value' key so FHIRPath '.value' works without schema-aware choice handling
                result_param.insert(
                    "value".to_string(),
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
                    display_param.insert("value".to_string(), serde_json::Value::String(display));
                    parameter_list.push(serde_json::Value::Object(display_param));
                }

                // Add message parameter if available
                if let Some(message) = validation_result.message {
                    let mut message_param = serde_json::Map::new();
                    message_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("message".to_string()),
                    );
                    message_param.insert("value".to_string(), serde_json::Value::String(message));
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
            } else {
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0059,
                    "Code validation against value set failed".to_string(),
                ))
            }
        } else {
            // Enforce %terminologies usage only
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                "validateVS function must be called on %terminologies".to_string(),
            ));
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
