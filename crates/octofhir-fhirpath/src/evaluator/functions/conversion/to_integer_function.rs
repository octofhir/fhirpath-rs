//! ToInteger function implementation
//!
//! The toInteger function converts a value to an integer.
//! Syntax: value.toInteger()

use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ToInteger function evaluator
pub struct ToIntegerFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToIntegerFunctionEvaluator {
    /// Create a new toInteger function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toInteger".to_string(),
                description: "Converts a value to an integer".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Integer".to_string(),
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
impl PureFunctionEvaluator for ToIntegerFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toInteger function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            match &value {
                FhirPathValue::Integer(i, _, _) => {
                    results.push(FhirPathValue::integer(*i));
                }
                FhirPathValue::Boolean(b, _, _) => {
                    let integer_value = if *b { 1 } else { 0 };
                    results.push(FhirPathValue::integer(integer_value));
                }
                FhirPathValue::String(s, _, _) => {
                    // Try to parse string as integer, return empty if parsing fails
                    if let Ok(integer_value) = s.trim().parse::<i64>() {
                        results.push(FhirPathValue::integer(integer_value));
                    }
                    // If parsing fails, don't add anything to results (effectively returns empty)
                }
                FhirPathValue::Decimal(d, _, _) => {
                    // Check if decimal has fractional part - if so, return empty
                    if d.fract() != rust_decimal::Decimal::ZERO {
                        // Decimal has fractional part, return empty
                    } else {
                        // Truncate decimal to integer (towards zero)
                        if let Some(integer_value) = d.trunc().to_i64() {
                            results.push(FhirPathValue::integer(integer_value));
                        }
                        // If conversion fails (too large), don't add anything (effectively returns empty)
                    }
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
