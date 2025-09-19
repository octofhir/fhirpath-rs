//! Translate function implementation
//!
//! The translate function translates concepts between code systems using ConceptMap.
//! This function requires a terminology provider to perform the translation.
//! Syntax: concept.translate(targetSystem) or translate(conceptMap, concept)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use octofhir_fhir_model::TerminologyProvider;

/// Translate function evaluator
pub struct TranslateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TranslateFunctionEvaluator {
    /// Create a new translate function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "translate".to_string(),
                description: "Translates concepts between code systems using ConceptMap. Returns translation results as Parameters.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "target".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Target system URL or ConceptMap URL/resource for translation".to_string(),
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
    fn extract_coding_info(input: &[FhirPathValue]) -> Result<(String, String)> {
        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "translate function requires a single Coding input".to_string(),
            ));
        }

        match &input[0] {
            FhirPathValue::Resource(resource, _, _) => {
                let system = resource
                    .get("system")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "Coding resource must have a system field".to_string(),
                        )
                    })?;

                let code = resource
                    .get("code")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "Coding resource must have a code field".to_string(),
                        )
                    })?;

                Ok((system.to_string(), code.to_string()))
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                "translate function can only be called on Coding resources".to_string(),
            )),
        }
    }

    /// Extract target system or concept map URL from parameter
    async fn get_target_parameter(
        arg: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<String> {
        let result = evaluator.evaluate(arg, context).await?;
        let values: Vec<FhirPathValue> = result.value.iter().cloned().collect();

        if values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "translate function target parameter must evaluate to a single value".to_string(),
            ));
        }

        match &values[0] {
            FhirPathValue::String(url, _, _) => Ok(url.clone()),
            FhirPathValue::Resource(resource, _, _) => {
                // Extract URL from ConceptMap resource
                resource
                    .get("url")
                    .and_then(|u| u.as_str())
                    .map(|u| u.to_string())
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            "ConceptMap resource must have a url field".to_string(),
                        )
                    })
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                "translate function target parameter must be a string URL or ConceptMap resource"
                    .to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for TranslateFunctionEvaluator {
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
                "translate function requires exactly one argument (target)".to_string(),
            ));
        }

        // Get terminology provider
        let terminology_provider = context.terminology_provider().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "translate function requires a terminology provider".to_string(),
            )
        })?;

        // Extract coding information
        let (system, code) = Self::extract_coding_info(&input)?;

        // Get target parameter
        let target = Self::get_target_parameter(&args[0], context, evaluator).await?;

        // Perform translation
        match terminology_provider
            .translate_code(&code, &target, Some(&system))
            .await
        {
            Ok(translation_result) => {
                // Convert translation result to FHIR Parameters resource structure
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
                    serde_json::Value::Bool(translation_result.success),
                );
                parameter_list.push(serde_json::Value::Object(result_param));

                // Add message parameter if available
                if let Some(message) = translation_result.message {
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

                // Add match parameters for each translation target
                for target in translation_result.targets {
                    let mut match_param = serde_json::Map::new();
                    match_param.insert(
                        "name".to_string(),
                        serde_json::Value::String("match".to_string()),
                    );

                    let mut match_parts = Vec::new();

                    let mut equivalence_part = serde_json::Map::new();
                    equivalence_part.insert(
                        "name".to_string(),
                        serde_json::Value::String("equivalence".to_string()),
                    );
                    let equivalence_code = match target.equivalence {
                        octofhir_fhir_model::terminology::EquivalenceLevel::Equivalent => {
                            "equivalent"
                        }
                        octofhir_fhir_model::terminology::EquivalenceLevel::Related => "related",
                        octofhir_fhir_model::terminology::EquivalenceLevel::Narrower => "narrower",
                        octofhir_fhir_model::terminology::EquivalenceLevel::Broader => "broader",
                    };
                    equivalence_part.insert(
                        "valueCode".to_string(),
                        serde_json::Value::String(equivalence_code.to_string()),
                    );
                    match_parts.push(serde_json::Value::Object(equivalence_part));

                    let mut concept_part = serde_json::Map::new();
                    concept_part.insert(
                        "name".to_string(),
                        serde_json::Value::String("concept".to_string()),
                    );

                    let mut concept_value = serde_json::Map::new();
                    concept_value
                        .insert("code".to_string(), serde_json::Value::String(target.code));
                    concept_value.insert(
                        "system".to_string(),
                        serde_json::Value::String(target.system),
                    );
                    if let Some(display) = target.display {
                        concept_value
                            .insert("display".to_string(), serde_json::Value::String(display));
                    }

                    concept_part.insert(
                        "valueCoding".to_string(),
                        serde_json::Value::Object(concept_value),
                    );
                    match_parts.push(serde_json::Value::Object(concept_part));

                    match_param.insert("part".to_string(), serde_json::Value::Array(match_parts));
                    parameter_list.push(serde_json::Value::Object(match_param));
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
                format!("Concept translation failed: {}", e),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
