//! FHIRPath engine - the main entry point for FHIRPath evaluation

use super::error::Result;
use crate::ast::ExpressionNode;
use crate::evaluator::FhirPathEngine as EvaluatorEngine;
use crate::model::FhirPathValue;
use crate::parser::parse_expression;
use crate::registry::create_standard_registries;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Main FHIRPath engine for parsing and evaluating expressions
#[derive(Clone)]
pub struct FhirPathEngine {
    /// The underlying evaluator engine
    evaluator: EvaluatorEngine,
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
        let (functions, operators) = create_standard_registries();
        let evaluator = EvaluatorEngine::with_registries(Arc::new(functions), Arc::new(operators));

        Self {
            evaluator,
            expression_cache: HashMap::new(),
            max_cache_size: 1000,
        }
    }

    /// Evaluate an FHIRPath expression against input data
    pub async fn evaluate(&mut self, expression: &str, input_data: Value) -> Result<FhirPathValue> {
        // Handle parse errors by returning empty collection per FHIRPath spec
        let ast = match self.get_or_compile_expression(expression) {
            Ok(ast) => ast.clone(),
            Err(e) => {
                // Per FHIRPath spec, syntax errors should return empty collection
                if e.to_string().contains("parse error")
                    || e.to_string().contains("Parse error")
                    || e.to_string().contains("Unclosed")
                    || e.to_string().contains("Unexpected")
                    || e.to_string().contains("Expected")
                {
                    return Ok(FhirPathValue::collection(vec![]));
                } else {
                    return Err(e);
                }
            }
        };

        let input_value = FhirPathValue::from(input_data);

        match self.evaluator.evaluate(&ast, input_value).await {
            Ok(result) => Ok(result),
            Err(eval_error) => Err(crate::error::FhirPathError::evaluation_error(
                eval_error.to_string(),
            )),
        }
    }

    /// Get or compile an expression, using cache when possible
    fn get_or_compile_expression(&mut self, expression: &str) -> Result<&ExpressionNode> {
        if !self.expression_cache.contains_key(expression) {
            let ast = parse_expression(expression)
                .map_err(|e| crate::error::FhirPathError::parse_error(0, e.to_string()))?;
            if self.expression_cache.len() >= self.max_cache_size {
                self.expression_cache.clear();
            }
            self.expression_cache.insert(expression.to_string(), ast);
        }
        Ok(self.expression_cache.get(expression).unwrap())
    }
}

/// Alias for compatibility with original API
pub type Engine = FhirPathEngine;
