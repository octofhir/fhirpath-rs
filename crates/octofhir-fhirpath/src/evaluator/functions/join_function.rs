//! Join function implementation
//!
//! The join function joins a collection of strings into a single string with a separator.
//! Syntax: collection.join(separator)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Join function evaluator
pub struct JoinFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl JoinFunctionEvaluator {
    /// Create a new join function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "join".to_string(),
                description: "Joins a collection of strings into a single string with a separator"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "separator".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "String separator to join with".to_string(),
                        default_value: None,
                    }],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::StringManipulation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for JoinFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "join function requires exactly one argument (separator)".to_string(),
            ));
        }

        // Get the pre-evaluated separator argument
        if args[0].len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "join function separator argument must be a single value".to_string(),
            ));
        }

        let separator_str = match &args[0][0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "join function separator argument must be a string".to_string(),
                ));
            }
        };

        // Convert all input values to strings
        let string_values: Result<Vec<String>> = input
            .iter()
            .map(|value| match value {
                FhirPathValue::String(s, _, _) => Ok(s.clone()),
                FhirPathValue::Integer(i, _, _) => Ok(i.to_string()),
                FhirPathValue::Decimal(d, _, _) => Ok(d.to_string()),
                FhirPathValue::Boolean(b, _, _) => Ok(b.to_string()),
                FhirPathValue::Date(d, _, _) => Ok(d.to_string()),
                FhirPathValue::DateTime(dt, _, _) => Ok(dt.to_string()),
                FhirPathValue::Time(t, _, _) => Ok(t.to_string()),
                _ => Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!("Cannot convert {:?} to string for join operation", value),
                )),
            })
            .collect();

        let strings = string_values?;

        // Join the strings
        let joined = strings.join(&separator_str);

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::string(joined)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
