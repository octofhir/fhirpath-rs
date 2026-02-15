//! %factory.Identifier(system, value, use, type) function implementation
//!
//! Creates a FHIR Identifier instance.
//! Syntax: %factory.Identifier(system, value, use, type)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryIdentifierFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryIdentifierFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "Identifier".to_string(),
                description: "Creates a FHIR Identifier instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "system".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Identifier system URI".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Identifier value".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "use".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description:
                                "Identifier use (usual | official | temp | secondary | old)"
                                    .to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "type".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "CodeableConcept for identifier type".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Identifier".to_string(),
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
impl PureFunctionEvaluator for FactoryIdentifierFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "Identifier function can only be called on %factory variable".to_string(),
            ));
        }

        let system = extract_string(&args, 0).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "Identifier function requires a string system argument".to_string(),
            )
        })?;

        let value = extract_string(&args, 1).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "Identifier function requires a string value argument".to_string(),
            )
        })?;

        let mut identifier = serde_json::Map::new();
        identifier.insert("system".to_string(), serde_json::Value::String(system));
        identifier.insert("value".to_string(), serde_json::Value::String(value));

        if let Some(use_val) = extract_string(&args, 2) {
            identifier.insert("use".to_string(), serde_json::Value::String(use_val));
        }

        if let Some(type_value) = args.get(3).and_then(|a| a.first())
            && let FhirPathValue::Resource(json, _, _) = type_value
        {
            identifier.insert("type".to_string(), json.as_ref().clone());
        }

        let type_info = TypeInfo::new_complex("Identifier");
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(identifier),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
