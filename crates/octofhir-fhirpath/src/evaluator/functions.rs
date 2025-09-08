//! Function evaluation implementation for FHIRPath function calls
//!
//! This module implements minimal FunctionEvaluator functionality to get the new engine working.
//! It will be expanded to include full function registry integration.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    core::{FhirPathError, FhirPathValue, Result, error_code::*},
    evaluator::{
        EvaluationContext,
        metadata_functions::MetadataFunctionEvaluator,
        traits::{FunctionEvaluator, MetadataAwareFunctionEvaluator},
    },
    path::CanonicalPath,
    registry::FunctionRegistry,
    typing::{TypeResolver, type_utils},
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue, collection_utils},
};

/// Implementation of FunctionEvaluator for basic operations
pub struct FunctionEvaluatorImpl {
    pub function_registry: Arc<FunctionRegistry>,
    pub model_provider: Arc<dyn crate::core::ModelProvider>,
}

impl FunctionEvaluatorImpl {
    /// Create a new function evaluator
    pub fn new(
        registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn crate::core::ModelProvider>,
    ) -> Self {
        Self {
            function_registry: registry,
            model_provider,
        }
    }

    /// Handle iif function - if-then-else conditional
    async fn handle_iif(
        &mut self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 3 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "iif() requires exactly 3 arguments (condition, true_value, false_value)"
                    .to_string(),
            ));
        }

        let condition = &args[0];
        let true_value = &args[1];
        let false_value = &args[2];

        match condition {
            FhirPathValue::Boolean(true) => Ok(true_value.clone()),
            FhirPathValue::Boolean(false) => Ok(false_value.clone()),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                "iif condition must evaluate to a boolean".to_string(),
            )),
        }
    }

    /// Handle where function for filtering collections
    async fn handle_where(
        &mut self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // where() needs to be called as a method on a collection
        Err(FhirPathError::evaluation_error(
            FP0053,
            "where() must be called as a method on a collection".to_string(),
        ))
    }

    /// Handle where method on collections
    async fn handle_where_method(
        &mut self,
        object: &FhirPathValue,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "where() requires exactly 1 argument (condition expression)".to_string(),
            ));
        }

        // For now, we can't properly evaluate lambda expressions without the AST
        // This is a simplified implementation that always returns empty
        // TODO: Implement proper lambda evaluation when AST is available
        match object {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(_items) => {
                // TODO: Filter items based on condition
                Ok(FhirPathValue::Empty)
            }
            _single => {
                // TODO: Evaluate condition on single item
                Ok(FhirPathValue::Empty)
            }
        }
    }

    /// Handle select function for mapping collections
    async fn handle_select(
        &mut self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        Err(FhirPathError::evaluation_error(
            FP0053,
            "select() must be called as a method on a collection".to_string(),
        ))
    }

    /// Handle select method on collections
    async fn handle_select_method(
        &mut self,
        object: &FhirPathValue,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "select() requires exactly 1 argument (mapping expression)".to_string(),
            ));
        }

        // TODO: Implement proper lambda evaluation
        match object {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Handle sort function
    async fn handle_sort(
        &mut self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        Err(FhirPathError::evaluation_error(
            FP0053,
            "sort() must be called as a method on a collection".to_string(),
        ))
    }

    /// Handle sort method on collections
    async fn handle_sort_method(
        &mut self,
        object: &FhirPathValue,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "sort() takes at most 1 argument (sort expression)".to_string(),
            ));
        }

        // TODO: Implement proper sorting with lambda expression
        Ok(object.clone())
    }

    /// Handle aggregate function
    async fn handle_aggregate(
        &mut self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        Err(FhirPathError::evaluation_error(
            FP0053,
            "aggregate() must be called as a method on a collection".to_string(),
        ))
    }

    /// Handle aggregate method on collections
    async fn handle_aggregate_method(
        &mut self,
        object: &FhirPathValue,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() < 1 || args.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "aggregate() requires 1-2 arguments (initialValue, expression)".to_string(),
            ));
        }

        // TODO: Implement proper aggregation with lambda expression
        match object {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(args[0].clone()), // Return initial value for now
        }
    }

    /// Bridge method to call function with metadata awareness
    pub async fn call_function_with_metadata_bridge(
        &mut self,
        name: &str,
        args: &[WrappedCollection],
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Use the metadata-aware function evaluator
        let mut metadata_evaluator = MetadataFunctionEvaluator::new(
            self.function_registry.clone(),
            self.model_provider.clone(),
        );
        metadata_evaluator
            .call_function_with_metadata(name, args, context, resolver)
            .await
    }

    /// Bridge method to call method with metadata awareness
    pub async fn call_method_with_metadata_bridge(
        &mut self,
        object: &WrappedCollection,
        method: &str,
        args: &[WrappedCollection],
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Use the metadata-aware function evaluator
        let mut metadata_evaluator = MetadataFunctionEvaluator::new(
            self.function_registry.clone(),
            self.model_provider.clone(),
        );
        metadata_evaluator
            .call_method_with_metadata(object, method, args, context, resolver)
            .await
    }

    /// Convert plain function result to wrapped collection
    pub async fn wrap_function_result(
        &self,
        result: FhirPathValue,
        _function_name: &str,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Infer result type
        let result_type = type_utils::fhirpath_value_to_fhir_type(&result);
        let result_path = CanonicalPath::empty(); // Functions create new paths

        match result {
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            FhirPathValue::Collection(values) => {
                let wrapped_values: Vec<WrappedValue> = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let metadata = ValueMetadata {
                            fhir_type: result_type.clone(),
                            resource_type: None,
                            path: result_path.append_index(i),
                            index: Some(i),
                        };
                        WrappedValue::new(value, metadata)
                    })
                    .collect();
                Ok(wrapped_values)
            }
            single_value => {
                let metadata = ValueMetadata {
                    fhir_type: result_type,
                    resource_type: None,
                    path: result_path,
                    index: None,
                };
                Ok(collection_utils::single(WrappedValue::new(
                    single_value,
                    metadata,
                )))
            }
        }
    }
}

#[async_trait]
impl FunctionEvaluator for FunctionEvaluatorImpl {
    async fn call_function(
        &mut self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle special lambda-based functions that can't be handled by registry
        match name {
            "iif" => self.handle_iif(args, context).await,
            "where" => self.handle_where(args, context).await,
            "select" => self.handle_select(args, context).await,
            "sort" => self.handle_sort(args, context).await,
            "aggregate" => self.handle_aggregate(args, context).await,
            _ => {
                // Find the function in the registry
                // Try async function first (for most functions)
                if let Some((async_function, _metadata)) =
                    self.function_registry.get_async_function(name)
                {
                    // Create function context
                    let input = if context.start_context.is_empty() {
                        FhirPathValue::Empty
                    } else if context.start_context.len() == 1 {
                        context.start_context.first().unwrap().clone()
                    } else {
                        FhirPathValue::Collection(context.start_context.iter().cloned().collect())
                    };

                    let arguments = match args.len() {
                        0 => FhirPathValue::Empty,
                        1 => args[0].clone(),
                        _ => FhirPathValue::Collection(args.to_vec()),
                    };

                    let function_context = crate::registry::FunctionContext {
                        input,
                        arguments,
                        model_provider: &*self.model_provider,
                        variables: &context.variables,
                        resource_context: None,
                        terminology: None,
                    };

                    // Call the async registry function
                    async_function(&function_context).await
                } else if let Some((sync_function, _metadata)) =
                    self.function_registry.get_sync_function(name)
                {
                    // Fall back to sync function if no async version
                    let input = if context.start_context.is_empty() {
                        FhirPathValue::Empty
                    } else if context.start_context.len() == 1 {
                        context.start_context.first().unwrap().clone()
                    } else {
                        FhirPathValue::Collection(context.start_context.iter().cloned().collect())
                    };

                    let arguments = match args.len() {
                        0 => FhirPathValue::Empty,
                        1 => args[0].clone(),
                        _ => FhirPathValue::Collection(args.to_vec()),
                    };

                    let function_context = crate::registry::FunctionContext {
                        input,
                        arguments,
                        model_provider: &*self.model_provider,
                        variables: &context.variables,
                        resource_context: None,
                        terminology: None,
                    };

                    // Call the sync registry function
                    sync_function(&function_context)
                } else {
                    // Unknown function
                    Err(FhirPathError::evaluation_error(
                        FP0054,
                        format!("Unknown function: '{}'", name),
                    ))
                }
            }
        }
    }

    async fn call_method(
        &mut self,
        object: &FhirPathValue,
        method: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle special lambda-based methods using metadata-aware evaluation
        match method {
            "where" | "select" | "sort" | "aggregate" => {
                // Convert to WrappedCollection for metadata-aware processing
                let wrapped_object = self.convert_value_to_wrapped_collection(object, context)?;
                let wrapped_args = args
                    .iter()
                    .map(|arg| self.convert_value_to_wrapped_collection(arg, context))
                    .collect::<Result<Vec<_>>>()?;

                // Use the shared TypeResolver from context (TODO: get from evaluator/context)
                let resolver = crate::typing::TypeResolver::new(self.model_provider.clone());

                // Call the metadata-aware method evaluator
                let wrapped_result = self
                    .call_method_with_metadata_bridge(
                        &wrapped_object,
                        method,
                        &wrapped_args,
                        context,
                        &resolver,
                    )
                    .await?;

                // Convert back to FhirPathValue
                self.convert_wrapped_collection_to_value(wrapped_result)
            }
            _ => {
                // For other method calls, create a new context with the object as the current context
                let mut method_context = context.clone();
                method_context.start_context = crate::core::Collection::single(object.clone());

                // Call the method as a function with the object as context
                self.call_function(method, args, &method_context).await
            }
        }
    }

    fn has_function(&self, name: &str) -> bool {
        // Check special lambda functions first
        matches!(name, "iif" | "where" | "select" | "sort" | "aggregate")
            || self.function_registry.get_async_function(name).is_some()
            || self.function_registry.get_sync_function(name).is_some()
    }

    fn get_function_metadata(&self, name: &str) -> Option<&crate::registry::FunctionMetadata> {
        // Try async function first, then sync
        if let Some((_, _metadata)) = self.function_registry.get_async_function(name) {
            // We can't return a reference to a temporary value, so return None for now
            // TODO: Redesign this to avoid lifetime issues
            None
        } else if let Some((_, _metadata)) = self.function_registry.get_sync_function(name) {
            // We can't return a reference to a temporary value, so return None for now
            // TODO: Redesign this to avoid lifetime issues
            None
        } else {
            None
        }
    }
}

impl FunctionEvaluatorImpl {
    /// Convert FhirPathValue to WrappedCollection with appropriate metadata
    fn convert_value_to_wrapped_collection(
        &self,
        value: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        // Create basic metadata - in a real implementation this would be more sophisticated
        let metadata = crate::wrapped::ValueMetadata {
            fhir_type: self.infer_fhir_type_from_value(value),
            resource_type: None,
            path: crate::path::CanonicalPath::empty(), // TODO: derive from context
            index: None,
        };

        let wrapped_values = match value {
            FhirPathValue::Collection(items) => items
                .iter()
                .map(|item| {
                    let item_metadata = crate::wrapped::ValueMetadata {
                        fhir_type: self.infer_fhir_type_from_value(item),
                        resource_type: None,
                        path: metadata.path.clone(),
                        index: None,
                    };
                    crate::wrapped::WrappedValue::new(item.clone(), item_metadata)
                })
                .collect(),
            single_value => {
                vec![crate::wrapped::WrappedValue::new(
                    single_value.clone(),
                    metadata,
                )]
            }
        };

        Ok(wrapped_values)
    }

    /// Convert WrappedCollection back to FhirPathValue
    fn convert_wrapped_collection_to_value(
        &self,
        wrapped_collection: WrappedCollection,
    ) -> Result<FhirPathValue> {
        if wrapped_collection.is_empty() {
            Ok(FhirPathValue::Empty)
        } else if wrapped_collection.len() == 1 {
            Ok(wrapped_collection
                .into_iter()
                .next()
                .unwrap()
                .as_plain()
                .clone())
        } else {
            let plain_values: Vec<FhirPathValue> = wrapped_collection
                .into_iter()
                .map(|wrapped| wrapped.as_plain().clone())
                .collect();
            Ok(FhirPathValue::Collection(plain_values))
        }
    }

    /// Infer FHIR type from FhirPathValue
    fn infer_fhir_type_from_value(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(_) => "string".to_string(),
            FhirPathValue::Integer(_) => "integer".to_string(),
            FhirPathValue::Decimal(_) => "decimal".to_string(),
            FhirPathValue::Boolean(_) => "boolean".to_string(),
            FhirPathValue::Date(_) => "date".to_string(),
            FhirPathValue::DateTime(_) => "dateTime".to_string(),
            FhirPathValue::Time(_) => "time".to_string(),
            FhirPathValue::Quantity { .. } => "Quantity".to_string(),
            FhirPathValue::Id(_) => "id".to_string(),
            FhirPathValue::Base64Binary(_) => "base64Binary".to_string(),
            FhirPathValue::Uri(_) => "uri".to_string(),
            FhirPathValue::Url(_) => "url".to_string(),
            FhirPathValue::TypeInfoObject { .. } => "TypeInfo".to_string(),
            FhirPathValue::Resource(_) => "Resource".to_string(), // TODO: detect actual resource type
            FhirPathValue::JsonValue(_) => "unknown".to_string(),
            FhirPathValue::Collection(_) => "collection".to_string(),
            FhirPathValue::Empty => "empty".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::Collection, registry::create_standard_registry};

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
        let result = evaluator
            .call_method(&collection, "first", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test last() on collection
        let result = evaluator
            .call_method(&collection, "last", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test count() on collection
        let result = evaluator
            .call_method(&collection, "count", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test empty() on collection
        let result = evaluator
            .call_method(&collection, "empty", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test exists() on collection
        let result = evaluator
            .call_method(&collection, "exists", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_method_calls_on_empty() {
        let registry = Arc::new(create_standard_registry().await);
        let mut evaluator = FunctionEvaluatorImpl::new(registry);
        let context = EvaluationContext::new(Collection::empty());

        let empty_value = FhirPathValue::Empty;

        // Test methods on empty value
        let result = evaluator
            .call_method(&empty_value, "first", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        let result = evaluator
            .call_method(&empty_value, "count", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        let result = evaluator
            .call_method(&empty_value, "empty", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = evaluator
            .call_method(&empty_value, "exists", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_method_calls_on_single_value() {
        let registry = Arc::new(create_standard_registry().await);
        let mut evaluator = FunctionEvaluatorImpl::new(registry);
        let context = EvaluationContext::new(Collection::empty());

        let single_value = FhirPathValue::String("test".to_string());

        // Test methods on single value
        let result = evaluator
            .call_method(&single_value, "first", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::String("test".to_string()));

        let result = evaluator
            .call_method(&single_value, "count", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        let result = evaluator
            .call_method(&single_value, "empty", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        let result = evaluator
            .call_method(&single_value, "exists", &[], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
