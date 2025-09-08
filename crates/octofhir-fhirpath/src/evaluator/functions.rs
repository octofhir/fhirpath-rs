//! Function evaluation implementation for FHIRPath function calls
//!
//! This module implements minimal FunctionEvaluator functionality to get the new engine working.
//! It will be expanded to include full function registry integration.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    core::{FhirPathError, FhirPathValue, Result, error_code::*},
    evaluator::{traits::{FunctionEvaluator}, EvaluationContext},
    registry::FunctionRegistry,
};

/// Implementation of FunctionEvaluator for basic operations
pub struct FunctionEvaluatorImpl {
    _registry: Arc<FunctionRegistry>,
}

impl FunctionEvaluatorImpl {
    /// Create a new function evaluator
    pub fn new(registry: Arc<FunctionRegistry>) -> Self {
        Self {
            _registry: registry,
        }
    }

    /// Handle basic collection functions
    fn handle_collection_functions(&self, name: &str, args: &[FhirPathValue]) -> Result<Option<FhirPathValue>> {
        match name {
            "first" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "first() function takes no arguments".to_string(),
                    ));
                }
                // first() should be called on the context, but for now return empty
                Ok(Some(FhirPathValue::Empty))
            },
            "last" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "last() function takes no arguments".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::Empty))
            },
            "count" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "count() function takes no arguments".to_string(),
                    ));
                }
                // count() should count the current context, but for now return 0
                Ok(Some(FhirPathValue::Integer(0)))
            },
            "empty" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "empty() function takes no arguments".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::Boolean(true)))
            },
            "exists" => {
                // exists() with optional condition
                if args.len() > 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "exists() function takes at most one argument".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::Boolean(false)))
            },
            _ => Ok(None), // Not handled by this function
        }
    }

    /// Handle basic string functions
    fn handle_string_functions(&self, name: &str, args: &[FhirPathValue]) -> Result<Option<FhirPathValue>> {
        match name {
            "toString" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toString() function takes no arguments".to_string(),
                    ));
                }
                // Should convert current context to string
                Ok(Some(FhirPathValue::String("".to_string())))
            },
            "length" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "length() function takes no arguments".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::Integer(0)))
            },
            _ => Ok(None),
        }
    }

    /// Handle basic math functions
    fn handle_math_functions(&self, name: &str, args: &[FhirPathValue]) -> Result<Option<FhirPathValue>> {
        match name {
            "abs" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "abs() function takes no arguments".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::Integer(0)))
            },
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl FunctionEvaluator for FunctionEvaluatorImpl {
    async fn call_function(
        &mut self,
        name: &str,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try different function categories
        if let Some(result) = self.handle_collection_functions(name, args)? {
            return Ok(result);
        }
        
        if let Some(result) = self.handle_string_functions(name, args)? {
            return Ok(result);
        }
        
        if let Some(result) = self.handle_math_functions(name, args)? {
            return Ok(result);
        }

        // Unknown function
        Err(FhirPathError::evaluation_error(
            FP0054,
            format!("Unknown function: '{}'", name),
        ))
    }

    async fn call_method(
        &mut self,
        object: &FhirPathValue,
        method: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // For method calls, we need to apply the method to the object
        match method {
            "first" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "first() method takes no arguments".to_string(),
                    ));
                }
                match object {
                    FhirPathValue::Collection(items) => {
                        Ok(items.first().cloned().unwrap_or(FhirPathValue::Empty))
                    },
                    FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                    single_value => Ok(single_value.clone()),
                }
            },
            "last" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "last() method takes no arguments".to_string(),
                    ));
                }
                match object {
                    FhirPathValue::Collection(items) => {
                        Ok(items.last().cloned().unwrap_or(FhirPathValue::Empty))
                    },
                    FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                    single_value => Ok(single_value.clone()),
                }
            },
            "count" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "count() method takes no arguments".to_string(),
                    ));
                }
                let count = match object {
                    FhirPathValue::Collection(items) => items.len(),
                    FhirPathValue::Empty => 0,
                    _ => 1,
                };
                Ok(FhirPathValue::Integer(count as i64))
            },
            "empty" => {
                if !args.is_empty() {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "empty() method takes no arguments".to_string(),
                    ));
                }
                let is_empty = matches!(object, FhirPathValue::Empty);
                Ok(FhirPathValue::Boolean(is_empty))
            },
            "exists" => {
                if args.len() > 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "exists() method takes at most one argument".to_string(),
                    ));
                }
                let exists = !matches!(object, FhirPathValue::Empty);
                Ok(FhirPathValue::Boolean(exists))
            },
            _ => {
                // Delegate to function call
                self.call_function(method, args, context).await
            }
        }
    }

    fn has_function(&self, name: &str) -> bool {
        matches!(name, "first" | "last" | "count" | "empty" | "exists" | "toString" | "length" | "abs")
    }

    fn get_function_metadata(&self, _name: &str) -> Option<&crate::registry::FunctionMetadata> {
        // For now, return None - we'll implement this later
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{registry::create_standard_registry, core::Collection};

    #[tokio::test]
    async fn test_method_calls() {
        let registry = Arc::new(create_standard_registry().await);
        let mut evaluator = FunctionEvaluatorImpl::new(registry);
        let context = EvaluationContext::new(Collection::empty());

        // Test first() on collection
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let result = evaluator.call_method(&collection, "first", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test last() on collection
        let result = evaluator.call_method(&collection, "last", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test count() on collection
        let result = evaluator.call_method(&collection, "count", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test empty() on collection
        let result = evaluator.call_method(&collection, "empty", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test exists() on collection
        let result = evaluator.call_method(&collection, "exists", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_method_calls_on_empty() {
        let registry = Arc::new(create_standard_registry().await);
        let mut evaluator = FunctionEvaluatorImpl::new(registry);
        let context = EvaluationContext::new(Collection::empty());

        let empty_value = FhirPathValue::Empty;

        // Test methods on empty value
        let result = evaluator.call_method(&empty_value, "first", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        let result = evaluator.call_method(&empty_value, "count", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        let result = evaluator.call_method(&empty_value, "empty", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = evaluator.call_method(&empty_value, "exists", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_method_calls_on_single_value() {
        let registry = Arc::new(create_standard_registry().await);
        let mut evaluator = FunctionEvaluatorImpl::new(registry);
        let context = EvaluationContext::new(Collection::empty());

        let single_value = FhirPathValue::String("test".to_string());

        // Test methods on single value
        let result = evaluator.call_method(&single_value, "first", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("test".to_string()));

        let result = evaluator.call_method(&single_value, "count", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        let result = evaluator.call_method(&single_value, "empty", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        let result = evaluator.call_method(&single_value, "exists", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}