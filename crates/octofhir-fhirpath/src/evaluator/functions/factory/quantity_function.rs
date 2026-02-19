//! %factory.Quantity(system, code, value, unit) function implementation
//!
//! Creates a FHIR Quantity instance.
//! Syntax: %factory.Quantity(system, code, value, unit)

use std::sync::Arc;

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::factory_variable::is_factory_variable;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

pub struct FactoryQuantityFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl FactoryQuantityFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "Quantity".to_string(),
                description: "Creates a FHIR Quantity instance".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "system".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Quantity system URI".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "code".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Coded form of the unit".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Numerical value".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "unit".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Unit representation".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Quantity".to_string(),
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

fn extract_string(args: &[Collection], index: usize) -> Option<String> {
    args.get(index)
        .and_then(|a| a.first())
        .and_then(|v| match v {
            FhirPathValue::String(s, _, _) => Some(s.clone()),
            _ => None,
        })
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for FactoryQuantityFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if input.len() != 1 || !is_factory_variable(&input[0]) {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                "Quantity function can only be called on %factory variable".to_string(),
            ));
        }

        let system = extract_string(&args, 0).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "Quantity function requires a string system argument".to_string(),
            )
        })?;

        let code = extract_string(&args, 1).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "Quantity function requires a string code argument".to_string(),
            )
        })?;

        let mut qty = serde_json::Map::new();
        qty.insert("system".to_string(), serde_json::Value::String(system));
        qty.insert("code".to_string(), serde_json::Value::String(code));

        // value argument (can be decimal, integer, or string)
        if let Some(value_arg) = args.get(2).and_then(|a| a.first()) {
            match value_arg {
                FhirPathValue::Decimal(d, _, _) => {
                    if let Ok(f) = d.to_string().parse::<f64>()
                        && let Some(n) = serde_json::Number::from_f64(f)
                    {
                        qty.insert("value".to_string(), serde_json::Value::Number(n));
                    }
                }
                FhirPathValue::Integer(i, _, _) => {
                    qty.insert("value".to_string(), serde_json::json!(*i));
                }
                FhirPathValue::String(s, _, _) => {
                    if let Ok(f) = s.parse::<f64>()
                        && let Some(n) = serde_json::Number::from_f64(f)
                    {
                        qty.insert("value".to_string(), serde_json::Value::Number(n));
                    }
                }
                _ => {}
            }
        }

        if let Some(unit) = extract_string(&args, 3) {
            qty.insert("unit".to_string(), serde_json::Value::String(unit));
        }

        let type_info = Arc::new(TypeInfo::new_complex("Quantity"));
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::resource_wrapped(
                serde_json::Value::Object(qty),
                type_info,
            )),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
