//! %factory.Coding(system, code, display, version) function implementation
//!
//! Creates a FHIR Coding instance.
//! Syntax: %factory.Coding(system, code, display, version)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryCodingFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryCodingFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "Coding".to_string(),
                description: "Creates a FHIR Coding instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "system".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Coding system URI".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "code".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Code value".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "display".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Display text".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "version".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Version of the code system".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Coding".to_string(),
                    polymorphic: false,
                    min_params: 2,
                    max_params: Some(4),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

fn extract_string(args: &[Vec<FhirPathValue>], index: usize) -> Option<String> {
    args.get(index)
        .and_then(|a| a.first())
        .and_then(|v| match v {
            FhirPathValue::String(s, _, _) => Some(s.clone()),
            _ => None,
        })
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryCodingFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "Coding function can only be called on %factory variable".to_string(),
            ));
        }

        let system = extract_string(&args, 0).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "Coding function requires a string system argument".to_string(),
            )
        })?;

        let code = extract_string(&args, 1).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "Coding function requires a string code argument".to_string(),
            )
        })?;

        let mut coding = serde_json::Map::new();
        coding.insert("system".to_string(), serde_json::Value::String(system));
        coding.insert("code".to_string(), serde_json::Value::String(code));

        if let Some(display) = extract_string(&args, 2) {
            coding.insert("display".to_string(), serde_json::Value::String(display));
        }
        if let Some(version) = extract_string(&args, 3) {
            coding.insert("version".to_string(), serde_json::Value::String(version));
        }

        let type_info = TypeInfo::new_complex("Coding");
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(coding),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
