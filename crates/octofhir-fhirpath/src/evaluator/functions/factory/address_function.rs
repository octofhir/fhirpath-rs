//! %factory.Address(line, city, state, postalCode, country, use, type) function implementation
//!
//! Creates a FHIR Address instance.
//! Syntax: %factory.Address(line, city, state, postalCode, country, use, type)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryAddressFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryAddressFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "Address".to_string(),
                description: "Creates a FHIR Address instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "line".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Street address lines (collection)".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "city".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "City name".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "state".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "State/province".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "postalCode".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Postal/zip code".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "country".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Country".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "use".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Address use (home | work | temp | old | billing)"
                                .to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "type".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Address type (postal | physical | both)".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Address".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(7),
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

fn extract_string(args: &[Collection], index: usize) -> Option<String> {
    args.get(index)
        .and_then(|a| a.first())
        .and_then(|v| match v {
            FhirPathValue::String(s, _, _) => Some(s.clone()),
            _ => None,
        })
}

fn extract_string_collection(args: &[Collection], index: usize) -> Vec<String> {
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
impl PureFunctionEvaluator for FactoryAddressFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "Address function can only be called on %factory variable".to_string(),
            ));
        }

        let mut addr = serde_json::Map::new();

        let lines = extract_string_collection(&args, 0);
        if !lines.is_empty() {
            addr.insert(
                "line".to_string(),
                serde_json::Value::Array(
                    lines.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }

        if let Some(city) = extract_string(&args, 1) {
            addr.insert("city".to_string(), serde_json::Value::String(city));
        }
        if let Some(state) = extract_string(&args, 2) {
            addr.insert("state".to_string(), serde_json::Value::String(state));
        }
        if let Some(postal_code) = extract_string(&args, 3) {
            addr.insert(
                "postalCode".to_string(),
                serde_json::Value::String(postal_code),
            );
        }
        if let Some(country) = extract_string(&args, 4) {
            addr.insert("country".to_string(), serde_json::Value::String(country));
        }
        if let Some(use_val) = extract_string(&args, 5) {
            addr.insert("use".to_string(), serde_json::Value::String(use_val));
        }
        if let Some(type_val) = extract_string(&args, 6) {
            addr.insert("type".to_string(), serde_json::Value::String(type_val));
        }

        let type_info = Arc::new(TypeInfo::new_complex("Address"));
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(addr),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
