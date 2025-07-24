//! FHIRPath engine - the main entry point for FHIRPath evaluation

use fhirpath_ast::ExpressionNode;
use crate::error::Result;
use crate::evaluator::{evaluate_ast, EvaluationContext};
use crate::value_ext::FhirPathValue;
use crate::parser::parse_expression;
use serde_json::Value;
use std::collections::HashMap;

/// Main FHIRPath engine for parsing and evaluating expressions
#[derive(Debug, Clone)]
pub struct FhirPathEngine {
    /// Cached compiled expressions for performance
    expression_cache: HashMap<String, ExpressionNode>,
    /// Maximum cache size to prevent memory issues
    max_cache_size: usize,
}

impl Default for FhirPathEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FhirPathEngine {
    /// Create a new FHIRPath engine
    pub fn new() -> Self {
        Self {
            expression_cache: HashMap::new(),
            max_cache_size: 1000,
        }
    }

    /// Evaluate an FHIRPath expression against input data
    pub fn evaluate(&mut self, expression: &str, input_data: Value) -> Result<FhirPathValue> {
        let ast = self.get_or_compile_expression(expression)?;
        let context = EvaluationContext::from_json(input_data);
        evaluate_ast(&ast, &context)
    }

    /// Get or compile an expression, using cache when possible
    fn get_or_compile_expression(&mut self, expression: &str) -> Result<&ExpressionNode> {
        if !self.expression_cache.contains_key(expression) {
            let ast = parse_expression(expression)?;
            if self.expression_cache.len() >= self.max_cache_size {
                self.expression_cache.clear();
            }
            self.expression_cache.insert(expression.to_string(), ast);
        }
        Ok(self.expression_cache.get(expression).unwrap())
    }
}
