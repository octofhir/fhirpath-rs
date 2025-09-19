//! Union function implementation
//!
//! The union function returns the union of two collections, removing duplicates.
//! Syntax: collection1.union(collection2)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Union function evaluator
pub struct UnionFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl UnionFunctionEvaluator {
    /// Create a new union function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "union".to_string(),
                description: "Returns the union of two collections, removing duplicates".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "other".to_string(),
                        parameter_type: vec!["Any".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The other collection to union with".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Combining,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for UnionFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("union function expects 1 argument, got {}", args.len()),
            ));
        }

        // Get the pre-evaluated argument values
        let other_values = args[0].clone();

        // Combine the input and other collections
        let mut result_values = input;
        result_values.extend(other_values);

        // Remove duplicates while preserving order
        let mut unique_values = Vec::new();
        for value in result_values {
            if !unique_values.iter().any(|existing| {
                match (existing, &value) {
                    // Use the FhirPath equality semantics
                    (FhirPathValue::Integer(a, _, _), FhirPathValue::Integer(b, _, _)) => a == b,
                    (FhirPathValue::Decimal(a, _, _), FhirPathValue::Decimal(b, _, _)) => a == b,
                    (FhirPathValue::String(a, _, _), FhirPathValue::String(b, _, _)) => a == b,
                    (FhirPathValue::Boolean(a, _, _), FhirPathValue::Boolean(b, _, _)) => a == b,
                    (FhirPathValue::Date(a, _, _), FhirPathValue::Date(b, _, _)) => a == b,
                    (FhirPathValue::DateTime(a, _, _), FhirPathValue::DateTime(b, _, _)) => a == b,
                    (FhirPathValue::Time(a, _, _), FhirPathValue::Time(b, _, _)) => a == b,
                    (FhirPathValue::Quantity { value: v1, unit: u1, .. }, FhirPathValue::Quantity { value: v2, unit: u2, .. }) => {
                        v1 == v2 && u1 == u2
                    },
                    // For different types, they are not equal
                    _ => false,
                }
            }) {
                unique_values.push(value);
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(unique_values),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}