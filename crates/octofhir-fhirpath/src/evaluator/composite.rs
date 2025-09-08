//! Composite evaluator with full metadata support
//!
//! This module provides the main CompositeEvaluator that
//! integrates all metadata-aware evaluators for comprehensive rich evaluation.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    ast::ExpressionNode,
    core::{FhirPathValue, ModelProvider, Result},
    evaluator::{
        EvaluationContext, MetadataCollectionEvaluator, MetadataCoreEvaluator,
        MetadataFunctionEvaluator, MetadataNavigator,
        config::EngineConfig,
        traits::{
            ExpressionEvaluator, MetadataAwareCollectionEvaluator, MetadataAwareEvaluator,
            MetadataAwareFunctionEvaluator, MetadataAwareNavigator,
        },
    },
    registry::FunctionRegistry,
    typing::{TypeResolver, TypeResolverFactory},
    wrapped::{WrappedCollection, collection_utils},
};

/// Composite evaluator with metadata support
pub struct CompositeEvaluator {
    /// Metadata-aware core evaluator
    core_evaluator: MetadataCoreEvaluator,
    /// Metadata-aware navigator
    navigator: MetadataNavigator,
    /// Metadata-aware function evaluator
    function_evaluator: MetadataFunctionEvaluator,
    /// Standard operator evaluator (can be enhanced later)
    operator_evaluator: Box<dyn crate::evaluator::OperatorEvaluator + Send + Sync>,
    /// Metadata-aware collection evaluator
    collection_evaluator: MetadataCollectionEvaluator,
    /// Standard lambda evaluator (can be enhanced later)
    lambda_evaluator: Box<dyn crate::evaluator::LambdaEvaluatorTrait + Send + Sync>,
    /// Model provider for type resolution
    model_provider: Arc<dyn ModelProvider>,
    /// Function registry for function resolution
    function_registry: Arc<FunctionRegistry>,
    /// Type resolver for metadata operations
    type_resolver: TypeResolver,
    /// Engine configuration
    config: EngineConfig,
}

impl CompositeEvaluator {
    pub async fn new(
        _core_evaluator: Box<dyn ExpressionEvaluator + Send + Sync>,
        _navigator: Box<dyn crate::evaluator::ValueNavigator + Send + Sync>,
        _function_evaluator: Box<dyn crate::evaluator::FunctionEvaluator + Send + Sync>,
        operator_evaluator: Box<dyn crate::evaluator::OperatorEvaluator + Send + Sync>,
        _collection_evaluator: Box<dyn crate::evaluator::CollectionEvaluator + Send + Sync>,
        lambda_evaluator: Box<dyn crate::evaluator::LambdaEvaluatorTrait + Send + Sync>,
        model_provider: Arc<dyn ModelProvider>,
        function_registry: Arc<FunctionRegistry>,
        config: EngineConfig,
    ) -> Self {
        let type_resolver = TypeResolverFactory::create(model_provider.clone());

        Self {
            core_evaluator: MetadataCoreEvaluator::new(),
            navigator: MetadataNavigator::new(),
            function_evaluator: MetadataFunctionEvaluator::new(
                function_registry.clone(),
                model_provider.clone(),
            ),
            operator_evaluator,
            collection_evaluator: MetadataCollectionEvaluator::new(),
            lambda_evaluator,
            model_provider: model_provider.clone(),
            function_registry,
            type_resolver,
            config,
        }
    }

    /// Get the model provider reference
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
    }

    /// Get the function registry reference  
    pub fn function_registry(&self) -> &Arc<FunctionRegistry> {
        &self.function_registry
    }

    /// Evaluate expression with full metadata support
    pub async fn evaluate_with_metadata(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        // Always use metadata-aware evaluation

        // Dispatch to metadata-aware evaluation based on expression type
        match expr {
            ExpressionNode::Identifier(_)
            | ExpressionNode::Literal(_)
            | ExpressionNode::Variable(_) => {
                self.core_evaluator
                    .evaluate_with_metadata(expr, context, &self.type_resolver)
                    .await
            }
            ExpressionNode::PropertyAccess(prop) => {
                let object_result =
                    Box::pin(self.evaluate_with_metadata(&prop.object, context)).await?;
                let mut combined_result = Vec::new();

                for object_wrapped in object_result {
                    let nav_result = self
                        .navigator
                        .navigate_property_with_metadata(
                            &object_wrapped,
                            &prop.property,
                            &self.type_resolver,
                        )
                        .await?;
                    combined_result.extend(nav_result);
                }

                Ok(combined_result)
            }
            ExpressionNode::IndexAccess(idx) => {
                let object_result =
                    Box::pin(self.evaluate_with_metadata(&idx.object, context)).await?;
                let index_result =
                    Box::pin(self.evaluate_with_metadata(&idx.index, context)).await?;

                // Extract index value
                let index_value = index_result.first().and_then(|w| match w.as_plain() {
                    FhirPathValue::Integer(i) if *i >= 0 => Some(*i as usize),
                    _ => None,
                });

                if let Some(index) = index_value {
                    let mut indexed_results = Vec::new();

                    for object_wrapped in object_result {
                        if let Some(indexed) = self
                            .navigator
                            .navigate_index_with_metadata(
                                &object_wrapped,
                                index,
                                &self.type_resolver,
                            )
                            .await?
                        {
                            indexed_results.push(indexed);
                        }
                    }

                    Ok(indexed_results)
                } else {
                    Ok(collection_utils::empty())
                }
            }
            ExpressionNode::FunctionCall(func) => {
                let mut wrapped_args = Vec::new();
                for arg in &func.arguments {
                    let arg_result = Box::pin(self.evaluate_with_metadata(arg, context)).await?;
                    wrapped_args.push(arg_result);
                }

                self.function_evaluator
                    .call_function_with_metadata(
                        &func.name,
                        &wrapped_args,
                        context,
                        &self.type_resolver,
                    )
                    .await
            }
            ExpressionNode::MethodCall(method) => {
                let object_result =
                    Box::pin(self.evaluate_with_metadata(&method.object, context)).await?;

                let mut wrapped_args = Vec::new();
                for arg in &method.arguments {
                    let arg_result = Box::pin(self.evaluate_with_metadata(arg, context)).await?;
                    wrapped_args.push(arg_result);
                }

                self.function_evaluator
                    .call_method_with_metadata(
                        &object_result,
                        &method.method,
                        &wrapped_args,
                        context,
                        &self.type_resolver,
                    )
                    .await
            }
            ExpressionNode::Collection(coll) => {
                let mut element_collections = Vec::new();
                for element in &coll.elements {
                    let element_result =
                        Box::pin(self.evaluate_with_metadata(element, context)).await?;
                    element_collections.push(element_result);
                }

                self.collection_evaluator
                    .create_collection_with_metadata(element_collections, &self.type_resolver)
                    .await
            }
            ExpressionNode::Union(union) => {
                let left_result =
                    Box::pin(self.evaluate_with_metadata(&union.left, context)).await?;
                let right_result =
                    Box::pin(self.evaluate_with_metadata(&union.right, context)).await?;

                self.collection_evaluator
                    .union_collections_with_metadata(
                        &left_result,
                        &right_result,
                        &self.type_resolver,
                    )
                    .await
            }
            ExpressionNode::Filter(filter) => {
                let base_result =
                    Box::pin(self.evaluate_with_metadata(&filter.base, context)).await?;

                let mut metadata_coll_eval = self.collection_evaluator.clone();
                metadata_coll_eval
                    .filter_collection_with_metadata(
                        &base_result,
                        &filter.condition,
                        context,
                        &self.type_resolver,
                    )
                    .await
            }
            ExpressionNode::BinaryOperation(binop) => {
                let left_result =
                    Box::pin(self.evaluate_with_metadata(&binop.left, context)).await?;
                let right_result =
                    Box::pin(self.evaluate_with_metadata(&binop.right, context)).await?;

                // Convert to plain values for operator evaluation
                let left_plain = self.wrapped_to_plain(&left_result);
                let right_plain = self.wrapped_to_plain(&right_result);

                // Use standard operator evaluator and wrap result with metadata
                let result = self.operator_evaluator.evaluate_binary_op(
                    &left_plain,
                    &binop.operator,
                    &right_plain,
                )?;

                self.wrap_plain_result(result).await
            }
            ExpressionNode::UnaryOperation(unop) => {
                let operand_result =
                    Box::pin(self.evaluate_with_metadata(&unop.operand, context)).await?;

                // Convert to plain value for operator evaluation
                let operand_plain = self.wrapped_to_plain(&operand_result);

                // Use standard operator evaluator and wrap result with metadata
                let result = self
                    .operator_evaluator
                    .evaluate_unary_op(&unop.operator, &operand_plain)?;

                self.wrap_plain_result(result).await
            }
            ExpressionNode::Lambda(lambda) => {
                // For lambda expressions, we need to convert wrapped values to plain for compatibility
                // with the existing lambda evaluator
                let lambda_context = context.clone();

                // Convert the current context collection to a FhirPathValue for lambda evaluation
                let context_value = if context.start_context.is_empty() {
                    // In FHIRPath, empty contexts are represented as empty collections
                    FhirPathValue::Collection(Vec::new())
                } else if context.start_context.len() == 1 {
                    context.start_context.first().unwrap().clone()
                } else {
                    FhirPathValue::Collection(context.start_context.clone().into_vec())
                };

                // Use standard lambda evaluator and wrap result with metadata
                let result = self
                    .lambda_evaluator
                    .evaluate_lambda(lambda, &context_value, &lambda_context)
                    .await?;

                self.wrap_plain_result(result).await
            }
            ExpressionNode::Parenthesized(inner) => {
                // Simply evaluate the inner expression - parentheses don't change semantics
                Box::pin(self.evaluate_with_metadata(inner, context)).await
            }
            ExpressionNode::TypeCast(cast) => {
                let expr_result =
                    Box::pin(self.evaluate_with_metadata(&cast.expression, context)).await?;

                // Convert to plain value for type cast evaluation
                let expr_plain = self.wrapped_to_plain(&expr_result);

                // Perform type cast - for now, return the original value
                // TODO: Implement proper type casting logic
                let result = match cast.target_type.as_str() {
                    "string" => match &expr_plain {
                        FhirPathValue::String(s) => FhirPathValue::String(s.clone()),
                        FhirPathValue::Integer(i) => FhirPathValue::String(i.to_string()),
                        FhirPathValue::Decimal(d) => FhirPathValue::String(d.to_string()),
                        FhirPathValue::Boolean(b) => FhirPathValue::String(b.to_string()),
                        _ => expr_plain,
                    },
                    _ => expr_plain, // For now, return as-is for other casts
                };
                self.wrap_plain_result(result).await
            }
            ExpressionNode::TypeCheck(check) => {
                let expr_result =
                    Box::pin(self.evaluate_with_metadata(&check.expression, context)).await?;

                // Convert to plain value for type check evaluation
                let expr_plain = self.wrapped_to_plain(&expr_result);

                // Perform type check - basic implementation
                // TODO: Implement proper type checking logic using TypeResolver
                let result = match check.target_type.as_str() {
                    "string" => matches!(&expr_plain, FhirPathValue::String(_)),
                    "integer" => matches!(&expr_plain, FhirPathValue::Integer(_)),
                    "decimal" => matches!(&expr_plain, FhirPathValue::Decimal(_)),
                    "boolean" => matches!(&expr_plain, FhirPathValue::Boolean(_)),
                    "date" => matches!(&expr_plain, FhirPathValue::Date(_)),
                    "dateTime" => matches!(&expr_plain, FhirPathValue::DateTime(_)),
                    "time" => matches!(&expr_plain, FhirPathValue::Time(_)),
                    _ => false, // For complex types, return false for now
                };
                self.wrap_plain_result(FhirPathValue::Boolean(result)).await
            }
            _ => {
                // For any remaining expression types, return error to identify missing implementations
                Err(crate::core::FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    format!(
                        "Metadata-aware evaluation not yet implemented for expression type: {:?}",
                        expr
                    ),
                ))
            }
        }
    }

    /// Convert WrappedCollection to plain FhirPathValue for compatibility with standard evaluators
    fn wrapped_to_plain(&self, wrapped_result: &WrappedCollection) -> FhirPathValue {
        if wrapped_result.is_empty() {
            // In FHIRPath, empty results are always represented as empty collections, never as null/Empty
            FhirPathValue::Collection(Vec::new())
        } else if wrapped_result.len() == 1 {
            wrapped_result.first().unwrap().as_plain().clone()
        } else {
            let values: Vec<FhirPathValue> = wrapped_result
                .iter()
                .map(|w| w.as_plain().clone())
                .collect();
            FhirPathValue::Collection(values)
        }
    }

    /// Wrap plain result with basic metadata
    async fn wrap_plain_result(&self, result: FhirPathValue) -> Result<WrappedCollection> {
        use crate::path::CanonicalPath;
        use crate::typing::type_utils;
        use crate::wrapped::{ValueMetadata, WrappedValue};

        match result {
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            FhirPathValue::Collection(values) => {
                let wrapped_values: Vec<WrappedValue> = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let fhir_type = type_utils::fhirpath_value_to_fhir_type(&value);
                        let path = CanonicalPath::parse(&format!("[{}]", i)).unwrap();
                        let metadata = ValueMetadata {
                            fhir_type,
                            resource_type: None,
                            path,
                            index: Some(i),
                        };
                        WrappedValue::new(value, metadata)
                    })
                    .collect();
                Ok(wrapped_values)
            }
            single_value => {
                let fhir_type = type_utils::fhirpath_value_to_fhir_type(&single_value);
                let metadata = ValueMetadata {
                    fhir_type,
                    resource_type: None,
                    path: CanonicalPath::empty(),
                    index: None,
                };
                Ok(collection_utils::single(WrappedValue::new(
                    single_value,
                    metadata,
                )))
            }
        }
    }

    /// Get reference to function evaluator (for extracting registry)
    pub fn function_evaluator(&self) -> &MetadataFunctionEvaluator {
        &self.function_evaluator
    }
}

#[async_trait]
impl ExpressionEvaluator for CompositeEvaluator {
    async fn evaluate(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Convert metadata-aware result back to plain result for compatibility
        let wrapped_result = self.evaluate_with_metadata(expr, context).await?;

        if wrapped_result.is_empty() {
            // In FHIRPath, empty results are always represented as empty collections, never as null/Empty
            Ok(FhirPathValue::Collection(Vec::new()))
        } else if wrapped_result.len() == 1 {
            Ok(wrapped_result.into_iter().next().unwrap().into_plain())
        } else {
            let values: Vec<FhirPathValue> =
                wrapped_result.into_iter().map(|w| w.into_plain()).collect();
            Ok(FhirPathValue::Collection(values))
        }
    }

    fn can_evaluate(&self, _expr: &ExpressionNode) -> bool {
        true // Composite evaluator can handle all expressions
    }

    fn evaluator_name(&self) -> &'static str {
        "CompositeEvaluator"
    }
}

#[async_trait]
impl MetadataAwareEvaluator for CompositeEvaluator {
    async fn evaluate_with_metadata(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Use our internal type resolver if provided one doesn't match
        if std::ptr::eq(resolver as *const _, &self.type_resolver as *const _) {
            self.evaluate_with_metadata(expr, context).await
        } else {
            // Temporarily switch type resolver
            let original_resolver = std::mem::replace(&mut self.type_resolver, resolver.clone());
            let result = self.evaluate_with_metadata(expr, context).await;
            self.type_resolver = original_resolver;
            result
        }
    }

    async fn initialize_root_context(
        &self,
        root_data: &crate::core::Collection,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        self.core_evaluator
            .initialize_root_context(root_data, resolver)
            .await
    }
}
