//! Subsumes function implementation
//!
//! The subsumes function tests subsumption relationships between concepts.
//! This function requires a terminology provider to perform the subsumption test.
//! Syntax: parentCode.subsumes(childCode) or subsumes(system, parent, child)

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};

/// Subsumes function evaluator
pub struct SubsumesFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SubsumesFunctionEvaluator {
    /// Create a new subsumes function evaluator
    pub fn create() -> Arc<dyn ProviderPureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "subsumes".to_string(),
                description: "Tests subsumption relationships between concepts. Returns boolean indicating if the first concept subsumes the second.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "childCode".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Child concept (Coding) or system URL when using three-parameter form".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "parentCode".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Parent code when using three-parameter form (system, parent, child)".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "childCodeParam".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Child code when using three-parameter form (system, parent, child)".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(3),
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

    /// Extract subsumption parameters based on argument count
    fn extract_subsumption_params(
        input: &[FhirPathValue],
        args: &[Vec<FhirPathValue>],
    ) -> Result<(String, String, String)> {
        match args.len() {
            1 => {
                // Two-parameter form: parentCode.subsumes(childCode)
                if input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        "subsumes function requires a single Coding input in two-parameter form".to_string(),
                    ));
                }

                let (parent_system, parent_code) = Self::extract_coding_info(&input[0])?;

                let child_values = &args[0];
                if child_values.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0056,
                        "subsumes function child parameter must evaluate to a single value".to_string(),
                    ));
                }

                let (child_system, child_code) = Self::extract_coding_info(&child_values[0])?;

                // Systems must match for subsumption
                if parent_system != child_system {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0058,
                        "subsumes function requires parent and child codes to be from the same system".to_string(),
                    ));
                }

                Ok((parent_system, parent_code, child_code))
            }
            3 => {
                // Three-parameter form: subsumes(system, parent, child)
                let system_values = &args[0];
                if system_values.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0056,
                        "subsumes function system parameter must evaluate to a single value".to_string(),
                    ));
                }

                let system = match &system_values[0] {
                    FhirPathValue::String(s, _, _) => s.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "subsumes function system parameter must be a string".to_string(),
                    )),
                };

                let parent_values = &args[1];
                if parent_values.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0056,
                        "subsumes function parent parameter must evaluate to a single value".to_string(),
                    ));
                }

                let parent_code = match &parent_values[0] {
                    FhirPathValue::String(c, _, _) => c.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "subsumes function parent parameter must be a string".to_string(),
                    )),
                };

                let child_values = &args[2];
                if child_values.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0056,
                        "subsumes function child parameter must evaluate to a single value".to_string(),
                    ));
                }

                let child_code = match &child_values[0] {
                    FhirPathValue::String(c, _, _) => c.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "subsumes function child parameter must be a string".to_string(),
                    )),
                };

                Ok((system, parent_code, child_code))
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "subsumes function takes either 1 argument (childCode) or 3 arguments (system, parent, child)".to_string(),
            )),
        }
    }

    /// Extract system and code from a Coding value
    fn extract_coding_info(value: &FhirPathValue) -> Result<(String, String)> {
        match value {
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
                "subsumes function requires Coding resources".to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl ProviderPureFunctionEvaluator for SubsumesFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
        context: &crate::evaluator::EvaluationContext,
    ) -> Result<EvaluationResult> {
        if args.is_empty() || args.len() > 3 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "subsumes function takes 1 or 3 arguments".to_string(),
            ));
        }

        // Get terminology provider
        let terminology_provider = context.terminology_provider().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "subsumes function requires a terminology provider".to_string(),
            )
        })?;

        // Extract subsumption parameters
        let (system, parent_code, child_code) = Self::extract_subsumption_params(&input, &args)?;

        // Perform subsumption test
        match terminology_provider
            .subsumes(&system, &parent_code, &child_code)
            .await
        {
            Ok(subsumes_result) => {
                // Extract subsumption outcome and convert to boolean
                use octofhir_fhir_model::terminology::SubsumptionOutcome;
                let subsumes = matches!(subsumes_result.outcome, SubsumptionOutcome::Subsumes);

                Ok(EvaluationResult {
                    value: crate::core::Collection::from(vec![FhirPathValue::boolean(subsumes)]),
                })
            }
            Err(e) => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0059,
                format!("Subsumption test failed: {e}"),
            )),
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
