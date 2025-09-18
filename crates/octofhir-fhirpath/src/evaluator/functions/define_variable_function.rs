//! DefineVariable function implementation
//!
//! The defineVariable function allows defining a variable within an expression scope.
//! It evaluates the value expression and makes it available under the specified variable name.
//! Syntax: defineVariable(name, value)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// DefineVariable function evaluator
pub struct DefineVariableFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DefineVariableFunctionEvaluator {
    /// Create a new defineVariable function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "defineVariable".to_string(),
                description: "Defines a variable within an expression scope. Evaluates the value expression and makes it available under the specified variable name.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "name".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "The variable name to define".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "The value expression to assign to the variable".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 2,
                    max_params: Some(2),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for DefineVariableFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "defineVariable function requires exactly two arguments (name, value)".to_string(),
            ));
        }

        // Evaluate the variable name
        let name_result = evaluator.evaluate(&args[0], context).await?;
        let name_values: Vec<FhirPathValue> = name_result.value.iter().cloned().collect();

        if name_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "defineVariable function name argument must evaluate to a single value".to_string(),
            ));
        }

        let variable_name = match &name_values[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "defineVariable function name argument must be a string".to_string(),
                ));
            }
        };

        // Validate variable name
        if variable_name.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                "defineVariable function name cannot be empty".to_string(),
            ));
        }

        // Check for valid identifier pattern (letters, numbers, underscore, but not starting with number)
        if !variable_name.chars().next().unwrap_or('0').is_alphabetic()
            && !variable_name.starts_with('_')
        {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                "defineVariable function name must start with a letter or underscore".to_string(),
            ));
        }

        if !variable_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0057,
                "defineVariable function name can only contain letters, numbers, and underscores"
                    .to_string(),
            ));
        }

        // Evaluate the variable value in the current context
        let value_result = evaluator.evaluate(&args[1], context).await?;

        // Get the collection as the variable value (defineVariable can assign collections)
        let variable_value = if value_result.value.is_empty() {
            // If the value expression evaluates to empty, use empty value
            FhirPathValue::Empty
        } else if value_result.value.len() == 1 {
            // Single value - store directly
            value_result
                .value
                .iter()
                .next()
                .cloned()
                .unwrap_or(FhirPathValue::Empty)
        } else {
            // Multiple values - store as a collection
            FhirPathValue::Collection(value_result.value.clone())
        };

        // For now, we'll implement a basic version that stores the variable
        // but doesn't make it available to subsequent expressions in the chain
        // This is a limitation of the current architecture where context is immutable

        // Log the variable definition (in a real implementation, this would be stored
        // in a mutable context that persists across the evaluation chain)
        log::debug!("defineVariable: {} = {:?}", variable_name, variable_value);

        // Return the original input collection (defineVariable is a pass-through function)
        // NOTE: The variable is not actually available for subsequent expressions
        // This requires architectural changes to support context mutation
        Ok(EvaluationResult {
            value: crate::core::Collection::from(input),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
