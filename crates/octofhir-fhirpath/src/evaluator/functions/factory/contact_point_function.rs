//! %factory.ContactPoint(system, value, use) function implementation
//!
//! Creates a FHIR ContactPoint instance.
//! Syntax: %factory.ContactPoint(system, value, use)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryContactPointFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryContactPointFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "ContactPoint".to_string(),
                description: "Creates a FHIR ContactPoint instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "system".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "ContactPoint system (phone | fax | email | pager | url | sms | other)".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Contact value".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "use".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "ContactPoint use (home | work | temp | old | mobile)".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "ContactPoint".to_string(),
                    polymorphic: false,
                    min_params: 2,
                    max_params: Some(3),
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
impl PureFunctionEvaluator for FactoryContactPointFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "ContactPoint function can only be called on %factory variable".to_string(),
            ));
        }

        let system = extract_string(&args, 0).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "ContactPoint function requires a string system argument".to_string(),
            )
        })?;

        let value = extract_string(&args, 1).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "ContactPoint function requires a string value argument".to_string(),
            )
        })?;

        let mut cp = serde_json::Map::new();
        cp.insert("system".to_string(), serde_json::Value::String(system));
        cp.insert("value".to_string(), serde_json::Value::String(value));

        if let Some(use_val) = extract_string(&args, 2) {
            cp.insert("use".to_string(), serde_json::Value::String(use_val));
        }

        let type_info = TypeInfo::new_complex("ContactPoint");
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(cp),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
