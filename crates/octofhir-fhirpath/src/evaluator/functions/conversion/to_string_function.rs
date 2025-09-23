//! ToString function implementation
//!
//! The toString function converts a value to its string representation.
//! Syntax: value.toString()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use octofhir_ucum::find_unit;
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ToString function evaluator
pub struct ToStringFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToStringFunctionEvaluator {
    /// Create a new toString function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toString".to_string(),
                description: "Converts a value to its string representation".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Conversion,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ToStringFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toString function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let string_result = match &value {
                FhirPathValue::String(s, _, _) => s.clone(),
                FhirPathValue::Boolean(b, _, _) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                FhirPathValue::Integer(i, _, _) => i.to_string(),
                FhirPathValue::Decimal(d, _, _) => d.to_string(),
                FhirPathValue::Date(date, _, _) => {
                    // Format date as FHIR date string using Display implementation
                    format!("{date}")
                }
                FhirPathValue::DateTime(dt, _, _) => {
                    // Format datetime as FHIR datetime string using Display implementation
                    format!("{dt}")
                }
                FhirPathValue::Time(time, _, _) => {
                    // Format time as FHIR time string using Display implementation
                    format!("{time}")
                }
                FhirPathValue::Quantity { value, unit, ucum_unit, calendar_unit, .. } => {
                    // Format Quantity according to FHIRPath rules:
                    // - UCUM units are rendered with single quotes: 1 'wk'
                    // - Calendar units are rendered as plain words: 1 week
                    // - Dimensionless quantities (unit '1') are rendered as just the value: 1 or 1.0
                    if let Some(u) = unit.as_deref() {
                        if u == "1" {
                            value.to_string()
                        } else {
                            let is_ucum = find_unit(u).is_some() || ucum_unit.is_some();
                            if is_ucum {
                                format!("{value} '{u}'")
                            } else if calendar_unit.is_some() {
                                format!("{value} {u}")
                            } else {
                                // Default to quoting non-UCUM, non-calendar units to preserve literal form
                                format!("{value} '{u}'")
                            }
                        }
                    } else {
                        value.to_string()
                    }
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        format!("Cannot convert {} to string", value.type_name()),
                    ));
                }
            };

            results.push(FhirPathValue::string(string_result));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
