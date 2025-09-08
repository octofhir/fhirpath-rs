//! Lambda evaluation implementation for FHIRPath lambda expressions
//!
//! Lambda expressions in FHIRPath are used with functions like where(), select(), and sort().

use async_trait::async_trait;

use crate::{
    ast::ExpressionNode,
    core::{FhirPathError, FhirPathValue, Result, error_code::*},
    evaluator::{traits::LambdaEvaluator, EvaluationContext},
};

/// Implementation of LambdaEvaluator for lambda operations
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
        _lambda: &crate::ast::LambdaNode,
        _collection: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        Err(FhirPathError::evaluation_error(
            FP0055,
            "Lambda expressions not yet implemented".to_string(),
        ))
    }

    async fn map_lambda(
        &mut self,
        _lambda: &crate::ast::LambdaNode,
        _collection: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> Result<Vec<FhirPathValue>> {
        Err(FhirPathError::evaluation_error(
            FP0055,
            "Lambda mapping not yet implemented".to_string(),
        ))
    }

    fn create_lambda_context(
        &self,
        parent_context: &EvaluationContext,
        _lambda_param: Option<&str>,
        _param_value: &FhirPathValue,
    ) -> EvaluationContext {
        parent_context.clone()
    }
}