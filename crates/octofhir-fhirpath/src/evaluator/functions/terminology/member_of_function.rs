//! MemberOf function implementation
//!
//! The memberOf function tests whether a code is a member of a specified value set.
//! This function requires a terminology provider to perform the membership test.
//! Syntax: code.memberOf(valueSet) or memberOf(code, valueSetUrl)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// MemberOf function evaluator
pub struct MemberOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MemberOfFunctionEvaluator {
    /// Create a new memberOf function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "memberOf".to_string(),
                description: "Tests whether a code is a member of a specified value set. Returns boolean indicating membership.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "valueSet".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Value set URL or value set resource to test membership against".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Terminology operations may change over time
                category: FunctionCategory::Terminology,
                requires_terminology: true,
                requires_model: false,
            },
        })
    }

    /// Extract coding information from input (supports Coding, CodeableConcept, and code strings)
    fn extract_codings(input: &[FhirPathValue]) -> Result<Vec<(Option<String>, String)>> {
        let mut codings = Vec::new();

        for value in input {
            match value {
                FhirPathValue::Resource(resource, _, _) => {
                    // Check if it's a Coding resource
                    if resource.get("code").is_some() && resource.get("system").is_some() {
                        let system = resource
                            .get("system")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());

                        let code =
                            resource
                                .get("code")
                                .and_then(|c| c.as_str())
                                .ok_or_else(|| {
                                    FhirPathError::evaluation_error(
                                        crate::core::error_code::FP0058,
                                        "Coding resource must have a code field".to_string(),
                                    )
                                })?;

                        codings.push((system, code.to_string()));
                    }
                    // Check if it's a CodeableConcept resource
                    else if let Some(coding_array) = resource.get("coding") {
                        if let Some(coding_list) = coding_array.as_array() {
                            for coding_value in coding_list {
                                if let Some(coding_obj) = coding_value.as_object() {
                                    let system = coding_obj
                                        .get("system")
                                        .and_then(|s| s.as_str())
                                        .map(|s| s.to_string());

                                    let code = coding_obj
                                        .get("code")
                                        .and_then(|c| c.as_str())
                                        .ok_or_else(|| {
                                            FhirPathError::evaluation_error(
                                                crate::core::error_code::FP0058,
                                                "Coding in CodeableConcept must have a code field"
                                                    .to_string(),
                                            )
                                        })?;

                                    codings.push((system, code.to_string()));
                                }
                            }
                        }
                    } else {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            "memberOf function requires Coding, CodeableConcept, or code string input".to_string(),
                        ));
                    }
                }
                FhirPathValue::String(code, _, _) => {
                    // Simple code string without system
                    codings.push((None, code.clone()));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        "memberOf function can only be called on Coding, CodeableConcept, or code string".to_string(),
                    ));
                }
            }
        }

        if codings.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "memberOf function requires at least one coding input".to_string(),
            ));
        }

        Ok(codings)
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
                "memberOf function valueSet parameter must evaluate to a single value".to_string(),
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
                "memberOf function valueSet parameter must be a string URL or ValueSet resource"
                    .to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for MemberOfFunctionEvaluator {
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
                "memberOf function requires exactly one argument (valueSet)".to_string(),
            ));
        }

        // Get terminology provider
        let terminology_provider = context.terminology_provider().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "memberOf function requires a terminology provider".to_string(),
            )
        })?;

        // Extract coding information from input
        let codings = Self::extract_codings(&input)?;

        // Get value set URL
        let value_set_url = Self::get_value_set_url(&args[0], context, evaluator).await?;

        let mut results = Vec::new();

        // Test membership for each coding
        for (system, code) in codings {
            match terminology_provider
                .validate_code_vs(&value_set_url, system.as_deref(), &code, None)
                .await
            {
                Ok(validation_result) => {
                    // memberOf returns boolean true if the code is valid in the value set
                    results.push(FhirPathValue::boolean(validation_result.result));
                }
                Err(e) => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0059,
                        format!("Value set membership test failed: {e}"),
                    ));
                }
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
