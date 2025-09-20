//! Lookup function implementation
//!
//! The lookup function performs concept lookup to get details about a specific concept.
//! This function requires a terminology provider to perform the lookup.
//! Syntax: concept.lookup() or lookup(system, code)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Lookup function evaluator
pub struct LookupFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LookupFunctionEvaluator {
    /// Create a new lookup function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "lookup".to_string(),
                description: "Performs concept lookup to get detailed information about a concept. Returns concept details.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "system".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Code system URL. Required when input is not a Coding.".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "code".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Code value. Required when input is not a Coding.".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Parameters".to_string(),
                    polymorphic: false,
                    min_params: 0,
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

    /// Extract system and code from input or parameters
    async fn get_system_and_code(
        input: &[FhirPathValue],
        args: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<(String, String)> {
        match args.len() {
            0 => {
                // Extract from input Coding
                if input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        "lookup function requires a single Coding input when no parameters are provided".to_string(),
                    ));
                }

                match &input[0] {
                    FhirPathValue::Resource(resource, _, _) => {
                        let system = resource.get("system")
                            .and_then(|s| s.as_str())
                            .ok_or_else(|| FhirPathError::evaluation_error(
                                crate::core::error_code::FP0058,
                                "Coding resource must have a system field".to_string(),
                            ))?;

                        let code = resource.get("code")
                            .and_then(|c| c.as_str())
                            .ok_or_else(|| FhirPathError::evaluation_error(
                                crate::core::error_code::FP0058,
                                "Coding resource must have a code field".to_string(),
                            ))?;

                        Ok((system.to_string(), code.to_string()))
                    }
                    _ => Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        "lookup function requires a Coding input when no parameters are provided".to_string(),
                    )),
                }
            }
            2 => {
                // System and code provided as parameters
                let system_result = evaluator.evaluate(&args[0], context).await?;
                let system_values: Vec<FhirPathValue> = system_result.value.iter().cloned().collect();

                if system_values.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0056,
                        "lookup function system parameter must evaluate to a single value".to_string(),
                    ));
                }

                let system = match &system_values[0] {
                    FhirPathValue::String(s, _, _) => s.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "lookup function system parameter must be a string".to_string(),
                    )),
                };

                let code_result = evaluator.evaluate(&args[1], context).await?;
                let code_values: Vec<FhirPathValue> = code_result.value.iter().cloned().collect();

                if code_values.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0056,
                        "lookup function code parameter must evaluate to a single value".to_string(),
                    ));
                }

                let code = match &code_values[0] {
                    FhirPathValue::String(c, _, _) => c.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "lookup function code parameter must be a string".to_string(),
                    )),
                };

                Ok((system, code))
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "lookup function takes either no arguments (for Coding input) or two arguments (system, code)".to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for LookupFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "lookup function takes at most two arguments (system, code)".to_string(),
            ));
        }

        // Get terminology provider
        let terminology_provider = context.terminology_provider().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "lookup function requires a terminology provider".to_string(),
            )
        })?;

        // Get system and code
        let (system, code) = Self::get_system_and_code(&input, &args, context, evaluator).await?;

        // Perform concept lookup
        match terminology_provider
            .lookup_code(&system, &code, None, None)
            .await
        {
            Ok(lookup_result) => {
                // Convert lookup result to FHIR Parameters resource structure
                let mut parameters = serde_json::Map::new();
                parameters.insert(
                    "resourceType".to_string(),
                    serde_json::Value::String("Parameters".to_string()),
                );

                let mut parameter_list = Vec::new();

                // Add display parameter if available
                if let Some(display) = lookup_result.display {
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

                // Add definition parameter if available
                if let Some(definition) = lookup_result.definition {
                    let mut definition_param = serde_json::Map::new();
                    definition_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("definition".to_string()),
                    );
                    definition_param.insert(
                        "valueString".to_string(),
                        serde_json::Value::String(definition),
                    );
                    parameter_list.push(serde_json::Value::Object(definition_param));
                }

                // Add property parameters
                for property in lookup_result.properties {
                    let mut prop_param = serde_json::Map::new();
                    prop_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("property".to_string()),
                    );

                    let mut prop_parts = Vec::new();

                    let mut code_part = serde_json::Map::new();
                    code_part.insert(
                        "name".to_string(),
                        serde_json::Value::String("code".to_string()),
                    );
                    code_part.insert(
                        "valueCode".to_string(),
                        serde_json::Value::String(property.code),
                    );
                    prop_parts.push(serde_json::Value::Object(code_part));

                    let mut value_part = serde_json::Map::new();
                    value_part.insert(
                        "name".to_string(),
                        serde_json::Value::String("value".to_string()),
                    );
                    value_part.insert(
                        "valueString".to_string(),
                        serde_json::Value::String(property.value),
                    );
                    prop_parts.push(serde_json::Value::Object(value_part));

                    if let Some(prop_type) = property.property_type {
                        let mut type_part = serde_json::Map::new();
                        type_part.insert(
                            "name".to_string(),
                            serde_json::Value::String("type".to_string()),
                        );
                        type_part.insert(
                            "valueCode".to_string(),
                            serde_json::Value::String(prop_type),
                        );
                        prop_parts.push(serde_json::Value::Object(type_part));
                    }

                    prop_param.insert("part".to_string(), serde_json::Value::Array(prop_parts));
                    parameter_list.push(serde_json::Value::Object(prop_param));
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
                format!("Concept lookup failed: {e}"),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
