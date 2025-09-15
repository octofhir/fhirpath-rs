//! Lambda functions implementation following FHIR specification
//!
//! Implements where(), aggregate(), and defineVariable() functions with proper variable scoping

use std::sync::Arc;

use crate::{
    core::{FhirPathValue, FhirPathError, Collection, Result},
    evaluator::{EvaluationContext, evaluator::Evaluator},
    registry::{FunctionMetadata, FunctionCategory, ParameterMetadata},
    ast::ExpressionNode,
};

use octofhir_fhir_model::{ModelProvider, TerminologyProvider};

/// Lambda function evaluator for functions that require variable context management
pub struct LambdaFunctionEvaluator {
    evaluator: Arc<dyn Evaluator>,
}

impl LambdaFunctionEvaluator {
    pub fn new(evaluator: Arc<dyn Evaluator>) -> Self {
        Self { evaluator }
    }

    /// Implement where() function according to FHIRPath spec
    /// where(criteria : expression) : collection
    pub async fn where_function(
        &self,
        input: &Collection,
        condition_expr: &ExpressionNode,
        context: &EvaluationContext,
        model_provider: &dyn ModelProvider,
        terminology_provider: Option<&dyn TerminologyProvider>,
    ) -> Result<Collection> {
        let mut results = Vec::new();

        for (index, item) in input.iter().enumerate() {
            // Create iterator context with proper variable isolation
            let iter_context = context.create_iterator_context(item.clone(), index);

            // Evaluate condition in isolated context
            let condition_result = self.evaluator
                .evaluate(condition_expr, &iter_context, model_provider, terminology_provider)
                .await?;

            // Check if condition is true
            if self.is_true(&condition_result)? {
                results.push(item.clone());
            }
        }

        Ok(Collection::from_values(results))
    }

    /// Implement aggregate() function according to FHIRPath spec
    /// aggregate(iterator : expression [, initial : expression]) : value
    pub async fn aggregate_function(
        &self,
        input: &Collection,
        iterator_expr: &ExpressionNode,
        initial_expr: Option<&ExpressionNode>,
        context: &EvaluationContext,
        model_provider: &dyn ModelProvider,
        terminology_provider: Option<&dyn TerminologyProvider>,
    ) -> Result<Collection> {
        // Evaluate initial value if provided
        let mut accumulator = if let Some(init_expr) = initial_expr {
            let init_result = self.evaluator
                .evaluate(init_expr, context, model_provider, terminology_provider)
                .await?;

            // Get single value from initial expression
            init_result.first().cloned().unwrap_or(FhirPathValue::Empty)
        } else {
            FhirPathValue::Empty
        };

        for (index, item) in input.iter().enumerate() {
            // Create iterator context
            let mut iter_context = context.create_iterator_context(item.clone(), index);

            // Set accumulator variable for aggregate function
            iter_context.set_user_variable("acc".to_string(), accumulator.clone())?;

            // Evaluate iterator expression
            let iter_result = self.evaluator
                .evaluate(iterator_expr, &iter_context, model_provider, terminology_provider)
                .await?;

            // Update accumulator with result
            accumulator = iter_result.first().cloned().unwrap_or(FhirPathValue::Empty);
        }

        Ok(Collection::single(accumulator))
    }

    /// Implement defineVariable() function according to FHIRPath spec
    /// defineVariable(name : string [, value : expression]) : collection
    pub async fn define_variable_function(
        &self,
        input: &Collection,
        name_expr: &ExpressionNode,
        value_expr: Option<&ExpressionNode>,
        context: &mut EvaluationContext,
        model_provider: &dyn ModelProvider,
        terminology_provider: Option<&dyn TerminologyProvider>,
    ) -> Result<Collection> {
        // Get variable name
        let var_name = match name_expr {
            ExpressionNode::Literal(literal_node) => {
                match &literal_node.value {
                    crate::ast::LiteralValue::String(name) => name.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0001,
                        "Variable name must be a string literal",
                    )),
                }
            }
            _ => {
                // Dynamic variable name - evaluate expression
                let name_result = self.evaluator
                    .evaluate(name_expr, context, model_provider, terminology_provider)
                    .await?;

                match name_result.first() {
                    Some(FhirPathValue::String(name)) => name.clone(),
                    _ => return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0001,
                        "Variable name must evaluate to string",
                    )),
                }
            }
        };

        // Get variable value
        let var_value = if let Some(val_expr) = value_expr {
            let val_result = self.evaluator
                .evaluate(val_expr, context, model_provider, terminology_provider)
                .await?;

            val_result.first().cloned().unwrap_or(FhirPathValue::Empty)
        } else {
            // If no value provided, use input
            input.first().cloned().unwrap_or(FhirPathValue::Empty)
        };

        // Set variable with redefinition protection
        context.set_user_variable(var_name, var_value)?;

        // Return input unchanged (defineVariable returns the input)
        Ok(input.clone())
    }

    /// Check if collection represents true in FHIRPath boolean context
    fn is_true(&self, collection: &Collection) -> Result<bool> {
        if collection.is_empty() {
            return Ok(false);
        }

        // If collection has single boolean true, return true
        if collection.len() == 1 {
            if let Some(FhirPathValue::Boolean(b)) = collection.first() {
                return Ok(*b);
            }
        }

        // Non-empty non-boolean collection is truthy
        Ok(!collection.is_empty())
    }
}

/// Create function metadata for where() function
pub fn where_function_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "where".to_string(),
        category: FunctionCategory::Collection,
        description: "Filters input collection based on condition expression".to_string(),
        parameters: vec![
            ParameterMetadata {
                name: "criteria".to_string(),
                type_constraint: Some("expression".to_string()),
                is_optional: false,
                description: "Boolean expression to filter items".to_string(),
            }
        ],
        return_type: Some("collection".to_string()),
        is_async: true,
        examples: vec![
            "Patient.name.where(use = 'official')".to_string(),
            "Bundle.entry.where(resource is Patient)".to_string(),
        ],
        requires_model_provider: false,
        requires_terminology_provider: false,
        does_not_propagate_empty: false,
    }
}

/// Create function metadata for aggregate() function
pub fn aggregate_function_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "aggregate".to_string(),
        category: FunctionCategory::Collection,
        description: "Aggregates collection using iterator expression with accumulator".to_string(),
        parameters: vec![
            ParameterMetadata {
                name: "iterator".to_string(),
                type_constraint: Some("expression".to_string()),
                is_optional: false,
                description: "Expression evaluated for each item with $acc variable".to_string(),
            },
            ParameterMetadata {
                name: "initial".to_string(),
                type_constraint: Some("expression".to_string()),
                is_optional: true,
                description: "Initial value for accumulator".to_string(),
            }
        ],
        return_type: Some("value".to_string()),
        is_async: true,
        examples: vec![
            "Patient.name.aggregate($acc + ' ' + family)".to_string(),
            "(1 | 2 | 3).aggregate($acc + $this, 0)".to_string(),
        ],
        requires_model_provider: false,
        requires_terminology_provider: false,
        does_not_propagate_empty: true,
    }
}

/// Create function metadata for defineVariable() function
pub fn define_variable_function_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "defineVariable".to_string(),
        category: FunctionCategory::Utility,
        description: "Defines a variable in current context scope".to_string(),
        parameters: vec![
            ParameterMetadata {
                name: "name".to_string(),
                type_constraint: Some("string".to_string()),
                is_optional: false,
                description: "Name of the variable to define".to_string(),
            },
            ParameterMetadata {
                name: "value".to_string(),
                type_constraint: Some("expression".to_string()),
                is_optional: true,
                description: "Value expression (defaults to input)".to_string(),
            }
        ],
        return_type: Some("collection".to_string()),
        is_async: true,
        examples: vec![
            "Patient.name.defineVariable('patientName').family".to_string(),
            "Patient.defineVariable('pid', id).name".to_string(),
        ],
        requires_model_provider: false,
        requires_terminology_provider: false,
        does_not_propagate_empty: false,
    }
}