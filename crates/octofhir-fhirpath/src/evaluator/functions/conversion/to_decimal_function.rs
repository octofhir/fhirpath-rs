//! ToDecimal function implementation
//!
//! The toDecimal function converts a value to a decimal.
//! Syntax: value.toDecimal()

use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ToDecimal function evaluator
pub struct ToDecimalFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToDecimalFunctionEvaluator {
    /// Create a new toDecimal function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toDecimal".to_string(),
                description: "Converts a value to a decimal".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Decimal".to_string(),
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
impl PureFunctionEvaluator for ToDecimalFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toDecimal function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            match &value {
                FhirPathValue::Decimal(d, _, _) => {
                    results.push(FhirPathValue::decimal(*d));
                }
                FhirPathValue::Integer(i, _, _) => {
                    results.push(FhirPathValue::decimal(Decimal::from(*i)));
                }
                FhirPathValue::Boolean(b, _, _) => {
                    let decimal_value = if *b { Decimal::ONE } else { Decimal::ZERO };
                    results.push(FhirPathValue::decimal(decimal_value));
                }
                FhirPathValue::String(s, _, _) => {
                    // Try to parse string as decimal, return empty if parsing fails
                    if let Ok(decimal_value) = s.trim().parse::<Decimal>() {
                        results.push(FhirPathValue::decimal(decimal_value));
                    }
                    // If parsing fails, don't add anything to results (effectively returns empty)
                }
                _ => {
                    // For unsupported types, don't add anything to results (effectively returns empty)
                }
            };
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
