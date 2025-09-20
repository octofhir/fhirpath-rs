//! DefineVariable function implementation
//!
//! The defineVariable function allows defining a variable within an expression scope.
//! It evaluates the value expression and makes it available under the specified variable name.
//! Syntax: defineVariable(name, value)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// DefineVariable function evaluator
pub struct DefineVariableFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DefineVariableFunctionEvaluator {
    /// Create a new defineVariable function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
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
                            is_expression: false,
                            description: "The variable name to define".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "The value expression to assign to the variable (optional)".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(2),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
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
impl LazyFunctionEvaluator for DefineVariableFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "defineVariable function requires one or two arguments (name, [value])".to_string(),
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

        if context.get_variable(&variable_name).is_some() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0058,
                format!("Variable '{variable_name}' is already defined in this scope"),
            ));
        }

        if args.len() == 1 {
            // Single-parameter form: defineVariable(name) - assign current focus
            let variable_value = if input.is_empty() {
                FhirPathValue::Empty
            } else if input.len() == 1 {
                input.first().cloned().unwrap_or(FhirPathValue::Empty)
            } else {
                FhirPathValue::Collection(crate::core::Collection::from(input.clone()))
            };
            context.set_variable(variable_name, variable_value);
        } else {
            // Two-parameter form: defineVariable(name, value) - evaluate value expression
            // Create child context with current input as focus for value evaluation
            let child_context =
                context.create_child_context(crate::core::Collection::from(input.clone()));
            let value_result = evaluator.evaluate(&args[1], &child_context).await?;

            let variable_value = if value_result.value.is_empty() {
                FhirPathValue::Empty
            } else if value_result.value.len() == 1 {
                value_result
                    .value
                    .iter()
                    .next()
                    .cloned()
                    .unwrap_or(FhirPathValue::Empty)
            } else {
                FhirPathValue::Collection(value_result.value.clone())
            };
            context.set_variable(variable_name, variable_value);
        }

        // Return original focus (input collection) - variable is now available in current scope
        Ok(EvaluationResult {
            value: crate::core::Collection::from(input),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
