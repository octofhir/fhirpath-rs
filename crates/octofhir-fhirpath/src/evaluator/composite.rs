//! Composite evaluator with full metadata support
//!
//! This module provides the main CompositeEvaluator that
//! integrates all metadata-aware evaluators for comprehensive rich evaluation.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    ast::{BinaryOperator, ExpressionNode},
    core::types::Collection,
    core::{FP0001, FP0051, FP0200, FhirPathError, FhirPathValue, ModelProvider, Result, SharedTraceProvider},
    evaluator::{
        EvaluationContext, MetadataCollectionEvaluator, MetadataCoreEvaluator,
        MetadataFunctionEvaluator, MetadataNavigator,
        config::EngineConfig,
        traits::{
            ExpressionEvaluator, MetadataAwareCollectionEvaluator, MetadataAwareEvaluator,
            MetadataAwareFunctionEvaluator, MetadataAwareNavigator,
        },
    },
    path::CanonicalPath,
    registry::FunctionRegistry,
    typing::{TypeResolver, TypeResolverFactory},
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue, collection_utils},
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
    /// Metadata-aware lambda evaluator
    lambda_evaluator: tokio::sync::Mutex<Box<dyn crate::evaluator::LambdaEvaluator + Send + Sync>>,
    /// Model provider for type resolution
    model_provider: Arc<dyn ModelProvider>,
    /// Function registry for function resolution
    function_registry: Arc<FunctionRegistry>,
    /// Type resolver for metadata operations
    type_resolver: TypeResolver,
    /// Engine configuration
    config: EngineConfig,
    /// Trace provider for trace() function output
    trace_provider: SharedTraceProvider,
}

impl CompositeEvaluator {
    pub async fn new(
        _core_evaluator: Box<dyn ExpressionEvaluator + Send + Sync>,
        _function_evaluator: Box<dyn crate::evaluator::FunctionEvaluator + Send + Sync>,
        operator_evaluator: Box<dyn crate::evaluator::OperatorEvaluator + Send + Sync>,
        _collection_evaluator: Box<dyn crate::evaluator::CollectionEvaluator + Send + Sync>,
        lambda_evaluator: Box<dyn crate::evaluator::LambdaEvaluator + Send + Sync>,
        model_provider: Arc<dyn ModelProvider>,
        function_registry: Arc<FunctionRegistry>,
        config: EngineConfig,
    ) -> Self {
        Self::with_trace_provider(
            _core_evaluator,
            _function_evaluator,
            operator_evaluator,
            _collection_evaluator,
            lambda_evaluator,
            model_provider,
            function_registry,
            config,
            crate::core::trace::create_cli_provider(), // Default to CLI provider
        ).await
    }

    /// Create a CompositeEvaluator with custom trace provider
    pub async fn with_trace_provider(
        _core_evaluator: Box<dyn ExpressionEvaluator + Send + Sync>,
        _function_evaluator: Box<dyn crate::evaluator::FunctionEvaluator + Send + Sync>,
        operator_evaluator: Box<dyn crate::evaluator::OperatorEvaluator + Send + Sync>,
        _collection_evaluator: Box<dyn crate::evaluator::CollectionEvaluator + Send + Sync>,
        lambda_evaluator: Box<dyn crate::evaluator::LambdaEvaluator + Send + Sync>,
        model_provider: Arc<dyn ModelProvider>,
        function_registry: Arc<FunctionRegistry>,
        config: EngineConfig,
        trace_provider: SharedTraceProvider,
    ) -> Self {
        let type_resolver = TypeResolverFactory::create(model_provider.clone());

        Self {
            core_evaluator: MetadataCoreEvaluator::new(),
            navigator: MetadataNavigator::new(),
            function_evaluator: MetadataFunctionEvaluator::with_trace_provider(
                function_registry.clone(),
                model_provider.clone(),
                trace_provider.clone(),
            ),
            operator_evaluator,
            collection_evaluator: MetadataCollectionEvaluator::new(),
            lambda_evaluator: tokio::sync::Mutex::new(lambda_evaluator),
            model_provider: model_provider.clone(),
            function_registry,
            type_resolver,
            config,
            trace_provider,
        }
    }

    /// Get the trace provider for collecting traces
    pub fn trace_provider(&self) -> &SharedTraceProvider {
        &self.trace_provider
    }

    /// Check if a method name represents a lambda method that needs special handling
    fn is_lambda_method(&self, method_name: &str) -> bool {
        matches!(
            method_name,
            "where" | "select" | "sort" | "aggregate" | "all" | "exists" | "repeat" | "repeatAll"
        )
    }

    /// Check if a method name is a type function that expects type name arguments
    fn is_type_function(&self, method_name: &str) -> bool {
        matches!(method_name, "is" | "as" | "ofType")
    }

    /// Evaluate an argument for type functions, converting identifiers to strings
    async fn evaluate_type_function_argument(
        &mut self,
        arg: &crate::ast::ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        use crate::ast::ExpressionNode;

        match arg {
            ExpressionNode::Identifier(identifier) => {
                // Convert identifier to string value for type names
                let type_name = &identifier.name;
                let string_value = crate::core::FhirPathValue::String(type_name.clone());
                let metadata = crate::wrapped::ValueMetadata::primitive(
                    "string".to_string(),
                    crate::path::CanonicalPath::empty(),
                );
                let wrapped_value = WrappedValue::new(string_value, metadata);
                Ok(vec![wrapped_value])
            }
            ExpressionNode::PropertyAccess(property_access) => {
                // Handle property access patterns like System.Integer or FHIR.Patient
                if let ExpressionNode::Identifier(base_identifier) = &*property_access.object {
                    // Check if this looks like a type expression (namespace.type)
                    if matches!(base_identifier.name.as_str(), "System" | "FHIR") {
                        let type_name = format!("{}.{}", base_identifier.name, property_access.property);
                        let string_value = crate::core::FhirPathValue::String(type_name);
                        let metadata = crate::wrapped::ValueMetadata::primitive(
                            "string".to_string(),
                            crate::path::CanonicalPath::empty(),
                        );
                        let wrapped_value = WrappedValue::new(string_value, metadata);
                        Ok(vec![wrapped_value])
                    } else {
                        // For non-type property access, evaluate normally
                        Box::pin(self.evaluate_with_metadata(arg, context)).await
                    }
                } else {
                    // For complex property access, evaluate normally
                    Box::pin(self.evaluate_with_metadata(arg, context)).await
                }
            }
            ExpressionNode::TypeInfo(type_info) => {
                // Handle TypeInfo nodes directly
                let type_name = format!("{}.{}", type_info.namespace, type_info.name);
                let string_value = crate::core::FhirPathValue::String(type_name);
                let metadata = crate::wrapped::ValueMetadata::primitive(
                    "string".to_string(),
                    crate::path::CanonicalPath::empty(),
                );
                let wrapped_value = WrappedValue::new(string_value, metadata);
                Ok(vec![wrapped_value])
            }
            _ => {
                // For non-identifier arguments, evaluate normally
                Box::pin(self.evaluate_with_metadata(arg, context)).await
            }
        }
    }

    /// Evaluate a lambda method call with raw AST arguments
    async fn evaluate_lambda_method_call(
        &mut self,
        object: &WrappedCollection,
        method_name: &str,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        // Special cases for methods with no arguments that need special handling
        if arguments.is_empty() {
            match method_name {
                "sort" => {
                    return self
                        .evaluate_sort_method_composite(object, arguments, context)
                        .await;
                }
                "exists" => {
                    return self
                        .evaluate_exists_method_composite(object, arguments, context)
                        .await;
                }
                _ => return Ok(object.clone()),
            }
        }

        // Create lambda node from the first argument (lambda expression)
        let lambda_expression = &arguments[0];

        match method_name {
            "where" => {
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🚀 WHERE: Composite evaluator called with {} items",
                        object.len()
                    );
                }

                // Check for resourceType filter patterns for aggressive type casting
                let detected_resource_type = self.detect_resource_type_filter(lambda_expression);

                if let Some(ref target_resource_type) = detected_resource_type {
                    if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                        eprintln!(
                            "🚀 AGGRESSIVE TYPE CAST: Detected resourceType='{}' filter",
                            target_resource_type
                        );
                    }
                }

                // Filter the collection based on the lambda expression
                let mut filtered_results = Vec::new();

                for wrapped_item in object {
                    // Create lambda context with this item as $this
                    let lambda_context = self
                        .create_lambda_context_for_item(wrapped_item, context)
                        .await?;

                    // Evaluate the lambda expression in this context
                    let result = self
                        .evaluate_expression_with_metadata(lambda_expression, &lambda_context)
                        .await?;

                    // Check if result is truthy (follows FHIRPath boolean conversion)
                    if self.is_collection_truthy(&result) {
                        // AGGRESSIVE TYPE CASTING: If this is a resourceType filter, typecast the result
                        if let Some(ref target_type) = detected_resource_type {
                            let mut typecast_item = wrapped_item.clone();
                            // Update metadata to reflect the specific resource type
                            typecast_item.metadata.fhir_type = target_type.clone();
                            typecast_item.metadata.resource_type = Some(target_type.clone());
                            filtered_results.push(typecast_item);
                        } else {
                            filtered_results.push(wrapped_item.clone());
                        }
                    }
                }

                if let Some(ref target_type) = detected_resource_type {
                    if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                        eprintln!(
                            "🚀 AGGRESSIVE TYPE CAST: Filtered {} resources typecast to {}",
                            filtered_results.len(),
                            target_type
                        );
                    }
                }

                Ok(filtered_results)
            }
            "select" => {
                // Transform each item in the collection
                let mut selected_results = Vec::new();

                for (index, wrapped_item) in object.iter().enumerate() {
                    // Create lambda context with this item as $this and current index as $index
                    let lambda_context = self
                        .create_lambda_context_for_item_with_index(
                            wrapped_item,
                            context,
                            Some(index),
                        )
                        .await?;

                    // Evaluate the lambda expression in this context
                    let result = self
                        .evaluate_expression_with_metadata(lambda_expression, &lambda_context)
                        .await?;

                    // Add all results from this evaluation
                    selected_results.extend(result);
                }

                Ok(selected_results)
            }
            "sort" => {
                // Handle sorting with lambda expressions
                self.evaluate_sort_method_composite(object, arguments, context)
                    .await
            }
            "aggregate" => {
                // Handle aggregation with lambda expressions
                self.evaluate_aggregate_method_composite(object, arguments, context)
                    .await
            }
            "repeat" => {
                // Handle repeat traversal with safety mechanisms
                self.evaluate_repeat_method_composite(object, arguments, context)
                    .await
            }
            "repeatAll" => {
                // Handle repeatAll traversal with safety mechanisms
                self.evaluate_repeat_all_method_composite(object, arguments, context)
                    .await
            }
            "trace" => {
                // Get the trace name from the first argument
                let trace_name = if let crate::ast::ExpressionNode::Literal(literal) = &arguments[0]
                {
                    if let crate::ast::LiteralValue::String(name) = &literal.value {
                        name.clone()
                    } else {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            crate::core::error_code::FP0053,
                            "trace() first argument must be a string (trace name)".to_string(),
                        ));
                    }
                } else {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        "trace() first argument must be a string literal".to_string(),
                    ));
                };

                if arguments.len() == 1 {
                    // trace(name) - trace input values and return them
                    for (i, wrapped) in object.iter().enumerate() {
                        let trace_output = self.format_trace_value_with_metadata(wrapped);
                        eprintln!("TRACE[{}][{}]: {}", trace_name, i, trace_output);
                    }
                    Ok(object.clone())
                } else if arguments.len() == 2 {
                    // trace(name, projection) - evaluate projection on each item and trace the results
                    // BUT return the original input collection unchanged (trace doesn't transform output)
                    let projection_expression = &arguments[1];

                    for (i, wrapped_item) in object.iter().enumerate() {
                        // Create lambda context for each item
                        let lambda_context = self
                            .create_lambda_context_for_item_with_index(
                                wrapped_item,
                                context,
                                Some(i),
                            )
                            .await?;

                        // Evaluate the projection expression in this context
                        let projection_result = self
                            .evaluate_expression_with_metadata(
                                projection_expression,
                                &lambda_context,
                            )
                            .await?;

                        // Trace each projected value
                        for (j, projected) in projection_result.iter().enumerate() {
                            let trace_output = self.format_trace_value_with_metadata(projected);
                            eprintln!("TRACE[{}][{}:{}]: {}", trace_name, i, j, trace_output);
                        }
                    }

                    // trace() always returns the original input collection unchanged
                    Ok(object.clone())
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        "trace() requires 1 or 2 arguments: trace(name) or trace(name, projection)"
                            .to_string(),
                    ))
                }
            }
            "exists" => {
                // Handle exists() lambda method
                self.evaluate_exists_method_composite(object, arguments, context)
                    .await
            }
            "all" => {
                // Handle all() lambda method
                self.evaluate_all_method_composite(object, arguments, context)
                    .await
            }
            _ => {
                // Other lambda methods not yet implemented
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!("Lambda method '{}' not yet fully implemented", method_name),
                ))
            }
        }
    }

    /// Create a lambda context where $this is bound to the given item
    async fn create_lambda_context_for_item(
        &self,
        item: &WrappedValue,
        parent_context: &EvaluationContext,
    ) -> Result<EvaluationContext> {
        self.create_lambda_context_for_item_with_index(item, parent_context, None)
            .await
    }

    async fn create_lambda_context_for_item_with_index(
        &self,
        item: &WrappedValue,
        parent_context: &EvaluationContext,
        index: Option<usize>,
    ) -> Result<EvaluationContext> {
        let mut lambda_context = parent_context.clone();

        // Set the start_context to this single item (this becomes $this for implicit property access)
        lambda_context.start_context = crate::core::Collection::single(item.as_plain().clone());

        // Store the complete WrappedValue directly as a special variable
        // We create a custom JsonValue that encodes the metadata information
        // This allows property access within lambdas to maintain full type information
        let metadata_json = serde_json::json!({
            "value": item.as_plain(),
            "metadata": {
                "fhir_type": item.metadata().fhir_type,
                "resource_type": item.metadata().resource_type,
                "path": item.metadata().path.to_string(),
                "index": item.metadata().index
            }
        });
        lambda_context.set_variable(
            "__$this_wrapped__".to_string(),
            FhirPathValue::JsonValue(metadata_json.into()),
        );

        // Also explicitly bind $this variable for explicit $this property access
        lambda_context.set_variable("this".to_string(), item.as_plain().clone());

        // If index is provided, bind $index variable
        if let Some(idx) = index {
            lambda_context.set_variable("index".to_string(), FhirPathValue::Integer(idx as i64));
        }

        Ok(lambda_context)
    }

    /// Format a WrappedValue for trace output with metadata information
    fn format_trace_value_with_metadata(&self, wrapped: &WrappedValue) -> String {
        let value_str = match wrapped.as_plain() {
            crate::core::FhirPathValue::String(s) => format!("\"{}\"(String)", s),
            crate::core::FhirPathValue::Integer(i) => format!("{}(Integer)", i),
            crate::core::FhirPathValue::Decimal(d) => format!("{}(Decimal)", d),
            crate::core::FhirPathValue::Boolean(b) => format!("{}(Boolean)", b),
            crate::core::FhirPathValue::Date(d) => format!("{}(Date)", d.to_string()),
            crate::core::FhirPathValue::DateTime(dt) => format!("{}(DateTime)", dt.to_string()),
            crate::core::FhirPathValue::Time(t) => format!("{}(Time)", t.to_string()),
            crate::core::FhirPathValue::Quantity { value, unit, .. } => match unit {
                Some(u) => format!("{} {}(Quantity)", value, u),
                None => format!("{}(Quantity)", value),
            },
            crate::core::FhirPathValue::Resource(resource) => {
                let resource_type = resource
                    .get("resourceType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                format!("Resource({})", resource_type)
            }
            crate::core::FhirPathValue::JsonValue(json) => {
                if json.is_object() {
                    if let Some(resource_type) = json.get("resourceType").and_then(|v| v.as_str()) {
                        format!("Resource({})", resource_type)
                    } else {
                        format!(
                            "Object({})",
                            json.to_string().chars().take(50).collect::<String>()
                        )
                    }
                } else if json.is_array() {
                    format!("Array[{}]", json.as_array().map(|a| a.len()).unwrap_or(0))
                } else {
                    format!("JsonValue({})", json.to_string())
                }
            }
            crate::core::FhirPathValue::Collection(items) => {
                format!("Collection[{}]", items.len())
            }
            crate::core::FhirPathValue::Id(id) => format!("{}(Id)", id),
            crate::core::FhirPathValue::Base64Binary(data) => {
                format!("Base64[{}](Base64Binary)", data.len())
            }
            crate::core::FhirPathValue::Uri(uri) => format!("{}(Uri)", uri),
            crate::core::FhirPathValue::Url(url) => format!("{}(Url)", url),
            crate::core::FhirPathValue::TypeInfoObject { namespace, name } => {
                format!("{}:{}(TypeInfo)", namespace, name)
            }
            crate::core::FhirPathValue::Empty => "<empty>".to_string(),
        };

        // Add metadata information if available
        let type_info = if wrapped.fhir_type() != "unknown" {
            format!(" [{}]", wrapped.fhir_type())
        } else {
            String::new()
        };

        let path_info = if !wrapped.path().is_empty() {
            format!(" @{}", wrapped.path().to_string())
        } else {
            String::new()
        };

        format!("{}{}{}", value_str, type_info, path_info)
    }

    /// Evaluate an expression with metadata in a given context
    async fn evaluate_expression_with_metadata(
        &mut self,
        expression: &crate::ast::ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        Box::pin(self.evaluate_with_metadata(expression, context)).await
    }

    /// Check if a collection is truthy according to FHIRPath rules
    fn is_collection_truthy(&self, collection: &WrappedCollection) -> bool {
        if collection.is_empty() {
            return false;
        }

        // If collection has one item, check if it's truthy
        if collection.len() == 1 {
            match collection[0].as_plain() {
                crate::core::FhirPathValue::Boolean(b) => *b,
                crate::core::FhirPathValue::Integer(i) => *i != 0,
                crate::core::FhirPathValue::String(s) => !s.is_empty(),
                crate::core::FhirPathValue::Empty => false,
                _ => true, // Non-empty complex values are truthy
            }
        } else {
            // Non-empty collections are truthy
            true
        }
    }

    /// Handle sort method with lambda expressions and multiple criteria
    async fn evaluate_sort_method_composite(
        &mut self,
        object: &WrappedCollection,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        // Handle empty collection
        if object.is_empty() {
            return Ok(vec![]);
        }

        // If no sort criteria, perform natural sort
        if arguments.is_empty() {
            let mut sorted = object.clone();
            sorted
                .sort_by(|a, b| self.compare_fhirpath_values_naturally(a.as_plain(), b.as_plain()));
            return Ok(sorted);
        }

        // Parse sort criteria and evaluate each item
        let mut items_with_sort_keys = Vec::new();

        for wrapped_item in object {
            // Create lambda context with this item as $this
            let lambda_context = self
                .create_lambda_context_for_item(wrapped_item, context)
                .await?;

            // Evaluate all sort criteria for this item
            let mut sort_keys = Vec::new();
            for sort_expression in arguments {
                // Check if expression starts with unary minus for descending sort
                let (descending, actual_expression) =
                    Self::parse_sort_direction_static(sort_expression);

                // Evaluate the sort key expression
                let result = self
                    .evaluate_expression_with_metadata(actual_expression, &lambda_context)
                    .await?;

                // Convert result to a single sort key value
                let sort_key = if result.is_empty() {
                    crate::core::FhirPathValue::Empty
                } else {
                    result[0].as_plain().clone()
                };

                sort_keys.push((sort_key, descending));
            }

            items_with_sort_keys.push((wrapped_item.clone(), sort_keys));
        }

        // Sort the items based on their sort keys
        items_with_sort_keys.sort_by(|a, b| {
            let (_, keys_a) = a;
            let (_, keys_b) = b;

            // Compare each sort key in order
            for (_i, ((key_a, desc_a), (key_b, _desc_b))) in
                keys_a.iter().zip(keys_b.iter()).enumerate()
            {
                let cmp = self.compare_fhirpath_values_for_sorting(key_a, key_b, *desc_a);
                
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }

            std::cmp::Ordering::Equal
        });

        // Extract the sorted items
        let sorted_items = items_with_sort_keys
            .into_iter()
            .map(|(item, _)| item)
            .collect();
        Ok(sorted_items)
    }

    /// Parse sort direction from expression (detect unary minus for descending)
    fn parse_sort_direction_static(
        expression: &crate::ast::ExpressionNode,
    ) -> (bool, &crate::ast::ExpressionNode) {
        use crate::ast::{ExpressionNode, UnaryOperator};

        match expression {
            ExpressionNode::UnaryOperation(unary_node)
                if unary_node.operator == UnaryOperator::Negate =>
            {
                (true, unary_node.operand.as_ref()) // Descending
            }
            _ => (false, expression), // Ascending
        }
    }

    /// Compare FhirPathValues naturally for sorting
    fn compare_fhirpath_values_naturally(
        &self,
        a: &crate::core::FhirPathValue,
        b: &crate::core::FhirPathValue,
    ) -> std::cmp::Ordering {
        use crate::core::FhirPathValue;
        use std::cmp::Ordering;

        match (a, b) {
            // Same types - direct comparison
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a.cmp(b),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a.cmp(b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a.cmp(b),
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a.cmp(b),

            // Mixed numeric types
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                rust_decimal::Decimal::from(*a).cmp(b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                a.cmp(&rust_decimal::Decimal::from(*b))
            }

            // Different types - use type precedence: numbers < strings < booleans < others
            (FhirPathValue::Integer(_) | FhirPathValue::Decimal(_), FhirPathValue::String(_)) => {
                Ordering::Less
            }
            (FhirPathValue::String(_), FhirPathValue::Integer(_) | FhirPathValue::Decimal(_)) => {
                Ordering::Greater
            }
            (FhirPathValue::Integer(_) | FhirPathValue::Decimal(_), FhirPathValue::Boolean(_)) => {
                Ordering::Less
            }
            (FhirPathValue::Boolean(_), FhirPathValue::Integer(_) | FhirPathValue::Decimal(_)) => {
                Ordering::Greater
            }
            (FhirPathValue::String(_), FhirPathValue::Boolean(_)) => Ordering::Less,
            (FhirPathValue::Boolean(_), FhirPathValue::String(_)) => Ordering::Greater,

            // Empty values are treated as largest (sort last in ascending, first in descending)
            (FhirPathValue::Empty, FhirPathValue::Empty) => Ordering::Equal,
            (FhirPathValue::Empty, _) => Ordering::Greater,
            (_, FhirPathValue::Empty) => Ordering::Less,

            // All other cases - fallback to string comparison
            _ => format!("{:?}", a).cmp(&format!("{:?}", b)),
        }
    }

    /// Compare FhirPathValues for sorting with proper empty value handling based on sort direction
    fn compare_fhirpath_values_for_sorting(
        &self,
        a: &crate::core::FhirPathValue,
        b: &crate::core::FhirPathValue,
        descending: bool,
    ) -> std::cmp::Ordering {
        // For sorting, we always use natural comparison and handle descending by reversing
        let cmp = self.compare_fhirpath_values_naturally(a, b);
        if descending { cmp.reverse() } else { cmp }
    }

    /// Handle aggregate method with lambda expressions
    async fn evaluate_aggregate_method_composite(
        &mut self,
        object: &WrappedCollection,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        // Aggregate requires 1 or 2 arguments: lambda expression and optional initial value
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "aggregate() function requires 1 or 2 arguments: expression and optional initial value".to_string(),
            ));
        }

        let lambda_expression = &arguments[0];

        // Determine initial value
        let mut total = if arguments.len() == 2 {
            // Initial value provided as second argument
            let initial_value_expr = &arguments[1];
            let initial_result = self
                .evaluate_expression_with_metadata(initial_value_expr, context)
                .await?;
            if initial_result.is_empty() {
                crate::core::FhirPathValue::Empty
            } else {
                initial_result[0].as_plain().clone()
            }
        } else {
            // No initial value - start with Empty (will be handled by $total.empty() in expression)
            crate::core::FhirPathValue::Empty
        };

        // Iterate through each item in the collection
        for wrapped_item in object {
            // Create lambda context with this item as $this and current total as $total
            let mut lambda_context = self
                .create_lambda_context_for_item(wrapped_item, context)
                .await?;
            lambda_context.set_variable("total".to_string(), total.clone());

            // Evaluate the lambda expression
            let result = self
                .evaluate_expression_with_metadata(lambda_expression, &lambda_context)
                .await?;

            // Update the total with the result
            total = if result.is_empty() {
                crate::core::FhirPathValue::Empty
            } else {
                result[0].as_plain().clone()
            };
        }

        // Return the final aggregated result as a single-item collection
        let metadata = crate::wrapped::ValueMetadata::unknown(crate::path::CanonicalPath::root(
            "<aggregate>".to_string(),
        ));
        let wrapped_result = crate::wrapped::WrappedValue::new(total, metadata);
        Ok(vec![wrapped_result])
    }

    /// Evaluate repeat() method with safety mechanisms
    /// repeat(projection: expression) : collection
    /// https://build.fhir.org/ig/HL7/FHIRPath/#repeatprojection-expression--collection
    async fn evaluate_repeat_method_composite(
        &mut self,
        object: &WrappedCollection,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        if arguments.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                FP0001,
                "repeat() requires exactly one argument (projection expression)".to_string(),
            ));
        }

        let projection_expr = &arguments[0];
        let mut result = Vec::new();
        let mut work_set: std::collections::VecDeque<WrappedValue> =
            object.iter().cloned().collect();
        let mut visited = std::collections::HashSet::new();

        // Safety limits to prevent infinite loops and stack overflow
        const MAX_ITERATIONS: usize = 1000; // Reduced limit
        const MAX_DEPTH: usize = 100;
        const MAX_CONSECUTIVE_SAME: usize = 5; // Max consecutive identical values
        let mut iteration_count = 0;
        let mut consecutive_same_count = 0;
        let mut last_work_set_size = work_set.len();

        while let Some(current_item) = work_set.pop_front() {
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                return Err(FhirPathError::evaluation_error(
                    FP0200,
                    format!(
                        "repeat() exceeded maximum iterations limit ({})",
                        MAX_ITERATIONS
                    ),
                ));
            }

            // Create unique identifier for cycle detection using value and path
            let item_id = format!("{:?}_{}", current_item.as_plain(), current_item.path());

            // Check for cycles to prevent infinite loops
            if visited.contains(&item_id) {
                continue; // Skip already processed items (deduplication)
            }
            visited.insert(item_id);

            // Add current item to result
            result.push(current_item.clone());

            // Check depth to prevent stack overflow
            let current_depth = current_item.path().segments().len();
            if current_depth > MAX_DEPTH {
                return Err(FhirPathError::evaluation_error(
                    FP0200,
                    format!("repeat() exceeded maximum depth limit ({})", MAX_DEPTH),
                ));
            }

            // Evaluate projection on current item
            let item_context = self
                .create_lambda_context_for_item(&current_item, context)
                .await?;
            let projection_result = self
                .evaluate_expression_with_metadata(projection_expr, &item_context)
                .await?;

            // Detect infinite loops by checking if work set keeps growing at same rate
            if work_set.len() == last_work_set_size {
                consecutive_same_count += 1;
                if consecutive_same_count > MAX_CONSECUTIVE_SAME {
                    return Err(FhirPathError::evaluation_error(
                        FP0200,
                        "repeat() detected potential infinite loop - work set not decreasing"
                            .to_string(),
                    ));
                }
            } else {
                consecutive_same_count = 0;
            }

            // Add projection results to work set for further processing
            for projected_item in projection_result {
                // Validate input types to prevent incompatible operations
                if let Err(e) = self.validate_repeat_input(&projected_item, projection_expr) {
                    return Err(e);
                }

                // Detect cycles in projection results - if we keep getting the same values
                let projected_id =
                    format!("{:?}_{}", projected_item.as_plain(), projected_item.path());
                if visited.contains(&projected_id) {
                    // This projection produces a value we've already seen - potential infinite loop
                    continue;
                }

                work_set.push_back(projected_item);
            }

            last_work_set_size = work_set.len();
        }

        Ok(result)
    }

    /// Evaluate repeatAll() method with safety mechanisms  
    /// repeatAll(projection: expression) : collection
    /// https://build.fhir.org/ig/HL7/FHIRPath/#repeatallprojection-expression--collection
    async fn evaluate_repeat_all_method_composite(
        &mut self,
        object: &WrappedCollection,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        if arguments.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                FP0001,
                "repeatAll() requires exactly one argument (projection expression)".to_string(),
            ));
        }

        let projection_expr = &arguments[0];
        let mut result = Vec::new();
        let mut work_set: std::collections::VecDeque<WrappedValue> =
            object.iter().cloned().collect();

        // Safety limits to prevent infinite loops and stack overflow
        const MAX_ITERATIONS: usize = 1000; // Reduced limit
        const MAX_DEPTH: usize = 100;
        const MAX_CONSECUTIVE_SAME: usize = 5; // Max consecutive identical values
        let mut iteration_count = 0;
        let mut consecutive_same_count = 0;
        let mut last_work_set_size = work_set.len();
        let mut seen_values = std::collections::HashSet::new();

        while let Some(current_item) = work_set.pop_front() {
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                return Err(FhirPathError::evaluation_error(
                    FP0200,
                    format!(
                        "repeatAll() exceeded maximum iterations limit ({})",
                        MAX_ITERATIONS
                    ),
                ));
            }

            // Add current item to result (no deduplication in repeatAll)
            result.push(current_item.clone());

            // Check depth to prevent stack overflow
            let current_depth = current_item.path().segments().len();
            if current_depth > MAX_DEPTH {
                return Err(FhirPathError::evaluation_error(
                    FP0200,
                    format!("repeatAll() exceeded maximum depth limit ({})", MAX_DEPTH),
                ));
            }

            // Evaluate projection on current item
            let item_context = self
                .create_lambda_context_for_item(&current_item, context)
                .await?;
            let projection_result = self
                .evaluate_expression_with_metadata(projection_expr, &item_context)
                .await?;

            // Detect infinite loops by checking if work set keeps growing at same rate
            if work_set.len() == last_work_set_size {
                consecutive_same_count += 1;
                if consecutive_same_count > MAX_CONSECUTIVE_SAME {
                    return Err(FhirPathError::evaluation_error(
                        FP0200,
                        "repeatAll() detected potential infinite loop - work set not decreasing"
                            .to_string(),
                    ));
                }
            } else {
                consecutive_same_count = 0;
            }

            // Add projection results to work set for further processing
            for projected_item in projection_result {
                // Validate input types to prevent incompatible operations
                if let Err(e) = self.validate_repeat_input(&projected_item, projection_expr) {
                    return Err(e);
                }

                // For repeatAll, still detect obvious infinite loops (same value over and over)
                let projected_id =
                    format!("{:?}_{}", projected_item.as_plain(), projected_item.path());
                if seen_values.contains(&projected_id) && seen_values.len() < 5 {
                    // If we keep seeing the same few values repeatedly, it's likely infinite
                    continue;
                }
                seen_values.insert(projected_id);

                work_set.push_back(projected_item);
            }

            last_work_set_size = work_set.len();
        }

        Ok(result)
    }

    /// Validate repeat function input to prevent invalid operations
    fn validate_repeat_input(
        &self,
        item: &WrappedValue,
        projection_expr: &crate::ast::ExpressionNode,
    ) -> Result<()> {
        // Check for arithmetic operations in projection - these are usually invalid
        // repeat() should be used for FHIR property traversal, not mathematical sequences
        if let crate::ast::ExpressionNode::BinaryOperation(bin_op) = projection_expr {
            if matches!(
                bin_op.operator,
                crate::ast::BinaryOperator::Add
                    | crate::ast::BinaryOperator::Subtract
                    | crate::ast::BinaryOperator::Multiply
                    | crate::ast::BinaryOperator::Divide
                    | crate::ast::BinaryOperator::Modulo
            ) {
                // Check if $this is involved in arithmetic - this typically creates infinite sequences
                if let crate::ast::ExpressionNode::Variable(var) = bin_op.left.as_ref() {
                    if var.name == "this" {
                        return Err(FhirPathError::evaluation_error(
                            FP0051,
                            "repeat() with arithmetic on $this typically creates infinite sequences. Use repeat() for FHIR property traversal instead.".to_string(),
                        ));
                    }
                }
                if let crate::ast::ExpressionNode::Variable(var) = bin_op.right.as_ref() {
                    if var.name == "this" {
                        return Err(FhirPathError::evaluation_error(
                            FP0051,
                            "repeat() with arithmetic on $this typically creates infinite sequences. Use repeat() for FHIR property traversal instead.".to_string(),
                        ));
                    }
                }
            }
        }

        // Check if we're operating on primitive values with simple arithmetic
        // This is a common source of infinite loops like 1.repeat($this + 1)
        match item.as_plain() {
            crate::core::FhirPathValue::Integer(_)
            | crate::core::FhirPathValue::Decimal(_)
            | crate::core::FhirPathValue::String(_) => {
                // For primitive values, only allow safe projection expressions
                match projection_expr {
                    // Allow literals like repeat('test') 
                    crate::ast::ExpressionNode::Literal(_) => Ok(()),
                    // Allow property access like repeat(item)
                    crate::ast::ExpressionNode::Identifier(_) => Ok(()),
                    // Allow conditional expressions like repeat(iif(...))
                    crate::ast::ExpressionNode::FunctionCall(func) if func.name == "iif" => Ok(()),
                    // Reject arithmetic and other potentially dangerous operations on primitives
                    _ => {
                        Err(FhirPathError::evaluation_error(
                            FP0051,
                            "repeat() on primitive values should use literal or property projections, not complex expressions that may create infinite sequences".to_string(),
                        ))
                    }
                }
            }
            // For complex objects, allow more flexibility
            _ => Ok(()),
        }
    }

    /// Evaluate exists() lambda method
    /// exists(criteria?: expression) : boolean
    async fn evaluate_exists_method_composite(
        &mut self,
        object: &WrappedCollection,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        if arguments.is_empty() {
            // Simple exists() - check if collection is non-empty
            let exists = !object.is_empty();
            let metadata = crate::wrapped::ValueMetadata::unknown(
                crate::path::CanonicalPath::root("<exists>".to_string()),
            );
            let wrapped_result = crate::wrapped::WrappedValue::new(
                crate::core::FhirPathValue::Boolean(exists),
                metadata,
            );
            Ok(vec![wrapped_result])
        } else if arguments.len() == 1 {
            // exists(criteria) - check if any item matches criteria
            let criteria_expr = &arguments[0];

            for wrapped_item in object {
                let item_context = self
                    .create_lambda_context_for_item(wrapped_item, context)
                    .await?;
                let result = self
                    .evaluate_expression_with_metadata(criteria_expr, &item_context)
                    .await?;

                // If any result is truthy, exists returns true
                if !result.is_empty() {
                    if let Some(first_result) = result.first() {
                        match first_result.as_plain() {
                            crate::core::FhirPathValue::Boolean(true) => {
                                let metadata = crate::wrapped::ValueMetadata::unknown(
                                    crate::path::CanonicalPath::root("<exists>".to_string()),
                                );
                                let wrapped_result = crate::wrapped::WrappedValue::new(
                                    crate::core::FhirPathValue::Boolean(true),
                                    metadata,
                                );
                                return Ok(vec![wrapped_result]);
                            }
                            crate::core::FhirPathValue::Boolean(false) => continue,
                            _ => {
                                // Non-boolean values are considered truthy if non-empty
                                let metadata = crate::wrapped::ValueMetadata::unknown(
                                    crate::path::CanonicalPath::root("<exists>".to_string()),
                                );
                                let wrapped_result = crate::wrapped::WrappedValue::new(
                                    crate::core::FhirPathValue::Boolean(true),
                                    metadata,
                                );
                                return Ok(vec![wrapped_result]);
                            }
                        }
                    }
                }
            }

            // No item matched criteria
            let metadata = crate::wrapped::ValueMetadata::unknown(
                crate::path::CanonicalPath::root("<exists>".to_string()),
            );
            let wrapped_result = crate::wrapped::WrappedValue::new(
                crate::core::FhirPathValue::Boolean(false),
                metadata,
            );
            Ok(vec![wrapped_result])
        } else {
            Err(FhirPathError::evaluation_error(
                FP0001,
                "exists() requires 0 or 1 arguments".to_string(),
            ))
        }
    }

    /// Evaluate all() lambda method
    /// all(criteria: expression) : boolean
    async fn evaluate_all_method_composite(
        &mut self,
        object: &WrappedCollection,
        arguments: &[crate::ast::ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        if arguments.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                FP0001,
                "all() requires exactly one argument (criteria expression)".to_string(),
            ));
        }

        let criteria_expr = &arguments[0];

        // Empty collection: all() returns true (vacuous truth)
        if object.is_empty() {
            let metadata = crate::wrapped::ValueMetadata::unknown(
                crate::path::CanonicalPath::root("<all>".to_string()),
            );
            let wrapped_result = crate::wrapped::WrappedValue::new(
                crate::core::FhirPathValue::Boolean(true),
                metadata,
            );
            return Ok(vec![wrapped_result]);
        }

        // Check if ALL items satisfy the criteria
        for wrapped_item in object {
            let item_context = self
                .create_lambda_context_for_item(wrapped_item, context)
                .await?;
            let result = self
                .evaluate_expression_with_metadata(criteria_expr, &item_context)
                .await?;

            // Check if this item fails the criteria
            if result.is_empty() {
                // Empty result is considered false
                let metadata = crate::wrapped::ValueMetadata::unknown(
                    crate::path::CanonicalPath::root("<all>".to_string()),
                );
                let wrapped_result = crate::wrapped::WrappedValue::new(
                    crate::core::FhirPathValue::Boolean(false),
                    metadata,
                );
                return Ok(vec![wrapped_result]);
            }

            if let Some(first_result) = result.first() {
                match first_result.as_plain() {
                    crate::core::FhirPathValue::Boolean(false) => {
                        // Explicit false
                        let metadata = crate::wrapped::ValueMetadata::unknown(
                            crate::path::CanonicalPath::root("<all>".to_string()),
                        );
                        let wrapped_result = crate::wrapped::WrappedValue::new(
                            crate::core::FhirPathValue::Boolean(false),
                            metadata,
                        );
                        return Ok(vec![wrapped_result]);
                    }
                    crate::core::FhirPathValue::Boolean(true) => continue,
                    _ => {
                        // Non-boolean values are considered truthy if non-empty
                        continue;
                    }
                }
            }
        }

        // All items passed the criteria
        let metadata = crate::wrapped::ValueMetadata::unknown(crate::path::CanonicalPath::root(
            "<all>".to_string(),
        ));
        let wrapped_result =
            crate::wrapped::WrappedValue::new(crate::core::FhirPathValue::Boolean(true), metadata);
        Ok(vec![wrapped_result])
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
            ExpressionNode::Variable(var) => {
                // Check scope manager for variables defined by defineVariable first
                if let Some(scoped_value) = self.function_evaluator.get_scoped_variable(&var.name).await {
                    // Found variable in scope manager - return it wrapped
                    let wrapped_value = crate::wrapped::WrappedValue::new(
                        scoped_value,
                        crate::wrapped::ValueMetadata::unknown(crate::path::CanonicalPath::empty()),
                    );
                    Ok(vec![wrapped_value])
                } else {
                    // Fall back to core evaluator for built-in variables
                    self.core_evaluator
                        .evaluate_with_metadata(expr, context, &self.type_resolver)
                        .await
                }
            }
            ExpressionNode::Identifier(_)
            | ExpressionNode::Literal(_) => {
                self.core_evaluator
                    .evaluate_with_metadata(expr, context, &self.type_resolver)
                    .await
            }
            ExpressionNode::PropertyAccess(prop) => {
                // Special handling for $this in property access
                let object_result = match prop.object.as_ref() {
                    ExpressionNode::Variable(var) => {
                        if var.name == "this" {
                            // Direct evaluation through core evaluator for $this variables
                            self.core_evaluator
                                .evaluate_with_metadata(&prop.object, context, &self.type_resolver)
                                .await?
                        } else {
                            Box::pin(self.evaluate_with_metadata(&prop.object, context)).await?
                        }
                    }
                    ExpressionNode::Identifier(id) => {
                        if id.name == "$this" {
                            // Convert to Variable and evaluate through core evaluator
                            let var_node = crate::ast::VariableNode {
                                name: "$this".to_string(),
                                location: Some(crate::core::SourceLocation::point(0, 0, 0)),
                            };
                            let var_expr = ExpressionNode::Variable(var_node);
                            self.core_evaluator
                                .evaluate_with_metadata(&var_expr, context, &self.type_resolver)
                                .await?
                        } else {
                            Box::pin(self.evaluate_with_metadata(&prop.object, context)).await?
                        }
                    }
                    _ => Box::pin(self.evaluate_with_metadata(&prop.object, context)).await?,
                };
                let mut combined_result = Vec::new();

                for object_wrapped in object_result {
                    match self
                        .navigator
                        .navigate_property_with_metadata(
                            &object_wrapped,
                            &prop.property,
                            &self.type_resolver,
                        )
                        .await
                    {
                        Ok(nav_result) => combined_result.extend(nav_result),
                        Err(err) => {
                            // Check if this is a property access error on primitive type (FP0052)
                            if err.error_code() == &crate::core::error_code::FP0052 {
                                // Silently ignore - FHIRPath specification allows property access
                                // on mixed collections to ignore non-navigable items
                                continue;
                            } else {
                                // Re-raise other types of errors
                                return Err(err);
                            }
                        }
                    }
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
                    // Index operation should select the item at the given index from the collection
                    if let Some(selected_item) = object_result.get(index) {
                        Ok(vec![selected_item.clone()])
                    } else {
                        // Index out of bounds - return empty result
                        Ok(collection_utils::empty())
                    }
                } else {
                    // Invalid index (e.g., negative) - return empty result
                    Ok(collection_utils::empty())
                }
            }
            ExpressionNode::FunctionCall(func) => {
                // Special handling for defineVariable function
                if func.name == "defineVariable" {
                    return self.evaluate_define_variable_impl(func, context).await;
                }

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

                // Check if this is a lambda method that needs special handling
                if self.is_lambda_method(&method.method) {
                    // For lambda methods, pass the raw AST nodes instead of evaluating them
                    return self
                        .evaluate_lambda_method_call(
                            &object_result,
                            &method.method,
                            &method.arguments,
                            context,
                        )
                        .await;
                }

                // Special handling for trace method with projection
                if method.method == "trace" && method.arguments.len() == 2 {
                    // trace(name, projection) needs lambda handling for the projection
                    return self
                        .evaluate_lambda_method_call(
                            &object_result,
                            &method.method,
                            &method.arguments,
                            context,
                        )
                        .await;
                }

                // For regular methods, evaluate arguments normally
                let mut wrapped_args = Vec::new();
                for arg in &method.arguments {
                    let arg_result = if self.is_type_function(&method.method) {
                        // For type functions, convert identifier arguments to string values
                        self.evaluate_type_function_argument(arg, context).await?
                    } else {
                        Box::pin(self.evaluate_with_metadata(arg, context)).await?
                    };
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

                // Handle type literal patterns for 'is' and 'as' operators
                let right_result =
                    if matches!(binop.operator, BinaryOperator::Is | BinaryOperator::As) {
                        // Check if right side is a type literal pattern like System.Integer or FHIR.code
                        if let Some(type_name) = self.extract_type_literal(&binop.right) {
                            // Convert type literal to string value
                            let type_value = FhirPathValue::String(type_name);
                            let metadata = ValueMetadata {
                                fhir_type: "string".to_string(),
                                resource_type: None,
                                path: CanonicalPath::empty(),
                                index: None,
                                is_ordered: None,
                            };
                            collection_utils::single(WrappedValue::new(type_value, metadata))
                        } else {
                            Box::pin(self.evaluate_with_metadata(&binop.right, context)).await?
                        }
                    } else {
                        Box::pin(self.evaluate_with_metadata(&binop.right, context)).await?
                    };

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
                // For lambda expressions, use metadata-aware lambda evaluation
                let lambda_context = context.clone();

                // Convert the current context to a WrappedCollection for metadata-aware evaluation
                let context_collection = if context.start_context.is_empty() {
                    collection_utils::empty()
                } else {
                    // Convert context values to wrapped values with basic metadata
                    let wrapped_values: Vec<WrappedValue> = context
                        .start_context
                        .iter()
                        .enumerate()
                        .map(|(i, value)| {
                            let fhir_type =
                                crate::typing::type_utils::fhirpath_value_to_fhir_type(value);
                            let path =
                                crate::path::CanonicalPath::parse(&format!("[{}]", i)).unwrap();
                            let metadata = crate::wrapped::ValueMetadata {
                                fhir_type,
                                resource_type: None,
                                path,
                                index: Some(i),
                                is_ordered: None,
                            };
                            WrappedValue::new(value.clone(), metadata)
                        })
                        .collect();
                    wrapped_values
                };

                // Use metadata-aware lambda evaluator
                let mut lambda_evaluator = self.lambda_evaluator.lock().await;
                let result = lambda_evaluator
                    .evaluate_lambda(
                        lambda,
                        &context_collection,
                        &lambda_context,
                        &self.type_resolver,
                    )
                    .await?;

                Ok(result)
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
                    "string" | "String" => matches!(&expr_plain, FhirPathValue::String(_)),
                    "integer" | "Integer" => matches!(&expr_plain, FhirPathValue::Integer(_)),
                    "decimal" | "Decimal" => matches!(&expr_plain, FhirPathValue::Decimal(_)),
                    "boolean" | "Boolean" => matches!(&expr_plain, FhirPathValue::Boolean(_)),
                    "date" | "Date" => matches!(&expr_plain, FhirPathValue::Date(_)),
                    "dateTime" | "DateTime" => matches!(&expr_plain, FhirPathValue::DateTime(_)),
                    "time" | "Time" => matches!(&expr_plain, FhirPathValue::Time(_)),
                    // Handle FHIR resource types
                    _ => {
                        // For FHIR resources, check the resourceType field
                        if let FhirPathValue::Resource(resource) | FhirPathValue::JsonValue(resource) = &expr_plain {
                            if let Some(resource_type_value) = resource.get("resourceType") {
                                if let Some(resource_type) = resource_type_value.as_str() {
                                    resource_type.eq_ignore_ascii_case(&check.target_type)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
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
            FhirPathValue::Collection(Collection::empty())
        } else if wrapped_result.len() == 1 {
            wrapped_result.first().unwrap().as_plain().clone()
        } else {
            let values: Vec<FhirPathValue> = wrapped_result
                .iter()
                .map(|w| w.as_plain().clone())
                .collect();
            FhirPathValue::Collection(Collection::from_values(values))
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
                            is_ordered: None,
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
                    is_ordered: None,
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

    /// Detect if a lambda expression represents a resourceType filter for aggressive type casting
    /// Returns the target resource type if detected, None otherwise
    fn detect_resource_type_filter(&self, expr: &crate::ast::ExpressionNode) -> Option<String> {
        use crate::ast::{
            BinaryOperator, ExpressionNode, IdentifierNode, LiteralNode, literal::LiteralValue,
        };

        match expr {
            // Pattern: resourceType = 'SomeResourceType'
            ExpressionNode::BinaryOperation(binop) => {
                if let BinaryOperator::Equal = binop.operator {
                    // Check left side for resourceType property access
                    if let ExpressionNode::Identifier(IdentifierNode { name, .. }) = &*binop.left {
                        if name == "resourceType" {
                            // Check right side for string literal
                            if let ExpressionNode::Literal(LiteralNode {
                                value: LiteralValue::String(resource_type),
                                ..
                            }) = &*binop.right
                            {
                                return Some(resource_type.clone());
                            }
                        }
                    }
                    // Also check reverse pattern: 'SomeResourceType' = resourceType
                    if let ExpressionNode::Identifier(IdentifierNode { name, .. }) = &*binop.right {
                        if name == "resourceType" {
                            if let ExpressionNode::Literal(LiteralNode {
                                value: LiteralValue::String(resource_type),
                                ..
                            }) = &*binop.left
                            {
                                return Some(resource_type.clone());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Extract type literal from expression node (e.g., System.Integer -> "System.Integer")
    fn extract_type_literal(&self, expr: &ExpressionNode) -> Option<String> {
        use crate::ast::{ExpressionNode, IdentifierNode, PropertyAccessNode};

        match expr {
            // Pattern: System.Integer, System.String, FHIR.code, etc.
            ExpressionNode::PropertyAccess(PropertyAccessNode {
                object, property, ..
            }) => {
                if let ExpressionNode::Identifier(IdentifierNode { name, .. }) = object.as_ref() {
                    // Check for known type literal namespaces
                    if name == "System" || name == "FHIR" {
                        return Some(format!("{}.{}", name, property));
                    }
                }
            }
            // Pattern: Plain resource type identifiers like Patient, Observation, etc.
            ExpressionNode::Identifier(IdentifierNode { name, .. }) => {
                // Check if this looks like a FHIR resource type (starts with uppercase)
                if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    return Some(name.clone());
                }
            }
            _ => {}
        }

        None
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
            Ok(FhirPathValue::Collection(Collection::empty()))
        } else if wrapped_result.len() == 1 {
            Ok(wrapped_result.into_iter().next().unwrap().into_plain())
        } else {
            let values: Vec<FhirPathValue> =
                wrapped_result.into_iter().map(|w| w.into_plain()).collect();
            Ok(FhirPathValue::Collection(Collection::from_values(values)))
        }
    }

    fn can_evaluate(&self, _expr: &ExpressionNode) -> bool {
        true // Composite evaluator can handle all expressions
    }

    fn evaluator_name(&self) -> &'static str {
        "CompositeEvaluator"
    }
}

impl CompositeEvaluator {
    /// Special evaluation for defineVariable function
    /// 
    /// defineVariable(name, value) stores the current context as a variable
    /// and returns the current context unchanged for chaining.
    async fn evaluate_define_variable_impl(
        &mut self,
        func: &crate::ast::FunctionCallNode,
        context: &EvaluationContext,
    ) -> Result<WrappedCollection> {
        use crate::core::{FP0053, FhirPathError};

        // Validate arguments
        if func.arguments.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "defineVariable() requires at least 1 argument (variable name)".to_string(),
            ));
        }
        if func.arguments.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "defineVariable() accepts at most 2 arguments (variable name and optional value expression)".to_string(),
            ));
        }

        // Evaluate the variable name argument - must be a string
        let name_result = Box::pin(self.evaluate_with_metadata(&func.arguments[0], context)).await?;
        let var_name = if name_result.len() == 1 {
            let name_value = &name_result.first().unwrap().value;
            match name_value {
                crate::core::FhirPathValue::String(s) => s.clone(),
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "defineVariable() variable name must be a string".to_string(),
                    ));
                }
            }
        } else {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                "defineVariable() variable name must be a single string value".to_string(),
            ));
        };

        // Determine the value to store
        let var_value = if func.arguments.len() == 2 {
            // Use the provided expression value
            let value_result = Box::pin(self.evaluate_with_metadata(&func.arguments[1], context)).await?;
            if value_result.is_empty() {
                crate::core::FhirPathValue::Empty
            } else if value_result.len() == 1 {
                value_result.first().unwrap().clone().into_plain()
            } else {
                let values: Vec<_> = value_result.into_iter().map(|w| w.into_plain()).collect();
                crate::core::FhirPathValue::Collection(crate::core::Collection::from_values(values))
            }
        } else {
            // Use current context as the value (this is the standard FHIRPath behavior)
            if context.start_context.is_empty() {
                crate::core::FhirPathValue::Empty
            } else if context.start_context.len() == 1 {
                context.start_context.first().unwrap().clone()
            } else {
                crate::core::FhirPathValue::Collection(context.start_context.clone())
            }
        };

        // Delegate to the function evaluator to handle the scope management
        self.function_evaluator.handle_define_variable(var_name, var_value, context).await
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
