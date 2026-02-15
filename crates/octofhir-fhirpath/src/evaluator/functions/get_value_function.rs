//! getValue function implementation
//!
//! The getValue function returns the system value of a FHIR primitive,
//! stripping extensions and returning just the underlying value.
//! Syntax: element.getValue()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// GetValue function evaluator
pub struct GetValueFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl GetValueFunctionEvaluator {
    /// Create a new getValue function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "getValue".to_string(),
                description: "Returns the system value of a FHIR primitive, excluding extensions"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
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
impl PureFunctionEvaluator for GetValueFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "getValue function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // For primitive types, return the value without extensions (wrapped_primitive_element stripped)
        let mut results = Vec::new();
        for item in input {
            let stripped = match item {
                FhirPathValue::Boolean(v, ti, _) => FhirPathValue::Boolean(v, ti, None),
                FhirPathValue::Integer(v, ti, _) => FhirPathValue::Integer(v, ti, None),
                FhirPathValue::Decimal(v, ti, _) => FhirPathValue::Decimal(v, ti, None),
                FhirPathValue::String(v, ti, _) => FhirPathValue::String(v, ti, None),
                FhirPathValue::Date(v, ti, _) => FhirPathValue::Date(v, ti, None),
                FhirPathValue::DateTime(v, ti, _) => FhirPathValue::DateTime(v, ti, None),
                FhirPathValue::Time(v, ti, _) => FhirPathValue::Time(v, ti, None),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    code,
                    system,
                    ucum_unit,
                    calendar_unit,
                    type_info,
                    primitive_element: _,
                } => FhirPathValue::Quantity {
                    value,
                    unit,
                    code,
                    system,
                    ucum_unit,
                    calendar_unit,
                    type_info,
                    primitive_element: None,
                },
                // For non-primitive types, return empty (no system value)
                _ => continue,
            };
            results.push(stripped);
        }

        Ok(EvaluationResult {
            value: Collection::from_values(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
