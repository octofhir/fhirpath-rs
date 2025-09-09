//! Lambda evaluation implementation for FHIRPath lambda expressions
//!
//! Lambda expressions in FHIRPath are used with functions like where(), select(), and sort().
//! This implementation preserves type information and metadata throughout lambda evaluation.

use async_trait::async_trait;

use crate::{
    ast::LambdaNode,
    core::{FhirPathError, Result, error_code::*},
    evaluator::{EvaluationContext, traits::LambdaEvaluator},
    typing::TypeResolver,
    wrapped::{WrappedCollection, WrappedValue},
};

/// Implementation of lambda evaluator for lambda operations with metadata preservation
pub struct LambdaEvaluatorImpl;

impl LambdaEvaluatorImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LambdaEvaluatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LambdaEvaluator for LambdaEvaluatorImpl {
    async fn evaluate_lambda(
        &mut self,
        lambda: &LambdaNode,
        collection: &WrappedCollection,
        context: &EvaluationContext,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match lambda.parameter.as_deref() {
            Some("$this") | None => {
                // Standard lambda evaluation - evaluate expression for each item
                let results = Vec::new();

                for wrapped_value in collection {
                    // Create child context with lambda parameter bound to current item
                    let mut lambda_context = context.clone();

                    // If there's an explicit parameter, bind it
                    if let Some(param) = &lambda.parameter {
                        // Convert wrapped value to plain for context binding
                        let plain_value = wrapped_value.as_plain().clone();
                        lambda_context.set_variable(param.to_string(), plain_value);
                    }

                    // Update context start_context to current item for implicit $this
                    lambda_context.start_context =
                        crate::core::Collection::single(wrapped_value.as_plain().clone());

                    // For now, return error indicating not yet implemented
                    // TODO: Need access to CompositeEvaluator to evaluate lambda.expression
                    return Err(FhirPathError::evaluation_error(
                        FP0055,
                        "Lambda expressions with metadata not yet fully implemented - need evaluator access".to_string(),
                    ));
                }

                Ok(results)
            }
            Some(param) => {
                // Named parameter lambda evaluation
                let results = Vec::new();

                for wrapped_value in collection {
                    // Create child context with named parameter bound to current item
                    let mut lambda_context = context.clone();
                    let plain_value = wrapped_value.as_plain().clone();
                    lambda_context.set_variable(param.to_string(), plain_value);

                    // For now, return error indicating not yet implemented
                    // TODO: Need access to CompositeEvaluator to evaluate lambda.expression
                    return Err(FhirPathError::evaluation_error(
                        FP0055,
                        format!(
                            "Named parameter lambda expressions ({}) with metadata not yet fully implemented",
                            param
                        ),
                    ));
                }

                Ok(results)
            }
        }
    }

    async fn map_lambda(
        &mut self,
        lambda: &LambdaNode,
        collection: &WrappedCollection,
        context: &EvaluationContext,
        _resolver: &TypeResolver,
    ) -> Result<Vec<WrappedCollection>> {
        // Map lambda is similar to evaluate lambda but returns individual results
        let results = Vec::new();

        for wrapped_value in collection {
            // Create child context for each item
            let mut lambda_context = context.clone();

            // Handle parameter binding
            match lambda.parameter.as_deref() {
                Some("$this") | None => {
                    // Update context start_context for implicit $this
                    lambda_context.start_context =
                        crate::core::Collection::single(wrapped_value.as_plain().clone());
                }
                Some(param) => {
                    // Bind named parameter
                    let plain_value = wrapped_value.as_plain().clone();
                    lambda_context.set_variable(param.to_string(), plain_value);
                }
            }

            // For now, return error indicating not yet implemented
            // TODO: Need access to CompositeEvaluator to evaluate lambda.expression
            return Err(FhirPathError::evaluation_error(
                FP0055,
                "Lambda mapping with metadata not yet fully implemented - need evaluator access"
                    .to_string(),
            ));
        }

        Ok(results)
    }

    async fn create_lambda_context(
        &self,
        parent_context: &EvaluationContext,
        lambda_param: Option<&str>,
        param_value: &WrappedValue,
        _resolver: &TypeResolver,
    ) -> Result<EvaluationContext> {
        let mut lambda_context = parent_context.clone();

        // Handle parameter binding with metadata preservation
        match lambda_param {
            Some("$this") | None => {
                // For $this or implicit parameter, update the start_context
                lambda_context.start_context =
                    crate::core::Collection::single(param_value.as_plain().clone());
            }
            Some(param) => {
                // For named parameters, add to variable bindings
                lambda_context.set_variable(param.to_string(), param_value.as_plain().clone());
            }
        }

        Ok(lambda_context)
    }
}
