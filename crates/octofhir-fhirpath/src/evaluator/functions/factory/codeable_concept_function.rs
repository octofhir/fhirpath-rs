//! %factory.CodeableConcept(coding, text) function implementation
//!
//! Creates a FHIR CodeableConcept instance.
//! Syntax: %factory.CodeableConcept(coding, text)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryCodeableConceptFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryCodeableConceptFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "CodeableConcept".to_string(),
                description: "Creates a FHIR CodeableConcept instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "coding".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Coding or collection of Codings".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "text".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Plain text representation".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "CodeableConcept".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(2),
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

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryCodeableConceptFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "CodeableConcept function can only be called on %factory variable".to_string(),
            ));
        }

        // First arg: coding (single or collection)
        let codings_arg = args.first().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "CodeableConcept function requires at least a coding argument".to_string(),
            )
        })?;

        let mut coding_array = Vec::new();
        for coding_value in codings_arg {
            match coding_value {
                FhirPathValue::Resource(json, _, _) => {
                    coding_array.push(json.as_ref().clone());
                }
                _ => {
                    // Skip non-resource values
                }
            }
        }

        let mut cc = serde_json::Map::new();
        if !coding_array.is_empty() {
            cc.insert("coding".to_string(), serde_json::Value::Array(coding_array));
        }

        // Optional text arg
        if let Some(text_value) = args.get(1).and_then(|a| a.first())
            && let FhirPathValue::String(s, _, _) = text_value
        {
            cc.insert("text".to_string(), serde_json::Value::String(s.clone()));
        }

        let type_info = Arc::new(TypeInfo::new_complex("CodeableConcept"));
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(cc),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
