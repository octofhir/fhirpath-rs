//! %factory.HumanName(family, given, prefix, suffix, text, use) function implementation
//!
//! Creates a FHIR HumanName instance.
//! Syntax: %factory.HumanName(family, given, prefix, suffix, text, use)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryHumanNameFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryHumanNameFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "HumanName".to_string(),
                description: "Creates a FHIR HumanName instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "family".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Family name".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "given".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Given names (collection)".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "prefix".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Name prefix".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "suffix".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Name suffix".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "text".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Full text representation".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "use".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Name use (usual | official | temp | nickname | anonymous | old | maiden)".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "HumanName".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(6),
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

fn extract_string_collection(args: &[Vec<FhirPathValue>], index: usize) -> Vec<String> {
    args.get(index)
        .map(|a| {
            a.iter()
                .filter_map(|v| match v {
                    FhirPathValue::String(s, _, _) => Some(s.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryHumanNameFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "HumanName function can only be called on %factory variable".to_string(),
            ));
        }

        let mut name = serde_json::Map::new();

        if let Some(family) = extract_string(&args, 0) {
            name.insert("family".to_string(), serde_json::Value::String(family));
        }

        let given = extract_string_collection(&args, 1);
        if !given.is_empty() {
            name.insert(
                "given".to_string(),
                serde_json::Value::Array(
                    given.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }

        if let Some(prefix) = extract_string(&args, 2) {
            name.insert(
                "prefix".to_string(),
                serde_json::Value::Array(vec![serde_json::Value::String(prefix)]),
            );
        }

        if let Some(suffix) = extract_string(&args, 3) {
            name.insert(
                "suffix".to_string(),
                serde_json::Value::Array(vec![serde_json::Value::String(suffix)]),
            );
        }

        if let Some(text) = extract_string(&args, 4) {
            name.insert("text".to_string(), serde_json::Value::String(text));
        }

        if let Some(use_val) = extract_string(&args, 5) {
            name.insert("use".to_string(), serde_json::Value::String(use_val));
        }

        let type_info = TypeInfo::new_complex("HumanName");
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(name),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
