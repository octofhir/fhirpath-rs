//! MemberOf function implementation
//!
//! The memberOf function tests whether a code is a member of a specified value set.
//! This function requires a terminology provider to perform the membership test.
//! Syntax: code.memberOf(valueSet) or memberOf(code, valueSetUrl)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::node::FhirNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
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

    /// The (system, code) pairs to test, per FHIRPath `memberOf()`: a code or
    /// string is one code with no system, a Coding is its own pair, and a
    /// CodeableConcept contributes each of its codings. A coding without a
    /// `code` is a member of nothing and is skipped.
    fn extract_codings(input: &[FhirPathValue]) -> Result<Vec<(Option<String>, String)>> {
        let mut codings = Vec::new();

        for value in input {
            match value {
                FhirPathValue::Resource(resource, _, _) => {
                    if let Some(coding_list) = resource.get("coding").and_then(|c| c.as_array()) {
                        codings.extend(coding_list.iter().filter_map(Self::coding_pair));
                    } else if resource.get("code").is_some() {
                        codings.extend(Self::coding_pair(resource));
                    } else {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0055,
                            "memberOf function requires Coding, CodeableConcept, or code input"
                                .to_string(),
                        ));
                    }
                }
                FhirPathValue::String(code, _, _) => codings.push((None, code.clone())),
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        "memberOf function can only be called on Coding, CodeableConcept, or code"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(codings)
    }

    /// The (system, code) of a Coding-shaped JSON object, if it has a code.
    fn coding_pair(coding: &FhirNode) -> Option<(Option<String>, String)> {
        let code = coding.get("code")?.as_str()?;
        let system = coding
            .get("system")
            .and_then(|s| s.as_str())
            .map(str::to_string);
        Some((system, code.to_string()))
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
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        self.evaluate_borrowed(input, context, &args, evaluator)
            .await
    }

    async fn evaluate_borrowed(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: &[ExpressionNode],
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

        // FHIRPath functions propagate an empty input.
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let codings = Self::extract_codings(input.values())?;
        let value_set_url = Self::get_value_set_url(&args[0], context, evaluator).await?;

        // A concept is a member when any of its codes is (FHIRPath memberOf):
        // the result is one boolean, not one per coding.
        let mut member = false;
        for (system, code) in codings {
            let validation = terminology_provider
                .validate_code_vs(&value_set_url, system.as_deref(), &code, None)
                .await
                .map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0059,
                        format!("Value set membership test failed: {e}"),
                    )
                })?;
            if validation.result {
                member = true;
                break;
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(member)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
